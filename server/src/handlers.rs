use crate::model::NewLog;
use crate::{db, Server};
use anyhow::Error;
use axum::body::{Bytes, Full};
use axum::http::{HeaderMap, Response};
use axum::response::IntoResponse;
use axum::{extract, http::StatusCode, response};
use axum_debug::debug_handler;
use chrono::{DateTime, Utc};
use diesel::r2d2;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::convert::Infallible;
use tracing::debug;

type State = axum::extract::Extension<Server>;

/// POST /csv のハンドラ
pub async fn handle_post_csv(state: State) -> Result<&'static str, ErrorResponse> {
    todo!()
}

/// POST /logs のハンドラ
pub async fn handle_post_logs(
    server: State,
    log: extract::Json<api::logs::post::Request>,
) -> Result<StatusCode, ErrorResponse> {
    tracing::info!("{:?}", log);

    let log = NewLog {
        user_agent: log.user_agent.clone(),
        response_time: log.response_time,
        timestamp: log.timestamp.unwrap_or_else(|| Utc::now()).naive_utc(),
    };
    debug!("Recieved log: {:?}", log);
    match || -> anyhow::Result<()> {
        let conn = server.pool.get()?;
        db::insert_log(&conn, &log)?;
        Ok(())
    }() {
        Ok(_) => Ok(StatusCode::ACCEPTED),
        Err(_) => Err(ErrorResponse {
            error: anyhow::anyhow!("Insert error"),
        }),
    }
}

/// GET /logs のハンドラ
#[debug_handler]
pub async fn handle_get_logs(
    server: State,
    range: extract::Query<api::logs::get::Query>,
) -> Result<response::Json<serde_json::Value>, ErrorResponse> {
    tracing::info!("{:?}", range);

    match || -> anyhow::Result<Vec<api::Log>> {
        let conn = server.pool.get()?;
        let logs = db::logs(&conn, range.from, range.until)?;
        let logs = logs
            .into_iter()
            .map(|log| api::Log {
                user_agent: log.user_agent,
                response_time: log.response_time,
                timestamp: DateTime::from_utc(log.timestamp, Utc),
            })
            .collect();
        Ok(logs)
    }() {
        Ok(logs) => Ok(response::Json(serde_json::json!(api::logs::get::Response(
            logs
        )))),
        Err(_) => Err(ErrorResponse {
            error: anyhow::anyhow!("Select error"),
        }),
    }
}

/// GET /csv のハンドラ
#[debug_handler]
pub async fn handle_get_csv(
    state: State,
    range: extract::Query<api::logs::get::Query>,
) -> Result<(StatusCode, HeaderMap, Vec<u8>), ErrorResponse> {
    debug!("{:?}", range);

    let csv: Vec<u8> = vec![];
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "text/csv".parse().unwrap());

    Ok((StatusCode::CREATED, headers, csv))
}

#[derive(Deserialize, Serialize)]
pub struct CountResponse {
    count: u64,
}

// エラーハンドリング周り
// TODO: 仮エラーハンドリング。全部InternalServerError
// https://github.com/tokio-rs/axum/blob/main/examples/error-handling-and-dependency-injection/src/main.rs#L80
// https://zenn.dev/techno_tanoc/articles/99e54c82cb049f#%E3%83%AC%E3%82%B9%E3%83%9D%E3%83%B3%E3%82%B9
pub struct ErrorResponse {
    error: Error,
}

impl IntoResponse for ErrorResponse {
    type Body = Full<Bytes>;
    type BodyError = Infallible;

    fn into_response(self) -> Response<Self::Body> {
        let body = response::Json(json!({"error": self.error.to_string()}));
        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}

// TOOD: handlers に r2d2 の依存が入ってくるのを何とかする
impl From<r2d2::Error> for ErrorResponse {
    fn from(e: r2d2::Error) -> Self {
        ErrorResponse {
            error: anyhow::anyhow!("{}", e),
        }
    }
}

impl From<diesel::result::Error> for ErrorResponse {
    fn from(e: diesel::result::Error) -> Self {
        ErrorResponse {
            error: anyhow::anyhow!("{}", e),
        }
    }
}
