use web3::contract::{Contract, Options};
use web3::types::Address;
use web3::{Transport, Web3};

const POOL_ABI_BYTES: &[u8] = include_bytes!("abi/UniswapV3PoolAbi.json");

#[derive(Debug)]
pub struct Descriptor {
    pub token0: Address,
    pub token1: Address,
    pub fee: u32,
}

pub struct Pool<T: Transport> {
    contract: Contract<T>,
}

impl<T: Transport> Pool<T> {
    pub fn new(web3: Web3<T>, address: Address) -> anyhow::Result<Self> {
        let contract = Contract::from_json(web3.eth(), address, POOL_ABI_BYTES)?;
        Ok(Self { contract })
    }

    pub async fn descriptor(&self) -> anyhow::Result<Descriptor> {
        let token0: Address = self
            .contract
            .query("token0", (), None, Options::default(), None)
            .await?;
        let token1: Address = self
            .contract
            .query("token1", (), None, Options::default(), None)
            .await?;
        let fee: u32 = self
            .contract
            .query("fee", (), None, Options::default(), None)
            .await?;

        Ok(Descriptor {
            token0,
            token1,
            fee,
        })
    }
}
