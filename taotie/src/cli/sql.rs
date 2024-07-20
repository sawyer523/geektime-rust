use clap::{ArgMatches, Parser};

use crate::{Backend, CmdExector, ReplContext, ReplDisplay, ReplMsg};

use super::ReplResult;

#[derive(Parser, Debug)]
pub struct SqlOpts {
    #[arg(help = "The SQL query")]
    pub query: String,
}

pub fn sql(args: ArgMatches, ctx: &mut ReplContext) -> ReplResult {
    let query = args
        .get_one::<String>("query")
        .expect("query is required")
        .to_string();

    let (msg, rx) = ReplMsg::new(SqlOpts::new(query));
    Ok(ctx.send(msg, rx))
}

impl CmdExector for SqlOpts {
    async fn execute<T: Backend>(self, backend: &mut T) -> anyhow::Result<String> {
        let df = backend.sql(&self.query).await?;
        df.display().await
    }
}

impl SqlOpts {
    pub fn new(query: String) -> Self {
        Self { query }
    }
}
