use std::mem;

use anyhow::Result;
use tonic::transport::{Identity, Server, ServerTlsConfig};
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{fmt::Layer, Layer as _, layer::SubscriberExt, util::SubscriberInitExt};

use crm::{AppConfig, CrmService};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let mut config = AppConfig::load().expect("Failed to load config");

    let tls = mem::take(&mut config.server.tls);

    let addr = config.server.port;
    let addr = format!("[::1]:{}", addr).parse().unwrap();
    info!("CRM service listening on {}", addr);
    let svc = CrmService::try_new(config).await?.into_server()?;

    if let Some(tls) = tls {
        let identity = Identity::from_pem(tls.cert.clone(), tls.key.clone());
        Server::builder()
            .tls_config(ServerTlsConfig::new().identity(identity))?
            .add_service(svc)
            .serve(addr)
            .await?;
    } else {
        Server::builder().add_service(svc).serve(addr).await?;
    }
    Ok(())
}
