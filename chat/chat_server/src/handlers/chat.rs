use axum::response::IntoResponse;
use axum_macros::debug_handler;

#[debug_handler]
pub(crate) async fn list_chat_handler() -> impl IntoResponse {
    "list_chat"
}

#[debug_handler]
pub(crate) async fn create_chat_handler() -> impl IntoResponse {
    "create_chat"
}

#[debug_handler]
pub(crate) async fn update_chat_handler() -> impl IntoResponse {
    "update_chat"
}

#[debug_handler]
pub(crate) async fn delete_chat_handler() -> impl IntoResponse {
    "delete_chat"
}
