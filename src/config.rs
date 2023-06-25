use std::fs;

use serde::Deserialize;
use tinyp2p::P2pConfig;
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
    /// The author account to receive mining rewards.
    pub author: String,
    /// P2p configuration.
    pub p2p: P2pConfig,
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
            p2p,
            wallet,
        } = Config::load(path_str).unwrap();

        assert_eq!(data_dir, "./db/database/");
        assert_eq!(genesis_file, "./genesis.json");
        assert_eq!(http_addr, "127.0.0.1:8000");
        assert_eq!(miner, "0xb98836a093828d1c97d26eba9270a670652231e1");
        assert_eq!(wallet.keystore_dir, "./db/keystore/");

        let P2pConfig {
            addr,
            secret,
            boot_node,
            discovery_interval,
            pubsub_topics,
            req_resp,
        }: P2pConfig = p2p;

        assert_eq!(addr, "/ip4/0.0.0.0/tcp/9000");

        assert_eq!(
            secret.unwrap(),
            "XZYk2USPCmrRp7mCu5pT8XuQKprUf58qESu4QcQv9rJ"
        );

        let boot_node = boot_node.unwrap();
        assert_eq!(
            boot_node.peer_id().to_base58(),
            "12D3KooWSoC2ngFnfgSZcyJibKmZ2G58kbFcpmSPSSvDxeqkBLJc"
        );
        assert_eq!(boot_node.address().to_string(), "/ip4/127.0.0.1/tcp/9000");
        assert_eq!(discovery_interval, Some(30));
        assert_eq!(
            pubsub_topics,
            vec![String::from("block"), String::from("tx")]
        );

        let req_resp = req_resp.unwrap();
        assert_eq!(req_resp.connection_keep_alive, Some(10));
        assert_eq!(req_resp.request_timeout, Some(10));
        assert_eq!(req_resp.max_request_size, Some(1048576));
        assert_eq!(req_resp.max_response_size, Some(1048576));
    }
}
