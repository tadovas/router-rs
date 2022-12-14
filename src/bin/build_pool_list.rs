use anyhow::Context;
use clap::Parser;
use log::info;
use router_rs::env_log::setup_env_logger;
use router_rs::erc20_token::Erc20TokenFinder;
use router_rs::pool::{Descriptor, DescriptorList};
use router_rs::pool_factory;
use router_rs::pool_factory::PoolCreationEvent;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::task;
use web3::types::BlockNumber;
use web3::{Transport, Web3};

#[derive(Parser, Debug)]
#[command(about, long_about = None)]
struct Args {
    /// url of web3 node to connect to
    #[arg(long)]
    node_url: String,

    /// output file name of collected pool descriptors
    #[arg(long)]
    descriptors_output: String,

    /// upper block limit to scan events to (default - latest)
    #[arg(long)]
    to_block_num: Option<u64>,

    /// max number of parallel processing of fetched pool creation events
    #[arg(long, short, default_value_t = 10)]
    max_parallel_pool_processing: usize,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_env_logger();
    let args = Args::parse();

    let transport =
        web3::transports::Http::new(&args.node_url).with_context(|| "http transport create")?;
    let web3 = Web3::new(transport);

    let pool_factory = pool_factory::UniSwapPoolFactory::new(web3.clone())?;
    let pool_creation_events = pool_factory
        .fetch_pool_creation_logs(
            pool_factory::POOL_FACTORY_CREATION_BLOCK.into(),
            args.to_block_num
                .map(|v| BlockNumber::Number(v.into()))
                .unwrap_or(BlockNumber::Latest),
        )
        .await?;

    let token_lookup = Erc20TokenFinder::new(web3.clone())?;
    info!("{} pool creation events total", pool_creation_events.len());
    let mut pool_descriptors = Vec::with_capacity(pool_creation_events.len());

    let mut task_set = task::JoinSet::new();
    // without task limiter, underlying http client fails with weird errors
    let task_limiter = Arc::new(tokio::sync::Semaphore::new(
        args.max_parallel_pool_processing,
    ));

    for pool_creation_event in pool_creation_events {
        let token_lookup = token_lookup.clone();
        let permit = task_limiter
            .clone()
            .acquire_owned()
            .await
            .with_context(|| "semaphore permit aquire")?;
        task_set.spawn(async move {
            let res = build_pool_descriptor(pool_creation_event, token_lookup).await;
            drop(permit);
            res
        });
    }

    while let Some(task_result) = task_set.join_next().await {
        let descriptor = task_result
            .with_context(|| "task join")?
            .with_context(|| "task result")?;
        pool_descriptors.push(descriptor)
    }

    let mut output = File::create(&args.descriptors_output).await?;
    let json = serde_json::to_string_pretty(&DescriptorList {
        descriptors: pool_descriptors,
    })?;

    output.write_all(json.as_bytes()).await?;
    info!("Pool descriptors written to {}", args.descriptors_output);
    Ok(())
}

async fn build_pool_descriptor<T: Transport>(
    pool_creation_event: PoolCreationEvent,
    token_lookup: Erc20TokenFinder<T>,
) -> anyhow::Result<Descriptor> {
    Ok(Descriptor {
        address: pool_creation_event.pool_address,
        token0: token_lookup
            .find(pool_creation_event.token0_address)
            .await
            .with_context(|| format!("token: {} lookup", pool_creation_event.token0_address))?,
        token1: token_lookup
            .find(pool_creation_event.token1_address)
            .await
            .with_context(|| format!("token: {} lookup", pool_creation_event.token1_address))?,
        fee: pool_creation_event.fee,
    })
}
