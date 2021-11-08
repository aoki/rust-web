mod db;
mod handlers;
mod model;
mod schema;
use crate::handlers::*;
use axum::{
    handler::{get, post},
    http::Request,
    AddExtensionLayer, Router,
};
use dotenv::dotenv;
use std::{env, net::SocketAddr};
use tower_http::{
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::{Level, Span};
#[macro_use]
extern crate diesel;

#[tokio::main]
async fn main() {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "server=debug, tower_http=debug")
    }

    tracing_subscriber::fmt::init();

    let server = Server::new();

    // うまく別の関数に切り出せなかった
    let app = Router::new()
        .route("/", get(|| async { "foo" }))
        .route("/logs", post(handle_post_logs))
        .route("/csv", post(handle_post_csv))
        .route("/csv", get(handle_get_csv))
        .route("/logs", get(handle_get_logs))
        .layer(AddExtensionLayer::new(server))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().include_headers(true))
                .on_request(|_request: &Request<_>, _span: &Span| {
                    DefaultOnRequest::new().level(Level::INFO);
                    tracing::info!("{:?}", _request)
                })
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(LatencyUnit::Micros),
                ),
        );

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// サーバーで持ち回る状態
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};

#[derive(Clone)]
pub struct Server {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl Server {
    pub fn new() -> Self {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_RUL is not set");
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = Pool::builder()
            .build(manager)
            .expect("Failed to create pool.");

        Server { pool }
    }
}
