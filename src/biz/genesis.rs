use std::{collections::HashMap, fs};

use serde::Deserialize;

use crate::error::Error;

#[derive(Debug, Deserialize, Clone)]
pub struct Genesis {
    balances: HashMap<String, u64>,
}

impl Genesis {
    pub fn load(path: &str) -> Result<Self, Error> {
        let content =
            fs::read_to_string(path).map_err(|_| Error::ConfigNotExist(path.to_string()))?;

        serde_json::from_str(&content).map_err(|_| Error::InvalidGenesis)
    }

    pub fn into_balances(self) -> HashMap<String, u64> {
        self.balances
    }
}
