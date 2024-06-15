use axum::{Extension, Json};
use axum::extract::State;
use axum::response::IntoResponse;
use axum_macros::debug_handler;

use crate::{AppError, AppState, User};
use crate::models::Workspace;

#[debug_handler]
pub(crate) async fn list_chat_user_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let ws_id = user.ws_id;
    let users = Workspace::fetch_all_chat_users(ws_id as _, &state.pool).await?;
    Ok(Json(users))
}
