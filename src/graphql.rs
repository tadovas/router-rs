use crate::uniswap_router::Router;
use crate::{erc20_token, uniswap_router};
use actix_web::{
    web::{self, Data},
    Error, HttpResponse,
};
use juniper::{graphql_object, EmptyMutation, EmptySubscription, GraphQLObject, RootNode};
use juniper_actix::{graphiql_handler, graphql_handler, playground_handler};
use std::sync::Arc;

#[derive(Clone, GraphQLObject)]
pub struct Token {
    pub address: String,
    pub symbol: String,
    pub name: String,
}

impl From<&erc20_token::Token> for Token {
    fn from(v: &erc20_token::Token) -> Self {
        Self {
            address: format!("{:?}", v.address),
            symbol: v.symbol.clone(),
            name: v.name.clone(),
        }
    }
}

#[derive(Clone, GraphQLObject)]
pub struct Pool {
    pub address: String,
    pub token0: Token,
    pub token1: Token,
    pub price: String,
    pub fee: String,
}

impl From<&uniswap_router::Pool> for Pool {
    fn from(v: &uniswap_router::Pool) -> Self {
        Self {
            address: format!("{:?}", v.descriptor.address),
            token0: (&v.descriptor.token0).into(),
            token1: (&v.descriptor.token1).into(),
            price: format!("{}", v.price),
            fee: format!("{}%", v.fee),
        }
    }
}

pub struct QueryContext {
    pub schema: Schema,
    pub uniswap_router: Arc<Router>,
}

impl juniper::Context for QueryContext {}

pub struct Query;
#[graphql_object(context = QueryContext)]
impl Query {
    fn pools(context: &QueryContext) -> Vec<Pool> {
        context
            .uniswap_router
            .get_available_pools()
            .iter()
            .map(Pool::from)
            .collect()
    }
}

pub type Schema =
    RootNode<'static, Query, EmptyMutation<QueryContext>, EmptySubscription<QueryContext>>;

pub fn schema() -> Schema {
    Schema::new(
        Query,
        EmptyMutation::<QueryContext>::new(),
        EmptySubscription::<QueryContext>::new(),
    )
}

pub async fn graphiql_route() -> Result<HttpResponse, Error> {
    graphiql_handler("/graphql", None).await
}
pub async fn playground_route() -> Result<HttpResponse, Error> {
    playground_handler("/graphql", None).await
}

pub async fn graphql_route(
    req: actix_web::HttpRequest,
    payload: web::Payload,
    query_ctx: Data<QueryContext>,
) -> Result<HttpResponse, Error> {
    graphql_handler(&query_ctx.schema, &query_ctx, req, payload).await
}
