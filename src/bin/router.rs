use clap::Parser;
use log::info;
use router_rs::env_log::setup_env_logger;
use router_rs::pool;
use rust_decimal::Decimal;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[derive(Parser, Debug)]
#[command(about, long_about = None)]
struct Args {
    /// url of web3 node to connect to
    #[arg(long)]
    node_url: String,

    /// input file name of collected pool descriptors
    #[arg(long)]
    descriptors_input: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_env_logger();
    let args = Args::parse();

    let mut json_file = File::open(args.descriptors_input).await?;
    let mut json_content = vec![];
    json_file.read_to_end(&mut json_content).await?;
    let pool_descriptors: pool::DescriptorList = serde_json::from_slice(&json_content)?;

    let transport = web3::transports::http::Http::new(&args.node_url)?;
    let web3 = web3::Web3::new(transport);

    for descriptor in pool_descriptors.descriptors {
        let pool = pool::Pool::new(web3.clone(), descriptor.clone())?;
        let slot0 = pool.slot0().await?;
        info!(
            "Pool {} -> {} [{}%] Slot0: {:?}",
            descriptor.token0.symbol,
            descriptor.token1.symbol,
            Decimal::new(descriptor.fee.into(), 4).normalize(),
            slot0
        )
    }

    Ok(())
}
