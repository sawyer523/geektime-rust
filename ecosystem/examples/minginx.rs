use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::{
    io,
    net::{TcpListener, TcpStream},
};
use tracing::{info, level_filters::LevelFilter, warn};
use tracing_subscriber::{fmt::Layer, Layer as _, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Config {
    listen_addr: String,
    upstream_addr: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();
    let config = resolve_config();
    let config = Arc::new(config);
    info!("config: {:?}", config);

    let listener = TcpListener::bind(&config.listen_addr).await?;
    loop {
        let (client, addr) = listener.accept().await?;
        info!("accepted connection from: {}", addr);
        let clone_config = Arc::clone(&config);
        tokio::spawn(async move {
            let upstream = TcpStream::connect(&clone_config.upstream_addr).await?;
            proxy(client, upstream).await?;
            Ok::<(), anyhow::Error>(())
        });
    }

    #[allow(unreachable_code)]
    Ok(())
}

async fn proxy(mut client: TcpStream, mut upstream: TcpStream) -> Result<()> {
    let (mut client_reader, mut client_writer) = client.split();
    let (mut upstream_reader, mut upstream_writer) = upstream.split();
    let client_to_upstream = io::copy(&mut client_reader, &mut upstream_writer);
    let upstream_to_client = io::copy(&mut upstream_reader, &mut client_writer);
    match tokio::try_join!(client_to_upstream, upstream_to_client) {
        Ok((n, m)) => info!(
            "proxied {} bytes from client to upstream, {} bytes from upstream to client",
            n, m
        ),
        Err(e) => warn!("error proxying: {:?}", e),
    }
    Ok(())
}

fn resolve_config() -> Config {
    Config {
        upstream_addr: "0.0.0.0:8080".to_string(),
        listen_addr: "0.0.0.0:8081".to_string(),
    }
}