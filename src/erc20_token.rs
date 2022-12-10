use std::cell::RefCell;
use std::collections::HashMap;
use web3::contract::{Contract, Options};
use web3::types::Address;
use web3::{Transport, Web3};

const CONTRACT_ABI: &[u8] = include_bytes!("abi/ERC20TokenAbi.json");

#[derive(Debug, Clone)]
pub struct Token {
    pub name: String,
    pub symbol: String,
    pub address: Address,
}

pub struct Erc20TokenFinder<T: Transport> {
    web3: Web3<T>,
    contract: ethabi::Contract,
    erc20_addr_cache: RefCell<HashMap<Address, Token>>,
}

impl<T: Transport> Erc20TokenFinder<T> {
    pub fn new(web3: Web3<T>) -> anyhow::Result<Self> {
        Ok(Self {
            web3,
            contract: ethabi::Contract::load(CONTRACT_ABI)?,
            erc20_addr_cache: Default::default(),
        })
    }

    pub async fn find(&self, addr: Address) -> anyhow::Result<Token> {
        if let Some(token) = self.erc20_addr_cache.borrow().get(&addr) {
            return Ok(token.clone());
        }

        let token = self.fetch_token(addr).await?;
        self.erc20_addr_cache
            .borrow_mut()
            .insert(addr, token.clone());
        Ok(token)
    }

    async fn fetch_token(&self, addr: Address) -> anyhow::Result<Token> {
        let contract = Contract::new(self.web3.eth(), addr, self.contract.clone());
        let symbol: String = contract
            .query("symbol", (), None, Options::default(), None)
            .await?;
        let name: String = contract
            .query("name", (), None, Options::default(), None)
            .await?;
        Ok(Token {
            name,
            symbol,
            address: addr,
        })
    }
}
