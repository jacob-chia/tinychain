#[derive(thiserror::Error, Debug)]
pub enum WalletError {
    #[error("Account not found: {0}")]
    AccountNotFound(String),
    #[error("Failed to access keystore")]
    DbError(#[from] sled::Error),
    #[error(transparent)]
    SignError(#[from] k256::ecdsa::Error),
    #[error("Invalid signature")]
    InvalidSignature,
}
