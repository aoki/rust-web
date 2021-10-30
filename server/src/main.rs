use axum::{extract, handler::get, AddExtensionLayer, Router};

#[tokio::main]
async fn main() {
    let server = Server::new();

    let app = Router::new()
        .route(
            "/",
            get(|_state: extract::Extension<Server>| async move {
                format!("Hello, World! {}", _state.name)
            }),
        )
        .route(
            "/foo",
            get(
                |_state: extract::Extension<Server>| async move { format!("/foo {}", _state.name) },
            ),
        )
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
