use axum::extract::Query;
use axum::http::StatusCode;
use axum::{
    extract::{Multipart, Path, State},
    http::HeaderMap,
    response::IntoResponse,
    Extension, Json,
};
use axum_macros::debug_handler;
use tokio::fs;
use tracing::{info, warn};

use chat_core::User;

use crate::{AppError, AppState, ChatFile, CreateMessage, ListMessages};

#[debug_handler]
#[utoipa::path(
    post,
    path = "/api/chats/{id}",
    tag = "message",
    params(
        ("id" = u64, Path, description = "Chat id"),
        CreateMessage,
    ),
    responses(
        (status = 201, description = "Message created", body = Message),
        (status = 400, description = "Invalid input", body = ErrorOutput),
    ),
    security(
        ("token" = [])
    )
)]
pub(crate) async fn send_message_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Path(chat_id): Path<u64>,
    Json(input): Json<CreateMessage>,
) -> Result<impl IntoResponse, AppError> {
    let message = state.create_message(&input, chat_id, user.id as _).await?;
    Ok((StatusCode::CREATED, Json(message)))
}

#[debug_handler]
#[utoipa::path(
    get,
    path = "/api/chats/{id}/messages",
    tag = "message",
    params(
        ("id" = u64, Path, description = "Chat id"),
        ListMessages
    ),
    responses(
        (status = 200, description = "List of messages", body = Vec<Message>),
        (status = 400, description = "Invalid input", body = ErrorOutput),
    ),
    security(
        ("token" = [])
    )
)]
pub(crate) async fn list_message_handler(
    State(state): State<AppState>,
    Path(chat_id): Path<u64>,
    Query(input): Query<ListMessages>,
) -> Result<impl IntoResponse, AppError> {
    let messages = state.list_messages(input, chat_id).await?;
    Ok(Json(messages))
}

#[debug_handler]
#[utoipa::path(
    post,
    path = "/api/upload",
    tag = "message",
    responses(
        (status = 200, description = "File uploaded", body = Vec<String>),
    ),
    security(
        ("token" = [])
    )
)]
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
#[utoipa::path(
    get,
    path = "/api/download/{ws_id}/{path:.+}",
    tag = "message",
    responses(
        (status = 200, description = "File downloaded", body = Vec<u8>),
        (status = 404, description = "File not found", body = ErrorOutput),
    ),
    security(
        ("token" = [])
    )
)]
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
