use axum::{Extension, Json};
use axum::extract::State;
use axum::response::IntoResponse;
use axum_macros::debug_handler;

use chat_core::User;

use crate::{AppError, AppState};

#[debug_handler]
#[utoipa::path(
    get,
    path = "/api/users",
    tag = "workspace",
    responses(
        (status = 200, description = "List of chat users", body = Vec<User>),
    ),
    security(
        ("token" = [])
    )
)]
pub(crate) async fn list_chat_user_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let ws_id = user.ws_id;
    let users = state.fetch_chat_users(ws_id as _).await?;
    Ok(Json(users))
}
