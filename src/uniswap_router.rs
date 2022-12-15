use crate::erc20_token::Token;
use crate::pool::Descriptor;
use anyhow::anyhow;
use itertools::Itertools;
use num_bigfloat::BigFloat;
use petgraph::algo::dijkstra;
use petgraph::graph::{DefaultIx, NodeIndex};
use petgraph::{Directed, Graph};
use rust_decimal::Decimal;
use std::collections::HashMap;
use web3::types::Address;

// Pool represents uniswap pool information needed by router to do all required logic
// as listing available pools or creating routes for given start and end token
#[derive(Debug, Clone)]
pub struct Pool {
    pub descriptor: Descriptor,
    pub price: BigFloat,
    pub fee: Decimal,
}

type NodeMap = HashMap<Address, NodeIndex<DefaultIx>>;

pub struct Router {
    pools: Vec<Pool>,
    routing_graph: Graph<Token, Pool, Directed>,
    node_map: NodeMap,
}

impl Router {
    pub fn new(pools: Vec<Pool>) -> Self {
        let (node_map, routing_graph) = build_routing_graph(&pools);
        Self {
            pools,
            routing_graph,
            node_map,
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
        let start_idx = *self
            .node_map
            .get(&_from)
            .ok_or_else(|| anyhow!("start token not found"))?;
        let end_idx = *self
            .node_map
            .get(&_to)
            .ok_or_else(|| anyhow!("end address not found"))?;
        let path = dijkstra(
            &self.routing_graph,
            start_idx,
            Some(end_idx),
            |_| 1, // TODO for now just check if path exists at all
        );
        Ok(path
            .keys()
            .into_iter()
            .filter_map(|idx| self.routing_graph.node_weight(*idx))
            .map(Clone::clone)
            .collect())
    }
}

fn build_routing_graph(pools: &[Pool]) -> (NodeMap, Graph<Token, Pool, Directed>) {
    let mut node_map = HashMap::new();
    let mut graph = Graph::with_capacity(pools.len() * 2, pools.len());
    for pool in pools {
        let token0 = pool.descriptor.token0.clone();
        let token0_idx = *node_map
            .entry(token0.address)
            .or_insert_with(|| graph.add_node(token0));

        let token1 = pool.descriptor.token1.clone();
        let token1_idx = *node_map
            .entry(token1.address)
            .or_insert_with(|| graph.add_node(token1));

        graph.update_edge(token0_idx, token1_idx, pool.clone());
        graph.update_edge(token1_idx, token0_idx, pool.clone());
    }
    (node_map, graph)
}

#[cfg(test)]
mod tests {
    use crate::erc20_token::Token;
    use crate::pool::Descriptor;
    use crate::uniswap_router::{Pool, Router};
    use web3::types::Address;

    #[test]
    fn test_path_doesnt_exists_between_unrelated_tokens() {
        let token_a = Token {
            name: "A".to_string(),
            symbol: "A".to_string(),
            address: Address::from_low_u64_le(1),
            decimals: None,
        };
        let token_b = Token {
            name: "B".to_string(),
            symbol: "B".to_string(),
            address: Address::from_low_u64_le(2),
            decimals: None,
        };

        let token_c = Token {
            name: "C".to_string(),
            symbol: "C".to_string(),
            address: Address::from_low_u64_le(3),
            decimals: None,
        };
        let token_d = Token {
            name: "D".to_string(),
            symbol: "D".to_string(),
            address: Address::from_low_u64_le(4),
            decimals: None,
        };

        let pools = vec![
            Pool {
                descriptor: Descriptor {
                    token0: token_a.clone(),
                    token1: token_b.clone(),
                    fee: 0,
                    address: Default::default(),
                },
                price: Default::default(),
                fee: Default::default(),
            },
            Pool {
                descriptor: Descriptor {
                    token0: token_c.clone(),
                    token1: token_d.clone(),
                    fee: 0,
                    address: Default::default(),
                },
                price: Default::default(),
                fee: Default::default(),
            },
        ];

        let router = Router::new(pools);
        let hops = router
            .find_route(token_b.address, token_c.address)
            .expect("should not fail");

        assert_eq!(hops.len(), 0)
    }
}
