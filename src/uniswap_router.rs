use crate::erc20_token::Token;
use crate::pool::Descriptor;
use itertools::Itertools;
use num_bigfloat::BigFloat;
use petgraph::{Directed, Graph};
use rust_decimal::Decimal;
use web3::types::Address;

// Pool represents uniswap pool information needed by router to do all required logic
// as listing available pools or creating routes for given start and end token
#[derive(Debug, Clone)]
pub struct Pool {
    pub descriptor: Descriptor,
    pub price: BigFloat,
    pub fee: Decimal,
}

pub struct Router {
    pools: Vec<Pool>,
    routing_graph: Graph<Token, Pool, Directed>,
}

impl Router {
    pub fn new(pools: Vec<Pool>) -> Self {
        let routing_graph = build_routing_graph(&pools);
        Self {
            pools,
            routing_graph,
        }
    }

    pub fn get_available_pools(&self) -> &[Pool] {
        &self.pools
    }

    pub fn get_available_tokens(&self) -> Vec<Token> {
        self.pools
            .iter()
            .flat_map(|pool| {
                [
                    pool.descriptor.token0.clone(),
                    pool.descriptor.token1.clone(),
                ]
            })
            .unique_by(|t| t.address)
            .collect()
    }

    pub fn find_route(&self, _from: Address, _to: Address) -> anyhow::Result<Vec<Token>> {
        Ok(self.get_available_tokens().into_iter().take(5).collect())
    }
}

fn build_routing_graph(pools: &[Pool]) -> Graph<Token, Pool, Directed> {
    let mut graph = Graph::with_capacity(pools.len() * 2, pools.len());
    for pool in pools {
        let token0_idx = graph.add_node(pool.descriptor.token0.clone());
        let token1_idx = graph.add_node(pool.descriptor.token1.clone());
        graph.update_edge(token0_idx, token1_idx, pool.clone());
    }
    graph
}
