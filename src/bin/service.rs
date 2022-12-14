use actix_web::web::Data;
use actix_web::{middleware, web, App, HttpServer};
use anyhow::Context;
use clap::Parser;
use futures::future::join_all;
use num_bigfloat::BigFloat;
use num_traits::Float;
use router_rs::env_log::init_with_default_level;
use router_rs::graphql::{graphiql_route, graphql_route, playground_route, schema, QueryContext};
use router_rs::pool;
use router_rs::pool::Descriptor;
use router_rs::uniswap_router::{Pool, Router};
use rust_decimal::Decimal;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use web3::transports::Http;
use web3::{Transport, Web3};

#[derive(Parser, Debug)]
#[command(about, long_about = None)]
struct Args {
    /// url of web3 node to connect to
    #[arg(long)]
    node_url: String,

    /// input file name of collected pool descriptors
    #[arg(long)]
    descriptors_input: String,

    /// http service listening port
    #[arg(long, default_value_t = 8080)]
    http_port: u16,
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    init_with_default_level();

    let args = Args::parse();

    let mut json_file = File::open(args.descriptors_input).await?;
    let mut json_content = vec![];
    json_file.read_to_end(&mut json_content).await?;
    let pool_descriptors: pool::DescriptorList = serde_json::from_slice(&json_content)?;

    let transport = Http::new(&args.node_url)?;
    let web3 = Web3::new(transport);

    let pool_futures = pool_descriptors
        .descriptors
        .into_iter()
        .map(|v| map_into_pool(v, &web3));

    let pools = join_all(pool_futures)
        .await
        .into_iter()
        .collect::<anyhow::Result<Vec<Pool>>>()?;

    let uniswap_router = Arc::new(Router::new(pools));
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(QueryContext {
                schema: schema(),
                uniswap_router: uniswap_router.clone(),
            }))
            .wrap(middleware::Logger::default())
            .service(
                web::resource("/graphql")
                    .route(web::post().to(graphql_route))
                    .route(web::get().to(graphql_route)),
            )
            .service(web::resource("/playground").route(web::get().to(playground_route)))
            .service(web::resource("/graphiql").route(web::get().to(graphiql_route)))
    })
    .bind(("127.0.0.1", args.http_port))?
    .run()
    .await
    .with_context(|| "http service error")
}

async fn map_into_pool<T: Transport>(
    descriptor: Descriptor,
    web3: &Web3<T>,
) -> anyhow::Result<Pool> {
    let pool_contract = pool::PoolContract::new(web3.clone(), descriptor.clone())?;
    let slot0 = pool_contract.slot0().await?;

    Ok(Pool {
        descriptor: descriptor.clone(),
        price: slot0.price.unwrap_or_else(BigFloat::nan),
        fee: Decimal::new(descriptor.fee.into(), 4).normalize(),
    })
}
