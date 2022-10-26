use clap::Parser;
use tonic::transport::Endpoint;

#[derive(Parser, Debug)]
pub struct Args {
    /// Uri to the `.so` file to load
    #[arg(long, short)]
    pub file_path: String,

    /// Address to reach provisioner at
    #[clap(long, default_value = "localhost:5000")]
    pub provisioner_address: Endpoint,
}
