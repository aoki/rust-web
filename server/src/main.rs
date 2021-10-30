mod handlers;
use crate::handlers::*;

// use crate::handlers::*;
use axum::{
    handler::{get, post},
    AddExtensionLayer, Router,
};

#[tokio::main]
async fn main() {
    let server = Server::new();

    // うまく別の関数に切り出せなかった
    let app = Router::new()
        .route("/", get(|| async { "foo" }))
        .route("/logs", post(handle_post_logs))
        .route("/csv", post(handle_post_csv))
        .route("/csv", get(handle_get_csv))
        .route("/logs", get(handle_get_logs))
        .layer(AddExtensionLayer::new(server));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// サーバーで持ち回る状態
use serde::{Deserialize, Serialize};
#[derive(Deserialize, Serialize, Clone)]
pub struct Server {
    name: String,
}

impl Server {
    pub fn new() -> Self {
        Server {
            name: "hello".into(),
        }
    }
}
