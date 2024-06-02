use axum::response::IntoResponse;
use axum_macros::debug_handler;

#[debug_handler]
pub(crate) async fn send_message_handler() -> impl IntoResponse {
    "send_message"
}

#[debug_handler]
pub(crate) async fn list_messages_handler() -> impl IntoResponse {
    "list_messages"
}
