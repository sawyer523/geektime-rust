use axum::response::IntoResponse;
use axum_macros::debug_handler;

pub(crate) use agent::*;
pub(crate) use auth::*;
pub(crate) use chat::*;
pub(crate) use messages::*;
pub(crate) use workspace::*;

mod auth;
mod chat;
mod messages;
mod workspace;
mod agent;

#[debug_handler]
pub(crate) async fn index_handler() -> impl IntoResponse {
    "index"
}
