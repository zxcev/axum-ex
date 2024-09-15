use axum::{routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;

#[derive(Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
}

async fn health_check() -> &'static str {
    "ok"
}

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:3000";
    let app = Router::new().route("/health-check", get(health_check));
    let listener = TcpListener::bind(addr).await.unwrap();

    axum::serve(listener, app).await.unwrap();
    println!("server is running on http://{addr}");
}
