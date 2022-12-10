mod erc20_token;
mod pool_factory;

use crate::pool_factory::POOL_FACTORY_CREATION_BLOCK;
use anyhow::Context;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let node_url = env::var("NODE_URL")?;
    let transport =
        web3::transports::Http::new(&node_url).with_context(|| "http transport create")?;
    let web3 = web3::Web3::new(transport);

    let pool_factory = pool_factory::UniSwapPoolFactory::new(web3.clone())?;

    pool_factory.print_some_info().await?;

    let pool_creation_events = pool_factory
        .fetch_pool_creation_logs(
            POOL_FACTORY_CREATION_BLOCK.into(),
            Some((POOL_FACTORY_CREATION_BLOCK + 6000).into()),
        )
        .await?;

    for pool_creation_event in pool_creation_events {
        println!("{:?}", pool_creation_event)
    }

    Ok(())
}