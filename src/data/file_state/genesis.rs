use std::{
    collections::HashMap,
    fs::{self, File},
    path::Path,
};

use serde::Deserialize;

use super::{BLOCKDB_PATH, DATABASE_DIR, GENESIS_PATH};
use crate::error::ChainError;

const GENESIS_DATA: &str = r#"{
	"symbol": "TCH",
	"balances": {
	  "2bde5a91-6411-46ba-9173-c3e075d32100": 100000000
	}
}"#;

#[derive(Debug, Deserialize, Clone)]
pub struct Genesis {
    pub(super) symbol: String,
    pub(super) balances: HashMap<String, u64>,
}

impl Genesis {
    pub fn load() -> Result<Self, ChainError> {
        init_genesis_if_not_exists()?;
        let genesis: Self = serde_json::from_str(GENESIS_DATA)?;
        Ok(genesis)
    }

    pub fn clone_balances(&self) -> HashMap<String, u64> {
        self.balances.clone()
    }
}

fn init_genesis_if_not_exists() -> Result<(), ChainError> {
    let database_dir = DATABASE_DIR.get().unwrap();
    let genesis_path = GENESIS_PATH.get().unwrap();
    if Path::new(genesis_path).exists() {
        return Ok(());
    }

    fs::create_dir_all(database_dir)?;
    fs::write(genesis_path, GENESIS_DATA)?;
    File::create(BLOCKDB_PATH.get().unwrap())?;
    Ok(())
}
