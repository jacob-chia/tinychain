use crate::types::Hash;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Config file not exist: {0}")]
    ConfigNotExist(String),
    #[error(transparent)]
    InvalidConfig(#[from] toml::de::Error),
    #[error("Genesis file not exist: {0}")]
    GenesisNotExist(String),
    #[error("Invalid genesis")]
    InvalidGenesis,
    #[error("Invalid http address: {0}")]
    InvalidHttpAddr(#[from] std::net::AddrParseError),
    #[error("Failed to decode requests")]
    JsonError(#[from] serde_json::Error),
    #[error("Failed to decode hash from hex")]
    HashError(#[from] hex::FromHexError),
    #[error("Invalid tx nonce from '{0}': expected '{1}', got '{2}'")]
    InvalidTxNonce(String, u64, u64),
    #[error("Balance of '{0}' is insufficient: balance '{1}', cost '{2}'")]
    BalanceInsufficient(String, u64, u64),
    #[error("Balance of '{0}' is overflow: balance '{1}', received '{2}'")]
    BalanceOverflow(String, u64, u64),
    #[error("Invalid block number: expected '{0}', not '{1}'")]
    InvalidBlockNumber(u64, u64),
    #[error("Invalid block parent: expected '{0}', not '{1}'")]
    InvalidBlockParent(Hash, Hash),
    #[error("Block hash '{0}' donot meet the mining difficulty '{1}'")]
    InvalidBlockHash(Hash, usize),
    #[error(transparent)]
    InvalidReqResp(#[from] prost::DecodeError),
    #[error("Failed to access db")]
    DbError(#[from] sled::Error),
    #[error("Failed to add block to db")]
    AddBlockError,

    #[error(transparent)]
    WalletError(#[from] wallet::WalletError),
    #[error(transparent)]
    P2pError(#[from] tinyp2p::Error),
}
