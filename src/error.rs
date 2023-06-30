#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to decode requests")]
    BadRequest(#[from] serde_json::Error),
    #[error("Failed to decode hash from hex")]
    InvalidHex(#[from] hex::FromHexError),
    #[error(transparent)]
    InvalidP2pMessage(#[from] prost::DecodeError),
}
