use clap::{Parser, Subcommand};

mod biz;
mod config;
mod data;
mod error;
mod network;
mod schema;
mod types;
mod utils;

use biz::Node;
use data::MemoryState;
use log::info;
use network::{http, p2p::P2pClient};
use tokio::task;
use wallet::Wallet;

use crate::{config::Config, network::p2p};

/// The command of tinychain
#[derive(Debug, Parser)]
struct Opts {
    #[command(subcommand)]
    subcmd: SubCommand,
}

#[derive(Debug, Subcommand)]
enum SubCommand {
    /// Create a new account for signing transactions
    NewAccount {
        /// the keystore directory, default is `./db/keystore/`
        #[arg(short, long, default_value_t = String::from("./db/keystore/"))]
        keystore_dir: String,
    },
    /// Create a random secret key for generating local peer id and keypair
    NewSecret,
    /// Run the node
    Run {
        /// the config file path, default is `config.toml`
        #[arg(short, long, default_value_t = String::from("config.toml"))]
        config: String,
    },
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let opts = Opts::parse();

    match opts.subcmd {
        SubCommand::NewAccount { keystore_dir } => new_account(&keystore_dir),
        SubCommand::NewSecret => new_secret_key(),
        SubCommand::Run { config } => run(&config).await,
    }
}

fn new_account(keystore_dir: &str) {
    let wallet = Wallet::new(keystore_dir);
    let acc = wallet.new_account().unwrap();
    info!("📣 New account: {:?}", acc);
}

fn new_secret_key() {
    let secret = p2p::new_secret_key();
    info!("📣 New secret key: {:?}", secret);
}

async fn run(config_file: &str) {
    let cfg = Config::load(config_file).unwrap();
    info!("📣 Config loaded: {:?}", cfg);
    let addr = cfg.http_addr.parse().unwrap();

    let (p2p_client, mut p2p_server) = p2p::new::<MemoryState>(cfg.p2p).unwrap();
    let node = Node::<MemoryState, P2pClient>::new(p2p_client);
    let event_handler = p2p::EventHandlerImpl::new(node.clone());
    p2p_server.set_event_handler(event_handler);

    task::spawn(p2p_server.run());
    http::run(addr, node).await;
}
