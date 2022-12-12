use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use web3::contract::tokens::Detokenize;
use web3::contract::{Contract, Error, Options};
use web3::types::Address;
use web3::{Transport, Web3};

const CONTRACT_ABI: &[u8] = include_bytes!("abi/ERC20TokenAbi.json");
const CONTRACT_ABI_FALLBACK: &[u8] = include_bytes!("abi/ERC20StringFallback.json");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub name: String,
    pub symbol: String,
    pub address: Address,
    pub decimals: Option<u8>,
}

#[derive(Clone)]
pub struct Erc20TokenFinder<T: Transport> {
    web3: Web3<T>,
    contract: ethabi::Contract,
    fallback_contract: ethabi::Contract,
    erc20_addr_cache: Arc<RwLock<HashMap<Address, Token>>>,
}

impl<T: Transport> Erc20TokenFinder<T> {
    pub fn new(web3: Web3<T>) -> anyhow::Result<Self> {
        Ok(Self {
            web3,
            contract: ethabi::Contract::load(CONTRACT_ABI)?,
            fallback_contract: ethabi::Contract::load(CONTRACT_ABI_FALLBACK)?,
            erc20_addr_cache: Default::default(),
        })
    }

    pub async fn find(&self, addr: Address) -> anyhow::Result<Token> {
        if let Some(token) = self.erc20_addr_cache.read().await.get(&addr) {
            return Ok(token.clone());
        }

        let token = self
            .fetch_token(addr)
            .await
            .with_context(|| "fetch token")?;
        self.erc20_addr_cache
            .write()
            .await
            .insert(addr, token.clone());
        Ok(token)
    }

    async fn fetch_token(&self, addr: Address) -> anyhow::Result<Token> {
        let contract = Contract::new(self.web3.eth(), addr, self.contract.clone());
        let fallback_contract =
            Contract::new(self.web3.eth(), addr, self.fallback_contract.clone());
        let symbol = Self::try_fetch_readable(&contract, &fallback_contract, "symbol").await?;
        let name = Self::try_fetch_readable(&contract, &fallback_contract, "name").await?;
        let decimals: Option<u8> = contract
            .query("decimals", (), None, Options::default(), None)
            .await
            .ok();
        Ok(Token {
            name,
            address: addr,
            decimals,
            symbol,
        })
    }
    async fn try_fetch_readable(
        contract: &Contract<T>,
        fallback: &Contract<T>,
        name: &str,
    ) -> anyhow::Result<String> {
        let res_as_bytes32: Bytes32Result = contract
            .query(name, (), None, Options::default(), None)
            .await
            .with_context(|| format!("query: {}", name))?;
        if let Bytes32Result::Success(str) = res_as_bytes32 {
            return Ok(str);
        }
        fallback
            .query(name, (), None, Options::default(), None)
            .await
            .with_context(|| format!("fallback: {}", name))
    }
}

enum Bytes32Result {
    Success(String),
    RetryWithFallback,
}

impl Detokenize for Bytes32Result {
    fn from_tokens(tokens: Vec<ethabi::Token>) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let token = tokens
            .into_iter()
            .next()
            .ok_or_else(|| Error::InvalidOutputType("expected exactly one token".to_string()))?;
        match token {
            ethabi::Token::String(str) => return Ok(Bytes32Result::Success(str)),
            ethabi::Token::FixedBytes(bytes) => {
                let last_byte = bytes.get(31).ok_or_else(|| {
                    Error::InvalidOutputType("expected bytes32 token".to_string())
                })?;
                if *last_byte == 0x20 {
                    // a possible string
                    return Ok(Bytes32Result::RetryWithFallback);
                }
                // naive try to drop trailing zero bytes to make a normal string
                let filtered: Vec<u8> = bytes.into_iter().filter(|v| *v > 0).collect();
                return Ok(Bytes32Result::Success(
                    String::from_utf8_lossy(&filtered).to_string(),
                ));
            }
            _ => {}
        }

        Err(Error::InvalidOutputType(
            "expected bytes32 or string token".to_string(),
        ))
    }
}
