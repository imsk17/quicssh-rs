use std::error::Error;

use clap::Parser;
mod client;
mod server;

mod cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = cli::Cli::parse();

    match args.command {
        cli::Commands::Server { bind } => server::invoke(bind).await,
        cli::Commands::Client { addr } => client::invoke(addr).await,
    }
}
