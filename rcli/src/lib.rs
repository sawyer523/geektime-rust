use enum_dispatch::enum_dispatch;

pub use cli::*;
pub use process::*;
pub use utils::*;

mod cli;
mod process;
mod utils;

#[allow(async_fn_in_trait)]
#[enum_dispatch]
pub trait CmdExecutor {
    async fn execute(self) -> anyhow::Result<()>;
}
