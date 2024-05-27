use anyhow::Result;
use axum::{
    extract::{Path, State},
    Json,
    response::IntoResponse,
    Router, routing::{get, post},
};
use axum::response::Response;
use http::{header::LOCATION, HeaderMap, StatusCode};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use sqlx::{Error, FromRow, PgPool};
use thiserror::Error;
use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{fmt::Layer, Layer as _, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Deserialize)]
struct ShortenReq {
    url: String,
}

#[derive(Debug, Serialize)]
struct ShortenRes {
    url: String,
}

#[derive(Debug, Clone)]
struct AppState {
    db: PgPool,
}

#[derive(Debug, FromRow)]
struct UrlRecord {
    #[sqlx(default)]
    id: String,
    #[sqlx(default)]
    url: String,
}

#[derive(Debug, Error)]
pub enum ShortenerError {
    #[error("Unprocessable entity: {0}")]
    Unprocessable(String),
    #[error("Not found: {0}")]
    Notfound(String),
    #[error("Database error")]
    Database(#[from] sqlx::Error)
}

const LISTEN_ADDR: &str = "127.0.0.1:9876";

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let url = "postgres://localhost:5432/shortener";
    let state = AppState::try_new(url).await?;
    info!("Connected to database: {url}");
    let listener = TcpListener::bind(LISTEN_ADDR).await?;
    info!("Listening on: {}", LISTEN_ADDR);

    let app = Router::new()
        .route("/", post(shorten))
        .route("/:id", get(redirect))
        .with_state(state);

    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

async fn shorten(
    State(state): State<AppState>,
    Json(data): Json<ShortenReq>,
) -> Result<impl IntoResponse, ShortenerError> {
    info!("Shorten request: {:?}", data);
    let ret = state.shorten(&data.url).await;
    match ret {
        Ok(id) => {
            let body = Json(ShortenRes {
                url: format!("http://{}/{}", LISTEN_ADDR, id),
            });
            Ok((StatusCode::CREATED, body))
        },
        Err(e) => Err(e)
    }
}

async fn redirect(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ShortenerError> {
    let ret = state
        .get_url(&id)
        .await;
    match ret { 
        Ok(url) => {
            let mut headers = HeaderMap::new();
            headers.insert(LOCATION, url.parse().unwrap());
            Ok((StatusCode::PERMANENT_REDIRECT, headers))
        },
        Err(e) => {
            Err(e)
        }
        
    }
}

impl AppState {
    async fn try_new(url: &str) -> Result<Self> {
        let pool = PgPool::connect(url).await?;
        // Create table if not exists
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS urls (
                id CHAR(6) PRIMARY KEY,
                url TEXT NOT NULL UNIQUE
            )
            "#,
        )
            .execute(&pool)
            .await?;
        Ok(Self { db: pool })
    }

    async fn shorten(&self, url: &str) -> Result<String, ShortenerError> {
        loop {
            let id = nanoid!(6);
            match sqlx::query_as::<_, UrlRecord>(
                "INSERT INTO urls (id, url) VALUES ($1, $2) ON CONFLICT(url) DO UPDATE SET url=EXCLUDED.url RETURNING id",
            )
                .bind(&id)
                .bind(url)
                .fetch_one(&self.db)
                .await {
                Ok(ret) => return Ok(ret.id),
                Err(e) => match e {
                    Error::RowNotFound => continue,
                    _ => return Err(ShortenerError::Database(e)),
                }
            }
        }
    }

    async fn get_url(&self, id: &str) -> Result<String, ShortenerError> {
        match sqlx::query_as::<_, UrlRecord>("SELECT * FROM urls WHERE id = $1")
            .bind(id)
            .fetch_one(&self.db)
            .await {
            Ok(record) => Ok(record.url),
            Err(e) => {
                match e {
                    Error::RowNotFound => Err(ShortenerError::Notfound(id.into())),
                    _ => Err(ShortenerError::Database(e))
                }
            }
        }
    }
}

impl IntoResponse for ShortenerError {
    fn into_response(self) -> Response {
        match self {
            ShortenerError::Unprocessable(msg) => {
                (StatusCode::UNPROCESSABLE_ENTITY, msg).into_response()
            }
            ShortenerError::Notfound(msg) => (StatusCode::NOT_FOUND, format!("Url {} not found", msg)).into_response(),
            ShortenerError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response(),
        }
    }
}