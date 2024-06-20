use anyhow::Result;
use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{fmt::Layer, Layer as _, layer::SubscriberExt, util::SubscriberInitExt};

use notify_server::{AppConfig, get_router, setup_pg_listener};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let config = AppConfig::load()?;
    let addr = format!("0.0.0.0:{}", config.server.port);
    let (app, state) = get_router(config);
    let listener = TcpListener::bind(&addr).await?;
    info!("Listening on: {}", addr);

    setup_pg_listener(state).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
