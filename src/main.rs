use std::error::Error;

use clap::Parser;
mod client;
mod server;

mod cli;
mod verifier;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = cli::Cli::parse();

    if args.verbose {
        tracing_subscriber::fmt::init();
    }
    tracing::info!("Verbose mode enabled");
    match args.command {
        cli::Commands::Server { bind } => server::invoke(bind).await,
        cli::Commands::Client { addr } => client::invoke(addr).await,
    }
}
