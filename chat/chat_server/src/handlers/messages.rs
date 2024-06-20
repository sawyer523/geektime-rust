use axum::{
    Extension,
    extract::{Multipart, Path, State},
    http::HeaderMap,
    Json, response::IntoResponse,
};
use axum::extract::Query;
use axum_macros::debug_handler;
use tokio::fs;
use tracing::{info, warn};

use chat_core::User;

use crate::{AppError, AppState, ChatFile, CreateMessage, ListMessage};

#[debug_handler]
pub(crate) async fn send_message_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Path(chat_id): Path<u64>,
    Json(input): Json<CreateMessage>,
) -> Result<impl IntoResponse, AppError> {
    let message = state.create_message(&input, chat_id, user.id as _).await?;
    Ok(Json(message))
}

#[debug_handler]
pub(crate) async fn list_messages_handler(
    State(state): State<AppState>,
    Path(chat_id): Path<u64>,
    Query(input): Query<ListMessage>,
) -> Result<impl IntoResponse, AppError> {
    let messages = state.list_messages(input, chat_id).await?;
    Ok(Json(messages))
}

#[debug_handler]
pub(crate) async fn upload_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let ws_id = user.ws_id as u64;
    let base_dir = &state.config.server.base_dir;
    let mut files = vec![];

    while let Some(field) = multipart.next_field().await.unwrap() {
        let filename = field.file_name().map(|name| name.to_string());
        let (Some(filename), Ok(data)) = (filename, field.bytes().await) else {
            warn!("Failed to read multipart field");
            continue;
        };

        let file = ChatFile::new(ws_id, &filename, &data);
        let path = file.path(base_dir);
        if path.exists() {
            info!("file already exists: {:?}", path);
            continue;
        }
        fs::create_dir_all(path.parent().expect("file path parent should exists")).await?;
        fs::write(&path, data).await?;
        files.push(file.url());
    }

    Ok(Json(files))
}

#[debug_handler]
pub(crate) async fn download_handler(
    Extension(user): Extension<User>,
    Path((ws_id, path)): Path<(i64, String)>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    if user.ws_id != ws_id {
        return Err(AppError::PermissionDenied(
            "file is not belong to user".to_string(),
        ));
    }
    let base_dir = state.config.server.base_dir.join(ws_id.to_string());
    let path = base_dir.join(path);
    if !path.exists() {
        return Err(AppError::NotFound("File doesn't exist".to_string()));
    }

    let mime = mime_guess::from_path(&path).first_or_octet_stream();
    let body = fs::read(path).await?;
    let mut headers = HeaderMap::new();
    headers.insert("content-type", mime.to_string().parse().unwrap());
    Ok((headers, body))
}
