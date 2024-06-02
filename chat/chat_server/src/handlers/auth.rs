use axum::response::IntoResponse;
use axum_macros::debug_handler;

#[debug_handler]
pub(crate) async fn signin_handler() -> impl IntoResponse {
    "signin"
}

#[debug_handler]
pub(crate) async fn signup_handler() -> impl IntoResponse {
    "signup"
}