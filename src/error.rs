#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Config file not exist: {0}")]
    ConfigNotExist(String),
    #[error(transparent)]
    InvalidConfig(#[from] toml::de::Error),
    #[error("Failed to decode requests")]
    BadRequest(#[from] serde_json::Error),
    #[error("Failed to decode hash from hex")]
    InvalidHex(#[from] hex::FromHexError),
    #[error(transparent)]
    InvalidP2pMessage(#[from] prost::DecodeError),

    #[error(transparent)]
    WalletFailure(#[from] wallet::WalletError),
    #[error(transparent)]
    P2pFailure(#[from] tinyp2p::P2pError),
}
