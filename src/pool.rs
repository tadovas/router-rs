use crate::erc20_token::Token;
use lazy_static::lazy_static;
use num_traits::{Pow, Zero};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::ops::{Div, Mul};
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
            price: convert_to_normal_price(
                slot.0,
                self.descriptor.token0.decimals.unwrap_or(0),
                self.descriptor.token1.decimals.unwrap_or(0),
            )
            .to_string(),
        })
    }
}

lazy_static! {
    static ref X96: U256 = U256::from(2).pow(96.into());
    static ref X192: U256 = X96.pow(2.into());
}

fn convert_to_normal_price(
    sqrt_price_x96: U256,
    token0_decimals: u8,
    token1_decimals: u8,
) -> Decimal {
    if sqrt_price_x96.is_zero() {
        return Decimal::zero();
    }
    // price is sqrt(token1/token0) Q64.96
    // price = sqrtRatioX96 ** 2 / 2 ** 192
    // we want token0/token1 price so its inversed
    let calculated = X192.div(sqrt_price_x96.pow(2.into()));
    // assuming we got token0/token1 price we now need to apply decimals to display it to something readable
    let diff: i64 = token1_decimals as i64 - token0_decimals as i64;
    let token_decimals_ratio: Decimal = Decimal::from(10).pow(diff);
    Decimal::from_str_exact(&calculated.to_string())
        .expect("too big number")
        .mul(token_decimals_ratio)
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

        let price = convert_to_normal_price(sqrt_price_x96, 18, 18);
        assert_eq!(dec!(0), price);
    }
}
