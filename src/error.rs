use std::path::PathBuf;

use crate::types::Hash;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Config file not exist: {0}")]
    ConfigNotExist(PathBuf),
    #[error(transparent)]
    InvalidConfig(#[from] toml::de::Error),
    #[error("Invalid http address: {0}")]
    InvalidHttpAddr(#[from] std::net::AddrParseError),
    #[error("Failed to decode requests")]
    BadRequest(#[from] serde_json::Error),
    #[error("Invalid tx signature from {0}")]
    InvalidTxSignature(String),
    #[error("Invalid tx nonce from {0}: expected '{1}', not '{2}'")]
    InvalidTxNonce(String, u64, u64),
    #[error("Insufficient balance of {0}: cost '{1}', balance is '{2}'")]
    InsufficientBalance(String, u64, u64),
    #[error("Block not found, number: {0}")]
    BlockNotFound(u64),
    #[error("Invalid block number: expected '{0}', not '{1}'")]
    InvalidBlockNumber(u64, u64),
    #[error("Invalid block parent: expected '{0}', not '{1}'")]
    InvalidBlockParent(Hash, Hash),
    #[error("Block hash '{0}' donot meet the mining difficulty '{1}'")]
    InvalidBlockHash(Hash, usize),
    #[error(transparent)]
    InvalidReqResp(#[from] prost::DecodeError),
    #[error("Failed to access db")]
    DbError(#[from] std::io::Error),

    #[error(transparent)]
    WalletError(#[from] wallet::WalletError),
    #[error(transparent)]
    P2pError(#[from] tinyp2p::Error),
}
