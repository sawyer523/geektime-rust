use clap::Parser;

use crate::{CmdExector, ReplContext, ReplDisplay};
use crate::cli::ReplResult;

#[derive(Parser, Debug)]
pub struct SchemaOpts {
    #[arg(help = "The name of dataset")]
    pub name: String,
}

pub fn schema(args: clap::ArgMatches, ctx: &mut ReplContext) -> ReplResult {
    let name = args
        .get_one::<String>("name")
        .expect("name is required")
        .to_string();

    let (msg, rx) = crate::ReplMsg::new(SchemaOpts::new(name));
    Ok(ctx.send(msg, rx))
}

impl CmdExector for SchemaOpts {
    async fn execute<T: crate::Backend>(self, backend: &mut T) -> anyhow::Result<String> {
        let df = backend.schema(&self.name).await?;
        df.display().await
    }
}

impl SchemaOpts {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}
