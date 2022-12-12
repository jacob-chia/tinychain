use async_trait::async_trait;

use crate::error::ChainError;

#[async_trait]
pub trait Peer {
    async fn ping(&self, my_addr: &str, peer_addr: &str) -> Result<(), ChainError>;
}
