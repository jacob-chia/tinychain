use clap::{Parser, Subcommand};
use data::HttpPeer;
use log::info;

mod data;
mod database;
mod error;
mod node;
mod server;
mod types;
mod utils;
mod wallet;

use node::Node;

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

#[tokio::main]
async fn main() {
    env_logger::init();

    // 解析命令行参数
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
            database::init_database_dir(&datadir);

            let http_peer = HttpPeer::new();
            let node = Node::new(addr, miner, bootstrap_addr, http_peer)
                .await
                .unwrap();

            server::run(node).await;
        }
    }
}
