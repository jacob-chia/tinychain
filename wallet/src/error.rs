use ethers_core::types::SignatureError;

#[derive(thiserror::Error, Debug)]
pub enum WalletError {
    #[error("Account not found: {0}")]
    AccountNotFound(String),
    #[error("Failed to access keystore")]
    DbError(#[from] std::io::Error),
    #[error(transparent)]
    InvalidSignature(#[from] SignatureError),
    #[error(transparent)]
    SignError(#[from] ethers_signers::WalletError),
}
