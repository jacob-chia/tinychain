use once_cell::sync::OnceCell;

mod block;
mod genesis;
mod state;
mod tx;

pub use block::*;
pub use genesis::*;
pub use state::*;
pub use tx::*;

static DATABASE_DIR: OnceCell<String> = OnceCell::new();
static GENESIS_PATH: OnceCell<String> = OnceCell::new();
static BLOCKDB_PATH: OnceCell<String> = OnceCell::new();

pub fn init_database_dir(datadir: &str) {
    let mut dir = datadir.to_owned();
    dir.push_str("database/");

    let mut genesis_path = dir.clone();
    let mut blockdb_path = genesis_path.clone();
    genesis_path.push_str("genesis.json");
    blockdb_path.push_str("block.db");

    DATABASE_DIR.get_or_init(|| dir);
    GENESIS_PATH.get_or_init(|| genesis_path);
    BLOCKDB_PATH.get_or_init(|| blockdb_path);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_dir_can_only_be_initialized_once() {
        init_database_dir("/tmp/");
        assert_eq!("/tmp/database/", DATABASE_DIR.get().unwrap());
        assert_eq!("/tmp/database/genesis.json", GENESIS_PATH.get().unwrap());
        assert_eq!("/tmp/database/block.db", BLOCKDB_PATH.get().unwrap());

        init_database_dir("/another/dir/");
        assert_eq!("/tmp/database/", DATABASE_DIR.get().unwrap());
        assert_eq!("/tmp/database/genesis.json", GENESIS_PATH.get().unwrap());
        assert_eq!("/tmp/database/block.db", BLOCKDB_PATH.get().unwrap());
    }
}
