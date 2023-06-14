use std::{sync::Arc, thread};

use clap::{Parser, Subcommand};
use config::Config;
use crossbeam_channel::unbounded;
use log::info;

use tokio::task;
use wallet;

mod config;
mod error;
mod network;
mod node;
mod state;
mod types;
mod utils;

use network::{http, p2p};
use node::Node;
use state::FileState;

const MINING_DIFFICULTY: usize = 2;

/// The command of tinychain
#[derive(Debug, Parser)]
struct Opts {
    #[command(subcommand)]
    subcmd: SubCommand,
}

#[derive(Debug, Subcommand)]
enum SubCommand {
    /// Create a new account with a new set of a elliptic-curve keypair
    NewAccount {
        /// the node data dir where the account will be stored
        #[arg(short, long, default_value_t = String::from("./db/"))]
        datadir: String,
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
        SubCommand::NewAccount { datadir } => new_account(&datadir),
        SubCommand::NewSecret => new_secret_key(),
        SubCommand::Run { config } => run(&config).await,
    }
}

fn new_account(datadir: &str) {
    wallet::init_keystore_dir(datadir);
    let acc = wallet::new_account().unwrap();
    info!("📣 New account: {:?}", acc);
    info!("📣 Saved in: {:?}", wallet::get_keystore_dir());
}

fn new_secret_key() {
    let secret = p2p::new_secret_key();
    info!("📣 New secret key: {:?}", secret);
}

async fn run(config_file: &str) {
    let Config {
        datadir,
        http_addr,
        miner,
        p2p: p2p_config,
    } = Config::load(config_file).expect("Failed to load config file");
    let http_addr = http_addr.parse().expect("Invalid http address");

    wallet::init_keystore_dir(&datadir);
    state::init_database_dir(&datadir);

    // When receiving a new block from other peers, a signal will be sent to the miner to stop mining.
    let (cancel_signal_s, cancel_signal_r) = unbounded();
    let file_state = FileState::new(MINING_DIFFICULTY).unwrap();
    let (p2p_client, event_loop, p2p_server) = p2p::new(p2p_config).unwrap();

    // Create a new node with `FileState` and `P2pClient`.
    let node = Arc::new(Node::new(miner, file_state, p2p_client, cancel_signal_s.clone()).unwrap());
    let miner = node.clone();
    let syncer = node.clone();

    task::spawn(p2p_server.run());
    task::spawn(event_loop.run(node.clone()));
    task::spawn(http::run(http_addr, node.clone()));

    thread::spawn(move || syncer.sync());
    miner.mine(cancel_signal_r)
}
