use askama::Template;
use axum::http::{Response, StatusCode};
use axum::service; // バージョンが上がると変わりそう
use axum::{extract, handler::get, response, AddExtensionLayer, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use axum::body::{Bytes, Full};
use axum::response::{Html, IntoResponse};
use std::convert::Infallible;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let state = MyApp {
        server_name: "server with state".into(),
    };

    let count_state = Arc::new(Mutex::new(Counter { counter: 0 }));

    // build our application with a single route
    let app = Router::new()
        .nest(
            "/static",
            service::get(ServeDir::new("./static")).handle_error(|error: std::io::Error| {
                Ok::<_, Infallible>((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unhandled internal error: {}", error),
                ))
            }),
        )
        .route("/greet/:name", get(greet))
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

// ---------------------------------------------
// Template
// ---------------------------------------------

async fn greet(extract::Path(name): extract::Path<String>) -> impl IntoResponse {
    let template = HelloTemplate { name };
    HtmlTemplate(template)
}

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate {
    name: String,
}

struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    type Body = Full<Bytes>;
    type BodyError = Infallible;

    fn into_response(self) -> Response<Self::Body> {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Full::from(format!(
                    "Failed to render template. Error: {}",
                    err
                )))
                .unwrap(),
        }
    }
}

// ---------------------------------------------
// State
// ---------------------------------------------

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

// ---------------------------------------------
// Path
// ---------------------------------------------

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
