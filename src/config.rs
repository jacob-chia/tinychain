use std::fs;

use serde::Deserialize;
use wallet::WalletConfig;

use crate::error::Error;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Config {
    /// The path to the data directory.
    pub data_dir: String,
    /// The path to the genesis file.
    pub genesis_file: String,
    /// The address to listen on for HTTP Server.
    pub http_addr: String,
    /// The miner account to receive mining rewards.
    pub author: String,
    /// Wallet configuration.
    pub wallet: WalletConfig,
}

impl Config {
    /// Load the configuration from the given path.
    pub fn load(path: &str) -> Result<Self, Error> {
        let content =
            fs::read_to_string(path).map_err(|_| Error::ConfigNotExist(path.to_string()))?;

        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_config() {
        let mut config_file = project_root::get_project_root().unwrap();
        config_file.push("config-template.toml");
        let path_str = config_file.to_str().unwrap();

        let Config {
            data_dir,
            genesis_file,
            http_addr,
            author: miner,
            wallet,
        } = Config::load(path_str).unwrap();

        assert_eq!(data_dir, "./db/database/");
        assert_eq!(genesis_file, "./genesis.json");
        assert_eq!(http_addr, "127.0.0.1:8000");
        assert_eq!(miner, "0xb98836a093828d1c97d26eba9270a670652231e1");
        assert_eq!(wallet.keystore_dir, "./db/keystore/");
    }
}
