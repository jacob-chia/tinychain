use std::{fs, path::Path};

use serde::Deserialize;
use tinyp2p::P2pConfig;

use crate::error::Error;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Config {
    /// The path to the data directory.
    pub datadir: String,
    /// The address to listen on for HTTP Server.
    pub http_addr: String,
    /// The miner address to receive mining rewards.
    pub miner: String,
    /// P2p configuration.
    pub p2p: P2pConfig,
}

impl Config {
    /// Load the configuration from the given path.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Error> {
        let config_str = fs::read_to_string(path.as_ref())
            .map_err(|_| Error::ConfigNotExist(path.as_ref().to_path_buf()))?;

        let config: Config = toml::from_str(&config_str)?;
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
        let Config {
            datadir,
            http_addr,
            miner,
            p2p,
        } = Config::load(&config_file).unwrap();

        assert_eq!(datadir, "./db/");
        assert_eq!(http_addr, "127.0.0.1:8000");
        assert_eq!(miner, "2bde5a91-6411-46ba-9173-c3e075d32100");

        let P2pConfig {
            addr,
            secret,
            boot_node,
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

        let req_resp = req_resp.unwrap();
        assert_eq!(req_resp.connection_keep_alive, Some(10));
        assert_eq!(req_resp.request_timeout, Some(10));
        assert_eq!(req_resp.max_request_size, Some(1048576));
        assert_eq!(req_resp.max_response_size, Some(1048576));
    }
}
