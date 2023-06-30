mod biz;
mod data;
mod error;
mod network;
mod schema;
mod types;
mod utils;

use biz::Node;
use data::MemoryState;
use network::{http, p2p::P2pClient};

#[tokio::main]
async fn main() {
    let addr = "0.0.0.0:8000".parse().unwrap();
    let node = Node::<MemoryState, P2pClient>::new();
    http::run(addr, node).await;
}
