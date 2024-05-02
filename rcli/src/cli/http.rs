use std::path::PathBuf;

use clap::Parser;
use enum_dispatch::enum_dispatch;

use crate::{process_http_serve, CmdExecutor};

use super::verify_path;

#[derive(Debug, Parser)]
#[enum_dispatch(CmdExecutor)]
pub enum HttpSubCommand {
    #[command(about = "Serve static files over HTTP.")]
    Serve(HttpServeOpts),
}

#[derive(Debug, Parser)]
pub struct HttpServeOpts {
    #[arg(short, long, default_value = ".", value_parser = verify_path)]
    pub dir: PathBuf,

    #[arg(short, long, default_value = "8080")]
    pub port: u16,
}

impl CmdExecutor for HttpServeOpts {
    async fn execute(self) -> anyhow::Result<()> {
        process_http_serve(self.dir, self.port).await
    }
}
