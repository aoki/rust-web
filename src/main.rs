use axum::{extract, handler::get, response, AddExtensionLayer, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let state = MyApp {
        server_name: "server with state".into(),
    };

    let count_state = Arc::new(Mutex::new(Counter { counter: 0 }));

    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/count", get(increment))
        .layer(AddExtensionLayer::new(count_state))
        .route("/info", get(with_state))
        .layer(AddExtensionLayer::new(state)) // ライフタイムパラメータを使えば&で渡せそうなんだけど。。。
        .route("/user/:name/:age", get(user))
        .route("/name/:name", get(name))
        .route("/age/:name/:age", get(name_and_age));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// Stateを使って簡単な情報を表示する
#[derive(Clone, Debug, Deserialize, Serialize)]
struct MyApp {
    server_name: String,
}
async fn with_state(state: extract::Extension<MyApp>) -> String {
    format!("State: {}", state.server_name)
}

// Stateを使ってCountする
#[derive(Serialize, Deserialize)]
struct Counter {
    counter: i64,
}

type SharedState = Arc<Mutex<Counter>>;

async fn increment(
    extract::Extension(state): extract::Extension<SharedState>,
) -> response::Json<Counter> {
    let mut s = state.lock().await;
    s.counter += 1;
    let current = s.counter;
    response::Json(Counter { counter: current })
}

// https://docs.rs/axum/0.2.8/axum/extract/struct.Path.html
// 単純にPathからパラメーターを取り出す
async fn name(extract::Path(params): extract::Path<String>) -> String {
    let name = params;
    format!("Hello {}!", name)
}

// Tuple で複数のパラメーターを受け取る
async fn name_and_age(extract::Path(params): extract::Path<(String, u8)>) -> String {
    let name = params.0;
    let age = params.1;
    format!("Hello {}({})!", name, age)
}

// 構造体でパラメーターを受け取る
#[derive(Deserialize)]
struct UserParam {
    name: String,
    age: u8,
}
async fn user(extract::Path::<UserParam>(params): extract::Path<UserParam>) -> String {
    format!("Hello {}({}) from struct", params.name, params.age)
}
