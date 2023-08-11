#![warn(rust_2021_compatibility)]

use std::path::PathBuf;
use anyhow::Result;
use clap::{Parser, Subcommand, Args};
use tracing::{error, Level, debug};
use client::run_client;

use crate::server::Server;

mod client;
mod server;
mod model;
mod image;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Clone, Debug, Subcommand)]
enum Command {
   Server(ServerArgs) ,
   Client(ClientArgs),
}

#[derive(Args, Debug, Clone)]
struct ServerArgs {
    #[arg(default_value = "127.0.0.1:4433")]
    bind: String,
    #[arg(short, long)]
    path: PathBuf
}

#[derive(Args, Debug, Clone)]
struct ClientArgs {
    #[arg(default_value = "127.0.0.1:4433")]
    host: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    let cli = Cli::parse();

    match cli.command {
        Command::Server(ServerArgs { bind, path }) => {
	    debug! {"bind to host: {}", bind};
	    debug! {"images path: {}", &path.as_path().display()};

	    let server = Server::new(bind, &path)?;
	    if let Err(e) = server.run() {
	    	error!("failed {reason}", reason = e.to_string());
	    }
        }
        Command::Client(ClientArgs { host }) => {
	    debug! {"connect to host: {}", host};
	    if let Err(e) = run_client(host).await {
		error!("failed {reason}", reason = e.to_string());
	    }
	}
    }
    Ok(())
}
