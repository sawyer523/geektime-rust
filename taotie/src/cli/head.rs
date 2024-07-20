use clap::{ArgMatches, Parser};

use crate::{Backend, CmdExector, ReplContext, ReplDisplay, ReplMsg};
use crate::cli::ReplResult;

#[derive(Parser, Debug)]
pub struct HeadOpts {
    #[arg(help = "The name of dataset")]
    pub name: String,

    #[arg(short, long, help = "The number of rows to show")]
    pub n: Option<usize>,
}

pub fn head(args: ArgMatches, ctx: &mut ReplContext) -> ReplResult {
    let name = args
        .get_one::<String>("name")
        .expect("name is required")
        .to_string();
    let n = args.get_one::<usize>("n").copied();

    let (msg, rx) = ReplMsg::new(HeadOpts::new(name, n));
    Ok(ctx.send(msg, rx))
}

impl CmdExector for HeadOpts {
    async fn execute<T: Backend>(self, backend: &mut T) -> anyhow::Result<String> {
        let df = backend.head(&self.name, self.n.unwrap_or(5)).await?;
        df.display().await
    }
}

impl HeadOpts {
    pub fn new(name: String, n: Option<usize>) -> Self {
        Self { name, n }
    }
}
