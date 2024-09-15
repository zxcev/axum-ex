use std::{collections::HashMap, sync::Arc};

use axum::{
    response::Html,
    routing::{get, post},
};
use sse_multi_senders::MultiSendersState;
use sse_single_sender::SingleSenderState;
use tokio::{net::TcpListener, sync::Mutex};

mod sse_multi_senders;
mod sse_single_sender;

async fn index() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

async fn multi() -> Html<&'static str> {
    Html(include_str!("../static/multi.html"))
}

#[tokio::main]
async fn main() {
    let single_sender_state = Arc::new(SingleSenderState {
        clients: Mutex::new(HashMap::new()),
    });
    let multi_senders_state = Arc::new(MultiSendersState {
        clients: Mutex::new(HashMap::new()),
    });

    // 공통 라우터
    let common_routes = axum::Router::new()
        .route("/", get(index))
        .route("/multi", get(multi));

    // SingleSenderState를 사용하는 라우터
    let single_sender_routes = axum::Router::new()
        .route("/connect/:user_id", get(sse_single_sender::connect))
        .route(
            "/send/:from_id/:to_id/:message",
            post(sse_single_sender::send),
        )
        .with_state(single_sender_state);

    // MultiSendersState를 사용하는 라우터
    let multi_senders_routes = axum::Router::new()
        .route("/connect-multi/:user_id", get(sse_multi_senders::connect))
        .route(
            "/send-multi/:from_id/:to_id/:message",
            post(sse_multi_senders::send),
        )
        .with_state(multi_senders_state);

    // 모든 라우터 병합
    let app = common_routes
        .merge(single_sender_routes)
        .merge(multi_senders_routes);
    let addr = "127.0.0.1:3000";
    let listener = TcpListener::bind(addr).await.unwrap();

    println!("server is listening on {addr}");
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();

    // await.unwrap();
}
