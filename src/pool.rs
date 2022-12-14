use crate::erc20_token::Token;
use anyhow::anyhow;
use num_bigfloat::BigFloat;
use num_traits::Pow;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use web3::contract::{Contract, Options};
use web3::types::{Address, U256};
use web3::{Transport, Web3};

const POOL_ABI_BYTES: &[u8] = include_bytes!("abi/UniswapV3PoolAbi.json");

// Root object for serde to/from JSON
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DescriptorList {
    pub descriptors: Vec<Descriptor>,
}

// Uniswap V3 Pool descriptor with basic information
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Descriptor {
    pub token0: Token,
    pub token1: Token,
    pub fee: u32,
    pub address: Address,
}

// Slot0 query result (with minimal required fields)
#[derive(Debug, Clone)]
pub struct Slot0 {
    pub price: Option<BigFloat>,
}

// Pool contract is used to interact with deployed contracts
pub struct PoolContract<T: Transport> {
    contract: Contract<T>,
    descriptor: Descriptor,
}

impl<T: Transport> PoolContract<T> {
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
            )?,
        })
    }
}

// converts to "normal" price, given the sqrtPriceX96 value (which can be 0 if contract is not initialized)
fn convert_to_normal_price(
    sqrt_price_x96: U256,
    token0_decimals: u8,
    token1_decimals: u8,
) -> anyhow::Result<Option<BigFloat>> {
    if sqrt_price_x96.is_zero() {
        // non-initialized contract
        return Ok(None);
    }
    // price is sqrt(token1/token0) Q64.96
    // price = sqrtRatioX96 ** 2 / 2 ** 192
    let x192: BigFloat = BigFloat::from(2).pow(192.into());
    let calculated = BigFloat::from_str(&sqrt_price_x96.to_string())
        .map_err(|e| anyhow!("U256 -> BigFloat: {}", e))?
        .pow(2.into())
        / x192;
    // assuming we got token0/token1 price we now need to apply decimals to display it to something readable
    let diff: i64 = token0_decimals as i64 - token1_decimals as i64;
    let token_decimals_ratio = BigFloat::from(10).pow(diff.into());
    let price = calculated * token_decimals_ratio;
    Ok(Some(price))
}

#[cfg(test)]
mod tests {
    use crate::pool::convert_to_normal_price;
    use web3::types::U256;

    #[test]
    fn test_sqrtx96_is_converted_correctly() {
        let sqrt_price_x96 =
            U256::from_dec_str("2889297909017548779246569").expect("should not fail");

        let price = convert_to_normal_price(sqrt_price_x96, 18, 6)
            .expect("not error")
            .expect("not None");
        assert_eq!(
            "1.329919883246710696663783607187959335420e+3",
            price.to_string()
        );
    }
}
