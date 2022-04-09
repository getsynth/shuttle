// FIXME: after doing the lib/main split, cargo clippy --tests suddenly adds
// a bunch of unused code warnings. I can't see a good way to silence them.
// Any suggestions?
#![allow(dead_code)]
mod args;
mod client;
mod config;

use anyhow::Result;
use structopt::StructOpt;

use cargo_shuttle::{Args, Shuttle};

#[tokio::main]
async fn main() -> Result<()> {
    Shuttle::new().run(Args::from_args()).await
}
