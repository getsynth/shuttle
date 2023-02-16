use std::{net::SocketAddr, path::PathBuf};

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
pub struct Args {
    /// Where to store auth state (such as users)
    #[arg(long, default_value = "./")]
    pub state: PathBuf,

    /// Address to bind to
    #[arg(long, default_value = "127.0.0.1:8000")]
    pub address: SocketAddr,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Start,
    Init(InitArgs),
}

#[derive(clap::Args, Debug, Clone)]
pub struct InitArgs {
    /// Name of initial account to create
    #[arg(long)]
    pub name: String,
    /// Key to assign to initial account
    #[arg(long)]
    pub key: Option<String>,
}
