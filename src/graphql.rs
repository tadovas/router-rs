use crate::uniswap_router::Router;
use crate::{erc20_token, uniswap_router};
use actix_web::{
    web::{self, Data},
    Error, HttpResponse,
};
use anyhow::anyhow;
use juniper::{
    graphql_object, EmptyMutation, EmptySubscription, FieldResult, GraphQLInputObject,
    GraphQLObject, GraphQLScalarValue, RootNode,
};
use juniper_actix::{graphiql_handler, graphql_handler, playground_handler};
use std::sync::Arc;
use web3::types::Address;

#[derive(Clone, GraphQLObject)]
pub struct Token {
    pub address: ID,
    pub symbol: String,
    pub name: String,
}

impl From<&erc20_token::Token> for Token {
    fn from(v: &erc20_token::Token) -> Self {
        Self {
            address: v.address.into(),
            symbol: v.symbol.clone(),
            name: v.name.clone(),
        }
    }
}

#[derive(Clone, GraphQLObject)]
pub struct Pool {
    pub address: ID,
    pub token0: Token,
    pub token1: Token,
    pub price: String,
    pub fee: String,
}

impl From<&uniswap_router::Pool> for Pool {
    fn from(v: &uniswap_router::Pool) -> Self {
        Self {
            address: v.descriptor.address.into(),
            token0: (&v.descriptor.token0).into(),
            token1: (&v.descriptor.token1).into(),
            price: format!("{}", v.price),
            fee: format!("{}%", v.fee),
        }
    }
}

#[derive(Clone, GraphQLScalarValue)]
pub struct ID(String);

impl From<Address> for ID {
    fn from(address: Address) -> Self {
        ID(format!("{:?}", address)) // we need full address, default Display truncates address to few symbols
    }
}

#[derive(Clone, GraphQLInputObject)]
pub struct RouteInput {
    pub from_token: ID,
    pub to_token: ID,
}

#[derive(Clone, Default, GraphQLObject)]
pub struct Route {
    pub path: Vec<Token>,
}

pub struct QueryContext {
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

    fn tokens(context: &QueryContext) -> Vec<Token> {
        context
            .uniswap_router
            .get_available_tokens()
            .iter()
            .map(Token::from)
            .collect()
    }

    fn route(input: RouteInput, context: &QueryContext) -> FieldResult<Route> {
        let from: Address = input
            .from_token
            .0
            .parse()
            .map_err(|err| anyhow!("input.from_token: {}", err))?;
        let to: Address = input
            .to_token
            .0
            .parse()
            .map_err(|err| anyhow!("input.to_token: {}", err))?;
        Ok(context
            .uniswap_router
            .find_route(from, to)
            .map(|v| Route {
                path: v.iter().map(Token::from).collect(),
            })
            .map_err(|err| anyhow!("routing error: {}", err))?)
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
    schema: Data<Schema>,
) -> Result<HttpResponse, Error> {
    graphql_handler(&schema, &query_ctx, req, payload).await
}
