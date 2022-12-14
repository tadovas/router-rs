## Uniswap V3 router
Fetches all deployed v3 uniswap pools and builds routing service base on GraphQL (actual routing TODO)
### Pool list builder
```
cargo run --bin build_pool_list -- --help
   Compiling router-rs v0.1.0
    Finished dev [unoptimized + debuginfo] target(s) in 6.01s
     Running `target\debug\build_pool_list --help`
Usage: build_pool_list [OPTIONS] --node-url <NODE_URL> --descriptors-output <DESCRIPTORS_OUTPUT>

Options:
--node-url <NODE_URL>
url of web3 node to connect to
--descriptors-output <DESCRIPTORS_OUTPUT>
output file name of collected pool descriptors
--to-block-num <TO_BLOCK_NUM>
upper block limit to scan events to (default - latest)
-m, --max-parallel-pool-processing <MAX_PARALLEL_POOL_PROCESSING>
max number of parallel processing of fetched pool creation events [default: 10]
-h, --help
Print help information
```
Connects to given web3 node and pulls all available pools for uniswap v3 project.
Created JSON file then is used to start routing service itself. Highly recommended to use upper block limit and adjust max parallel token fetching tasks, since it quickly kills underlying http client.

TODO:
* Current address -> ERC20 token cache is in-memory only and is created dropped on each run. A better approach would be to have similar JSON file which can be populated on first run and reused later.

### GraphQL routing service
```
cargo run --bin service -- --help
   Compiling router-rs v0.1.0
    Finished dev [unoptimized + debuginfo] target(s) in 4.24s
     Running `target\debug\service --help`
Usage: service [OPTIONS] --node-url <NODE_URL> --descriptors-input <DESCRIPTORS_INPUT>

Options:
      --node-url <NODE_URL>                    url of web3 node to connect to
      --descriptors-input <DESCRIPTORS_INPUT>  input file name of collected pool descriptors
      --http-port <HTTP_PORT>                  http service listening port [default: 8080]
  -h, --help                                   Print help information
```
Starts GraphQL API based routing service which takes pre-built pool descriptor list, additionally fetches latest pool prices and serves GraphQL API on given http port.
For simplified access playground is available at http://localhost:8080/playground. Currently supported query - list all available Pools for swapping with prices.

TODO:
* No real token to token swap routing :(. Although all required building blocks are present (pool list with token info etc.). It would require a deeper understanding what kind of graph needs to be used, node selection etc. etc., since simple Dijkstra won't work in this case.
* Price of pool is collected on startup only. Need to subscribe to `swap` events probably for each pool to get price updates (or any other events)
* All pools are currently fetched and displayed. Some of them are not initialized yet, some of them are not initialized yet, so another event subscription `pool initialized` is required to enable them to use for real routing 