use std::{sync::Arc, thread};

use clap::{Parser, Subcommand};
use crossbeam_channel::bounded;
use log::info;

mod data;
mod error;
mod node;
mod server;
mod types;
mod utils;
mod wallet;

use data::{FileState, HttpPeer};
use node::Node;

const MINING_DIFFICULTY: usize = 2;

/// The command of tiny-chain
#[derive(Debug, Parser)]
struct Opts {
    #[command(subcommand)]
    subcmd: SubCommand,
}

#[derive(Debug, Subcommand)]
enum SubCommand {
    /// Creates a new account with a new set of a elliptic-curve Private + Public keys
    NewAccount {
        /// the node data dir where the account will be stored
        #[arg(short, long, default_value_t = String::from("./db/"))]
        datadir: String,
    },
    /// Launches the node
    Run {
        /// the node data dir where the DB will/is stored
        #[arg(short, long, default_value_t = String::from("./db/"))]
        datadir: String,
        /// the exposed address for communication with peers
        #[arg(short, long, default_value_t = String::from("127.0.0.1:8000"))]
        addr: String,
        /// the miner account of this node to receive block rewards
        #[arg(short, long)]
        miner: String,
        /// the bootstraping node that provides initial information to newly joining nodes
        #[arg(short, long)]
        bootstrap_addr: Option<String>,
    },
}

fn main() {
    env_logger::init();
    let opts = Opts::parse();

    match opts.subcmd {
        SubCommand::NewAccount { datadir } => {
            wallet::init_keystore_dir(&datadir);
            let acc = wallet::new_account().unwrap();
            info!("New account created: {:?}", acc);
            info!("Saved in: {}", wallet::get_keystore_dir());
        }
        SubCommand::Run {
            datadir,
            addr,
            miner,
            bootstrap_addr,
        } => {
            wallet::init_keystore_dir(&datadir);
            data::init_database_dir(&datadir);

            let node = new_arc_node(addr, miner, bootstrap_addr);
            let miner = node.clone();
            let syncer = node.clone();

            let (block_sender, block_receiver) = bounded(1000);
            // 从其他Peers同步数据，当同步到新区块时，通过 block_sender 发送给 miner
            thread::spawn(move || syncer.sync(block_sender));
            // 挖矿过程中若收到其他peers的区块，会取消本次挖矿，添加收到的区块
            thread::spawn(move || miner.mine(block_receiver));
            // HTTP Server
            server::run(node);
        }
    }
}

fn new_arc_node(
    addr: String,
    miner: String,
    bootstrap_addr: Option<String>,
) -> Arc<Node<FileState, HttpPeer>> {
    let file_state = FileState::new(MINING_DIFFICULTY).unwrap();
    let http_peer = HttpPeer::new();
    Arc::new(Node::new(addr, miner, bootstrap_addr, file_state, http_peer).unwrap())
}
