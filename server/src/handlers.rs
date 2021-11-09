use crate::model::NewLog;
use crate::{db, Server};
use anyhow::Error;
use axum::body::{Bytes, Full};
use axum::extract::ContentLengthLimit;
use axum::extract::Multipart;
use axum::http::{HeaderMap, Response};
use axum::response::IntoResponse;
use axum::{extract, http::StatusCode, response};
use axum_debug::debug_handler;
use chrono::{DateTime, Utc};
use diesel::r2d2::{self, Pool};
use diesel::PgConnection;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::convert::Infallible;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use tracing::debug;

type State = axum::extract::Extension<Server>;

/// POST /csv のハンドラ
pub async fn handle_post_csv(
    server: State,
    ContentLengthLimit(mut multipart): ContentLengthLimit<Multipart, { 250 * 1024 * 1024 }>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let mut count = 0;
    while let Some(field) = multipart.next_field().await.unwrap() {
        println!("Field: {:?}", field);

        let name = field.name().unwrap().to_string();
        let content_type = field.content_type().unwrap().to_string();

        let data = field.bytes().await.unwrap();

        // text/csv　でない場合は無視する
        if content_type != "text/csv" {
            continue;
        };

        // データサイズが0の場合は無視する
        if data.len() == 0 {
            continue;
        }

        // TODO: ByteからReaderに繋ぐ方法がうまく思いつかなかったのでファイルにいったん書き出す
        let tmp_filename = format!("tmp-{}.csv", name);
        let mut file = std::fs::File::create(&tmp_filename).unwrap();
        let a: Result<Vec<_>, _> = data.bytes().collect();
        let a = a.unwrap();
        file.write_all(&a).expect("Error write file");

        // TODO: ファイルに書き出したのでdataはいらないんだが、旧コードをとりあえず利用するため（forで使ってる）いったんそのまま
        let size = load_data(&*server.pool.get().unwrap(), data, &tmp_filename).unwrap();
        count += size;
    }
    let body = response::Json(json!({ "count": count }));
    Ok(body)
}

fn load_data(conn: &PgConnection, data: Bytes, tmp_filename: &String) -> anyhow::Result<usize> {
    let mut ret = 0;
    let f = File::open(tmp_filename)?;
    let in_csv = BufReader::new(f);
    let in_logs = csv::Reader::from_reader(in_csv).into_deserialize::<::api::Log>();

    for logs in data.chunks(10) {
        println!("CHUNK: {:?}", logs);
    }

    // TODO: chunksがふつうにdataに生えてるのでそちらを使えそう？
    for logs in &in_logs.chunks(1000) {
        let logs = logs
            .filter_map(Result::ok)
            .map(|log| NewLog {
                user_agent: log.user_agent,
                response_time: log.response_time,
                timestamp: log.timestamp.naive_utc(),
            })
            .collect_vec();
        let inserted = db::insert_logs(conn, &logs)?;
        ret += inserted.len();
    }
    Ok(ret)
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
    server: State,
    range: extract::Query<api::logs::get::Query>,
) -> Result<(StatusCode, HeaderMap, Vec<u8>), ErrorResponse> {
    let conn = server.pool.get().unwrap();
    let logs = db::logs(&conn, range.from, range.until).unwrap();
    let v = Vec::new();
    let mut w = csv::Writer::from_writer(v);

    for log in logs.into_iter().map(|log| ::api::Log {
        user_agent: log.user_agent,
        response_time: log.response_time,
        timestamp: DateTime::from_utc(log.timestamp, Utc),
    }) {
        w.serialize(log).unwrap();
    }

    let csv = w.into_inner().unwrap();
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "text/csv".parse().unwrap());

    Ok((StatusCode::OK, headers, csv))
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
