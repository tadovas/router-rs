use anyhow::Context;
use router_rs::erc20_token::{Erc20TokenFinder, Token};
use router_rs::pool;
use router_rs::pool_factory;
use router_rs::pool_factory::PoolCreationEvent;
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::task;

use web3::types::Address;
use web3::{Transport, Web3};

#[derive(Serialize, Deserialize)]
struct PoolDescriptors {
    contracts: Vec<PoolDescriptor>,
}

#[derive(Serialize, Deserialize)]
struct PoolDescriptor {
    token0: Token,
    token1: Token,
    fee: u32,
    address: Address,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let node_url = env::var("NODE_URL")?;
    let transport =
        web3::transports::Http::new(&node_url).with_context(|| "http transport create")?;
    let web3 = Web3::new(transport);

    let pool_factory = pool_factory::UniSwapPoolFactory::new(web3.clone())?;
    let pool_creation_events = pool_factory
        .fetch_pool_creation_logs(
            pool_factory::POOL_FACTORY_CREATION_BLOCK.into(),
            Some((pool_factory::POOL_FACTORY_CREATION_BLOCK + 6000).into()),
        )
        .await?;

    let token_lookup = Erc20TokenFinder::new(web3.clone())?;
    println!("{} pool creation events total", pool_creation_events.len());
    let mut pool_descriptors = Vec::with_capacity(pool_creation_events.len());

    let mut task_set = task::JoinSet::new();
    let task_limiter = Arc::new(tokio::sync::Semaphore::new(10));

    for pool_creation_event in pool_creation_events {
        let web3 = web3.clone();
        let token_lookup = token_lookup.clone();
        let permit = task_limiter
            .clone()
            .acquire_owned()
            .await
            .with_context(|| "semaphore permit aquire")?;
        task_set.spawn(async move {
            let res = build_pool_descriptor(web3, pool_creation_event, token_lookup).await;
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

    let mut output = File::create("pool_descriptors.json").await?;
    let json = serde_json::to_string_pretty(&PoolDescriptors {
        contracts: pool_descriptors,
    })?;

    output.write_all(json.as_bytes()).await?;

    Ok(())
}

async fn build_pool_descriptor<T: Transport>(
    web3: Web3<T>,
    pool_creation_event: PoolCreationEvent,
    token_lookup: Erc20TokenFinder<T>,
) -> anyhow::Result<PoolDescriptor> {
    let pool_descriptor = pool::Pool::new(web3, pool_creation_event.pool_address)
        .with_context(|| format!("pool {} creation", pool_creation_event.pool_address))?
        .descriptor()
        .await
        .with_context(|| format!("pool {} descriptor fetch", pool_creation_event.pool_address))?;

    Ok(PoolDescriptor {
        address: pool_creation_event.pool_address,
        token0: token_lookup
            .find(pool_descriptor.token0)
            .await
            .with_context(|| format!("token: {} lookup", pool_descriptor.token0))?,
        token1: token_lookup
            .find(pool_descriptor.token1)
            .await
            .with_context(|| format!("token: {} lookup", pool_descriptor.token1))?,
        fee: pool_descriptor.fee,
    })
}
