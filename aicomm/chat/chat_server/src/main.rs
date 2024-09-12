use anyhow::Result;
use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{fmt::Layer, Layer as _, layer::SubscriberExt, util::SubscriberInitExt};

use chat_server::{AppConfig, AppState, get_router};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let config = AppConfig::load()?;
    let addr = format!("0.0.0.0:{}", config.server.port);

    let state = AppState::try_new(config).await?;
    let app = get_router(state).await?;
    let listener = TcpListener::bind(&addr).await?;
    info!("Listening on: {}", addr);

    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
