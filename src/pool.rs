use crate::erc20_token::Token;
use rust_decimal::prelude::One;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use web3::contract::{Contract, Options};
use web3::types::{Address, U256};
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
    pub price: String,
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
        let slot: (U256, i32, u16, u16, u16, u8, bool) = self
            .contract
            .query("slot0", (), None, Options::default(), None)
            .await?;

        Ok(Slot0 {
            price: convert_to_normal_price(slot.0, self.descriptor.token0.decimals.unwrap_or(0))
                .to_string(),
        })
    }
}

fn convert_to_normal_price(_sqrt_price_x96: U256, _token0_decimals: u8) -> Decimal {
    Decimal::one()
}

#[cfg(test)]
mod tests {
    use crate::pool::convert_to_normal_price;
    use rust_decimal_macros::dec;
    use web3::types::U256;

    #[test]
    fn test_sqrtx96_is_converted_correctly() {
        let sqrt_price_x96 =
            U256::from_dec_str("132913141809576967649153816958").expect("should not fail");

        let price = convert_to_normal_price(sqrt_price_x96, 18);
        assert_eq!(dec!(1.28143), price);
    }
}
