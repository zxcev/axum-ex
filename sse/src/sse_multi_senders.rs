use std::{collections::HashMap, sync::Arc};

use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, Response, StatusCode},
    response::IntoResponse,
};
use futures::{future::join_all, StreamExt};
use tokio::sync::{mpsc, Mutex};

pub struct MultiSendersState {
    pub clients: Mutex<HashMap<u64, Vec<mpsc::Sender<String>>>>,
}

pub async fn connect(
    Path(client_id): Path<u64>,
    State(state): State<Arc<MultiSendersState>>,
) -> impl IntoResponse {
    // capacity = number of messages to buffer
    let (tx, rx) = mpsc::channel(100);

    // add client channel
    state
        .clients
        .lock()
        .await
        // check if the key exists
        .entry(client_id)
        // if not exists, insert with empty vector
        .or_insert_with(Vec::new)
        // always executed
        .push(tx);

    let stream = tokio_stream::wrappers::ReceiverStream::new(rx)
        .map(|msg| Ok::<_, std::convert::Infallible>(format!("data: {msg}\n\n")));
    let body = Body::from_stream(stream);

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/event-stream")
        .header(header::CONNECTION, "keep-alive")
        .body(body)
        .unwrap()
}

pub async fn send(
    Path((from_id, to_id, message)): Path<(u64, u64, String)>,
    State(state): State<Arc<MultiSendersState>>,
) -> impl IntoResponse {
    // for consuming minimum duration to get the lock
    let senders = state.clients.lock().await.get(&to_id).cloned();

    if let Some(senders) = senders {
        // execute async task outside lock
        let futs = senders.into_iter().map(|tx| {
            let message = format!("[From {from_id}, To: {to_id}]\t\tMessage: {message}");
            async move {
                let result = tx.send(message.clone()).await.is_ok();
                println!("{message}");
                result
            }
        });

        let results = join_all(futs).await;
        let success_count = results.iter().filter(|&&r| r).count();
        let total_count = results.len();
        println!("[Message Sent] ok: {success_count}, total: {total_count}");
    } else {
        println!("[User {to_id} not found, sender_id: {from_id}]");
    }
}
