mod client;
mod protocol;
mod server;
mod turns;
mod validate;

use anyhow::{Context, Result, bail};
use std::env;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn")),
        )
        .with_writer(std::io::stderr)
        .init();

    let cwd = env::current_dir().context("Failed to read the current working directory.")?;
    let cwd = cwd.canonicalize().unwrap_or(cwd);

    match env::args().nth(1).as_deref() {
        Some("server") => server::Server::new(cwd).run().await,
        Some(command) => bail!("unknown subcommand {command}; expected server"),
        None => bail!("missing subcommand; expected server"),
    }
}
