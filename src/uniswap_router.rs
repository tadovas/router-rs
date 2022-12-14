use crate::pool::Descriptor;
use num_bigfloat::BigFloat;
use rust_decimal::Decimal;

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
}

impl Router {
    pub fn new(pools: Vec<Pool>) -> Self {
        Self { pools }
    }

    pub fn get_available_pools(&self) -> &[Pool] {
        &self.pools
    }
}
