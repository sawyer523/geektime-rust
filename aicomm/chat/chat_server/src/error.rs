use axum::body::Body;
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ErrorOutput {
    pub error: String,
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("create chat error: {0}")]
    CreateChatError(String),

    #[error("update chat error: {0}")]
    UpdateChatError(String),

    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("{0}")]
    ChatFileError(String),

    #[error("create message error: {0}")]
    CreateMessageError(String),

    #[error("email already exists: {0}")]
    EmailAlreadyExists(String),

    #[error("sql error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("password hash error: {0}")]
    PasswordHashError(#[from] argon2::password_hash::Error),

    #[error("password hash error: {0}")]
    JwtError(#[from] jwt_simple::Error),

    #[error("http header parse error: {0}")]
    HttpHeaderError(#[from] axum::http::header::InvalidHeaderValue),
}

impl ErrorOutput {
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response<Body> {
        let status = match self {
            Self::SqlxError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::PasswordHashError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::JwtError(_) => StatusCode::FORBIDDEN,
            Self::HttpHeaderError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::EmailAlreadyExists(_) => StatusCode::CONFLICT,
            Self::CreateChatError(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::PermissionDenied(_) => StatusCode::FORBIDDEN,
            Self::UpdateChatError(_) => StatusCode::BAD_REQUEST,
            Self::IoError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::CreateMessageError(_) => StatusCode::BAD_REQUEST,
            Self::ChatFileError(_) => StatusCode::BAD_REQUEST,
        };
        (status, Json(json!({"error": self.to_string() }))).into_response()
    }
}
