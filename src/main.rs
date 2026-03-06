mod abstraction;
mod cli;
mod commands;
mod inject;

use clap::Parser;
use cli::Cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Cli::parse().command.to_object().run().await
}
