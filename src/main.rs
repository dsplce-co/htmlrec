#[macro_use]
mod colors;
mod abstraction;
mod cli;
mod commands;
mod inject;
mod patched;
mod sys;
mod ui;

use clap::Parser;
use cli::Cli;

#[tokio::main]
async fn main() {
    if let Err(err) = Cli::parse().command.to_object().run().await {
        supercli::error!(&format!("Error: {:#}", err));
        std::process::exit(1);
    }
}
