use crate::erc20_token::Token;
use serde::{Deserialize, Serialize};
use web3::contract::Contract;
use web3::types::Address;
use web3::{Transport, Web3};

const POOL_ABI_BYTES: &[u8] = include_bytes!("abi/UniswapV3PoolAbi.json");

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DescriptorList {
    pub descriptors: Vec<Descriptor>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Descriptor {
    pub token0: Token,
    pub token1: Token,
    pub fee: u32,
    pub address: Address,
}

#[derive(Debug, Clone)]
pub struct Slot0 {
    pub info: String,
}

pub struct Pool<T: Transport> {
    contract: Contract<T>,
    descriptor: Descriptor,
}

impl<T: Transport> Pool<T> {
    pub fn new(web3: Web3<T>, descriptor: Descriptor) -> anyhow::Result<Self> {
        let contract = Contract::from_json(web3.eth(), descriptor.address, POOL_ABI_BYTES)?;
        Ok(Self {
            contract,
            descriptor,
        })
    }

    pub async fn slot0(&self) -> anyhow::Result<Slot0> {
        Ok(Slot0 {
            info: "some info".to_string(),
        })
    }
}
