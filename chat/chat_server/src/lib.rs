mod config;
mod handlers;
mod models;

use axum::routing::{get, patch, post};
use axum::Router;
use handlers::*;
use std::ops::Deref;
use std::sync::Arc;

pub use config::AppConfig;
pub use models::User;

#[derive(Debug, Clone)]
pub(crate) struct AppState {
    pub(crate) inner: Arc<AppStateInner>,
}

#[derive(Debug)]
pub(crate) struct AppStateInner {
    pub(crate) config: AppConfig,
}

pub fn get_router(config: AppConfig) -> Router {
    let state: AppState = AppState::new(config);

    let api = Router::new()
        .route("/signin", post(signin_handler))
        .route("/signup", post(signup_handler))
        .route("/chat", get(list_chat_handler).post(create_chat_handler))
        .route(
            "/chat/:id",
            patch(update_chat_handler)
                .delete(delete_chat_handler)
                .post(send_message_handler),
        )
        .route("/chat/:id/messages", get(list_messages_handler));

    let app = Router::new()
        .route("/", get(index_handler))
        .nest("/api", api)
        .with_state(state);
    app
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        Self {
            inner: Arc::new(AppStateInner { config }),
        }
    }
}
