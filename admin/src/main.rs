use shuttle_admin::{args::Args, run};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args: Args = clap::Parser::parse();

    run(args).await
}
