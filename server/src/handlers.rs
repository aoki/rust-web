use crate::Server;
use axum::{http::StatusCode, response};
use axum_debug::debug_handler;

type State = axum::extract::Extension<Server>;

/// POST /csv のハンドラ
pub async fn handle_post_csv(state: State) -> String {
    println!("State: {}", state.name);
    todo!()
}

/// POST /logs のハンドラ
pub async fn handle_post_logs(state: State) -> String {
    println!("State: {}", state.name);
    todo!()
}

/// GET /logs のハンドラ
#[debug_handler]
pub async fn handle_get_logs(state: State) -> Result<response::Json<CountResponse>, StatusCode> {
    println!("State: {}", state.name);
    Ok(response::Json(CountResponse { count: 120 }))
}

/// GET /csv のハンドラ
#[debug_handler]
pub async fn handle_get_csv(state: State) -> String {
    format!("State: {}", state.name);
    todo!()
}

use serde::{Deserialize, Serialize};
#[derive(Deserialize, Serialize)]
pub struct CountResponse {
    count: u64,
}
