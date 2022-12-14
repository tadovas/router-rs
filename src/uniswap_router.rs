use crate::pool::Descriptor;
use num_bigfloat::BigFloat;
use rust_decimal::Decimal;

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
