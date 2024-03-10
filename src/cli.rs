use std::net::SocketAddr;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = "A QUIC Proxy for SSH")]
pub(crate) struct Cli {
    #[arg(short, long, default_value = "false")]
    pub(crate) verbose: bool,
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    Server {
        #[arg(default_value = "127.0.0.1:4242")]
        bind: SocketAddr,
    },
    Client {
        #[arg(default_value = "127.0.0.1:4242")]
        addr: SocketAddr,
    },
}
