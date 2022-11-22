use anyhow::{Error, Result};
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs::{self, File},
    path::Path,
};

use super::*;

const GENESIS_DATA: &str = r#"{
	"symbol": "TCH",
	"balances": {
	  "0x09eE50f2F37FcBA1845dE6FE5C762E83E65E755c": 1000000
	}
}"#;

#[derive(Debug, Deserialize, Clone)]
pub struct Genesis {
    symbol: String,
    balances: HashMap<String, u64>,
}

impl Genesis {
    pub fn load() -> Result<Self> {
        init_genesis_if_not_exists()?;
        let genesis: Self = serde_json::from_str(GENESIS_DATA)?;
        Ok(genesis)
    }

    pub fn clone_balances(&self) -> HashMap<String, u64> {
        self.balances.clone()
    }
}

fn init_genesis_if_not_exists() -> Result<(), Error> {
    let database_dir = DATABASE_DIR.get().unwrap();
    let genesis_path = GENESIS_PATH.get().unwrap();
    if Path::new(genesis_path).exists() {
        return Ok(());
    }

    fs::create_dir_all(database_dir)?;
    fs::write(&genesis_path, GENESIS_DATA)?;
    File::create(BLOCKDB_PATH.get().unwrap())?;
    Ok(())
}
