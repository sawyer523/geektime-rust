mod auth;
mod chat;
mod messages;

use axum::response::IntoResponse;
use axum_macros::debug_handler;


pub(crate) use auth::*;
pub(crate) use chat::*;
pub(crate) use messages::*;

#[debug_handler]
pub(crate) async fn index_handler() -> impl IntoResponse {
    "index"
}