use std::thread;

use clap::{Parser, Subcommand};
use config::Config;
use crossbeam_channel::unbounded;
use log::info;

use tokio::task;
use wallet::{self, Wallet};

mod biz;
mod config;
mod data;
mod error;
mod network;
mod schema;
mod types;
mod utils;

use biz::{Genesis, Node};
use data::SledState;
use network::{http, p2p};

const MINING_DIFFICULTY: usize = 2;

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
    info!("📣 Saved in: {:?}", keystore_dir);
}

fn new_secret_key() {
    let secret = p2p::new_secret_key();
    info!("📣 New secret key: {:?}", secret);
}

async fn run(config_file: &str) {
    // Load config.
    let Config {
        data_dir,
        genesis_file,
        http_addr,
        author,
        p2p: p2p_config,
        wallet,
    } = Config::load(config_file).unwrap();
    let http_addr = http_addr.parse().unwrap();
    let genesis = Genesis::load(&genesis_file).unwrap();
    info!("📣 Genesis: {:?}", genesis);

    // When receiving a new block from other peers, a signal will be sent to the miner to stop mining.
    let (cancel_signal_s, cancel_signal_r) = unbounded();

    // Create a new node.
    let wallet = Wallet::new(&wallet.keystore_dir);
    let sled_state = SledState::new(&data_dir, genesis.into_balances(), MINING_DIFFICULTY).unwrap();
    let (p2p_client, mut p2p_server) = p2p::new(p2p_config).unwrap();
    let node = Node::new(
        author,
        sled_state,
        p2p_client,
        wallet,
        cancel_signal_s,
        MINING_DIFFICULTY,
    )
    .unwrap();
    p2p_server.set_event_handler(node.clone());

    let miner = node.clone();
    let syncer = node.clone();

    task::spawn(p2p_server.run());
    task::spawn(http::run(http_addr, node));
    thread::spawn(move || syncer.sync());
    miner.mine(cancel_signal_r)
}
