use crate::error::ChainError;

pub trait Peer {
    fn ping(&self, my_addr: &str, peer_addr: &str) -> Result<(), ChainError>;
}
