use std::{collections::HashMap, sync::Arc};

use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, Response, StatusCode},
    response::IntoResponse,
};
use futures::StreamExt;
use tokio::sync::{mpsc, Mutex};

pub struct SingleSenderState {
    pub clients: Mutex<HashMap<u64, mpsc::Sender<String>>>,
}

pub async fn connect(
    Path(client_id): Path<u64>,
    State(state): State<Arc<SingleSenderState>>,
) -> impl IntoResponse {
    // capacity = number of messages to buffer
    let (tx, rx) = mpsc::channel(100);

    // add client channel
    state.clients.lock().await.insert(client_id, tx);

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
    State(state): State<Arc<SingleSenderState>>,
) -> impl IntoResponse {
    if let Some(tx) = state.clients.lock().await.get(&to_id) {
        let _ = tx
            .send(format!(
                "[From {from_id}, To: {to_id}]\t\tMessage: {message}"
            ))
            .await;
        println!("Message sent");
    } else {
        println!("User not found");
    }
}
