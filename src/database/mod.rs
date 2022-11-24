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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_database_dir() {
        set_database_dir("/tmp/");
        assert_eq!("/tmp/database/", DATABASE_DIR.get().unwrap());
        assert_eq!("/tmp/database/genesis.json", GENESIS_PATH.get().unwrap());
        assert_eq!("/tmp/database/block.db", BLOCKDB_PATH.get().unwrap());
    }
}
