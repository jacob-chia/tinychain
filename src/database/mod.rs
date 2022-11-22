mod block;
mod genesis;
mod state;
mod tx;

pub use block::*;
pub use genesis::*;
pub use state::*;
pub use tx::*;

use once_cell::sync::OnceCell;

static DATABASE_DIR: OnceCell<String> = OnceCell::new();
static GENESIS_PATH: OnceCell<String> = OnceCell::new();
static BLOCKDB_PATH: OnceCell<String> = OnceCell::new();

pub fn set_database_dir(datadir: &str) {
    let mut dir = datadir.to_owned();
    dir.push_str("database/");

    let mut genesis_path = dir.clone();
    let mut blockdb_path = genesis_path.clone();
    genesis_path.push_str("genesis.json");
    blockdb_path.push_str("block.db");

    DATABASE_DIR.set(dir).unwrap();
    GENESIS_PATH.set(genesis_path).unwrap();
    BLOCKDB_PATH.set(blockdb_path).unwrap();
}
