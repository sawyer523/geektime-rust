use axum::{Extension, Json};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum_macros::debug_handler;

use crate::{AppError, AppState, CreateChat, PatchChat, User};
use crate::models::Chat;

#[debug_handler]
pub(crate) async fn list_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let chat = Chat::fetch_all(user.ws_id as _, &state.pool).await?;
    Ok((StatusCode::OK, Json(chat)))
}

#[debug_handler]
pub(crate) async fn create_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Json(input): Json<CreateChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = Chat::create(input, user.ws_id as _, &state.pool).await?;
    Ok((StatusCode::CREATED, Json(chat)))
}

#[debug_handler]
pub(crate) async fn get_chat_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let chat = Chat::get_by_id(id as _, &state.pool).await?;
    match chat {
        Some(chat) => Ok(Json(chat)),
        None => Err(AppError::NotFound(format!("chat id {id}"))),
    }
}

#[debug_handler]
pub(crate) async fn update_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(input): Json<PatchChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = Chat::update(&input, id, user.ws_id, &state.pool).await?;
    Ok((StatusCode::ACCEPTED, Json(chat)))
}

#[debug_handler]
pub(crate) async fn delete_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let chat = Chat::get_by_id(id, &state.pool).await?;
    match chat {
        Some(chat) => {
            if chat.ws_id == user.ws_id {
                Chat::delete(id, &state.pool).await?;
                Ok(StatusCode::ACCEPTED)
            } else {
                Err(AppError::PermissionDenied(
                    "chat is not belong to user".to_string(),
                ))
            }
        }
        None => Err(AppError::NotFound(format!("chat id {id}"))),
    }
}
