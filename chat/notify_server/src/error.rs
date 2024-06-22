use axum::body::Body;
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorOutput {
    pub error: String,
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("password hash error: {0}")]
    JwtError(#[from] jwt_simple::Error),
}

#[allow(unused)]
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
            Self::JwtError(_) => StatusCode::FORBIDDEN,
            Self::IoError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(json!({"error": self.to_string() }))).into_response()
    }
}
