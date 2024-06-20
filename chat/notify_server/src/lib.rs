use axum::{
    response::{Html, IntoResponse},
    Router,
    routing::get,
};
use futures::StreamExt;
use sqlx::postgres::PgListener;
use tracing::info;

use crate::sse::sse_handler;

mod sse;

const INDEX_HTML: &str = include_str!("../index.html");

pub fn get_router() -> Router {
    Router::new()
        .route("/", get(index_handler))
        .route("/events", get(sse_handler))
}

async fn index_handler() -> impl IntoResponse {
    Html(INDEX_HTML)
}

pub async fn setup_pg_listener() -> anyhow::Result<()> {
    let mut listerner = PgListener::connect("postgresql://localhost:5432").await?;
    listerner.listen("chat_updated").await?;
    listerner.listen("chat_message_created").await?;

    let mut stream = listerner.into_stream();
    tokio::spawn(async move {
        while let Some(notification) = stream.next().await {
            info!("notification: {:?}", notification);
        }
    });
    Ok(())
}
