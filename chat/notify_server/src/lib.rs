use std::ops::Deref;
use std::sync::Arc;

use axum::http::Method;
use axum::middleware::from_fn_with_state;
use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use chat_core::{verify_token, DecodingKey, TokenVerifier, User};
pub use config::AppConfig;
use dashmap::DashMap;
pub use notif::*;
use tokio::sync::broadcast;
use tower_http::cors;
use tower_http::cors::CorsLayer;

use crate::error::AppError;
use crate::sse::sse_handler;

mod config;
mod error;
mod notif;
mod sse;

const INDEX_HTML: &str = include_str!("../index.html");

pub type UserMap = Arc<DashMap<u64, broadcast::Sender<Arc<AppEvent>>>>;

#[derive(Clone)]
pub struct AppState(Arc<AppStateInner>);

pub struct AppStateInner {
    config: AppConfig,
    dk: DecodingKey,
    users: UserMap,
}

pub async fn get_router(config: AppConfig) -> anyhow::Result<Router> {
    let state = AppState::new(config);
    notif::setup_pg_listener(state.clone()).await?;

    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
            Method::PUT,
            Method::OPTIONS,
        ])
        .allow_origin(cors::Any)
        .allow_headers(cors::Any);

    let app = Router::new()
        .route("/events", get(sse_handler))
        .layer(from_fn_with_state(state.clone(), verify_token::<AppState>))
        .layer(cors)
        .route("/", get(index_handler))
        .with_state(state.clone());

    Ok(app)
}

async fn index_handler() -> impl IntoResponse {
    Html(INDEX_HTML)
}

impl TokenVerifier for AppState {
    type Error = AppError;

    fn verify(&self, token: &str) -> Result<User, Self::Error> {
        Ok(self.dk.verify(token)?)
    }
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        let dk = DecodingKey::load(&config.auth.pk).expect("Failed to load public key");
        let users = Arc::new(DashMap::new());
        Self(Arc::new(AppStateInner { config, dk, users }))
    }
}
