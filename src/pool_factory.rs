use anyhow::{anyhow, Context};

use ethabi::{Log, RawLog, Token};
use std::collections::HashMap;
use web3::contract::tokens::Tokenizable;
use web3::contract::Contract;
use web3::types::{Address, BlockNumber, FilterBuilder};
use web3::{Transport, Web3};

pub const POOL_FACTORY_CREATION_BLOCK: u64 = 12369621;
const UNISWAP_V3_POOL_FACTORY: &str = "0x1F98431c8aD98523631AE4a59f267346ea31F984";
const CONTRACT_ABI: &[u8] = include_bytes!("abi/UniswapV3PoolFactoryAbi.json");

pub struct UniSwapPoolFactory<T: Transport> {
    web3: Web3<T>,
    contract: Contract<T>,
}

impl<T: Transport> UniSwapPoolFactory<T> {
    pub fn new(web3: Web3<T>) -> anyhow::Result<Self> {
        let pool_factory_contract =
            Contract::from_json(web3.eth(), UNISWAP_V3_POOL_FACTORY.parse()?, CONTRACT_ABI)
                .with_context(|| "contract create")?;
        Ok(Self {
            web3,
            contract: pool_factory_contract,
        })
    }

    pub async fn fetch_pool_creation_logs(
        &self,
        from: BlockNumber,
        to: Option<BlockNumber>,
    ) -> anyhow::Result<Vec<PoolCreationEvent>> {
        let pool_created_event = self.contract.abi().event("PoolCreated")?;
        let mut filter = FilterBuilder::default()
            .topics(Some(vec![pool_created_event.signature()]), None, None, None)
            .from_block(from);

        if let Some(block_number) = to {
            filter = filter.to_block(block_number)
        }

        let logs = self
            .web3
            .eth()
            .logs(filter.build())
            .await
            .with_context(|| "eth raw logs")?;

        logs.into_iter()
            .map(move |l| {
                pool_created_event
                    .parse_log(RawLog {
                        topics: l.topics,
                        data: l.data.0,
                    })?
                    .try_into()
            })
            .collect::<anyhow::Result<Vec<PoolCreationEvent>>>()
    }
}

#[derive(Debug, Clone)]
pub struct PoolCreationEvent {
    pub token0_address: Address,
    pub token1_address: Address,
    pub pool_address: Address,
    pub fee: u32,
}

impl TryFrom<Log> for PoolCreationEvent {
    type Error = anyhow::Error;

    fn try_from(log: Log) -> Result<Self, Self::Error> {
        let mut param_map: HashMap<String, Token> = HashMap::new();
        for log_param in log.params {
            param_map.insert(log_param.name, log_param.value);
        }

        fn extract_parameter<R: Tokenizable>(
            param_map: &HashMap<String, Token>,
            name: &str,
        ) -> anyhow::Result<R> {
            param_map
                .get(name)
                .map(|t| R::from_token(t.clone()))
                .transpose()?
                .ok_or_else(|| anyhow!("missing parameter for: {}", name))
        }

        Ok(Self {
            token0_address: extract_parameter(&param_map, "token0")?,
            token1_address: extract_parameter(&param_map, "token1")?,
            pool_address: extract_parameter(&param_map, "pool")?,
            fee: extract_parameter(&param_map, "fee")?,
        })
    }
}
