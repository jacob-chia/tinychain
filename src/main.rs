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
use wallet::Wallet;

use crate::config::Config;

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
        SubCommand::Run { config } => run(&config).await,
    }
}

fn new_account(keystore_dir: &str) {
    let wallet = Wallet::new(keystore_dir);
    let acc = wallet.new_account().unwrap();
    info!("📣 New account: {:?}", acc);
}

async fn run(config_file: &str) {
    let cfg = Config::load(config_file).unwrap();
    info!("📣 Config loaded: {:?}", cfg);

    let addr = cfg.http_addr.parse().unwrap();
    let node = Node::<MemoryState, P2pClient>::new();
    http::run(addr, node).await;
}
