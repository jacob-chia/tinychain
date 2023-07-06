//! Mock p2p network for testing.

use tinyp2p::{EventHandler, P2pError};

use crate::{biz::PeerClient, error::Error, schema::*};

#[derive(Debug, Clone)]
pub struct MockP2pClient;

impl PeerClient for MockP2pClient {
    fn known_peers(&self) -> Vec<String> {
        vec![]
    }

    fn get_block_height(&self, _peer_id: &str) -> Result<u64, Error> {
        Ok(0)
    }

    fn get_blocks(&self, _peer_id: &str, _from_number: u64) -> Result<Vec<Block>, Error> {
        Ok(vec![])
    }

    fn broadcast_tx(&self, _tx: SignedTx) {}

    fn broadcast_block(&self, _block: Block) {}
}

#[derive(Debug, Clone)]
pub struct MockEventHandlerImpl;

impl EventHandler for MockEventHandlerImpl {
    fn handle_inbound_request(&self, _request: Vec<u8>) -> Result<Vec<u8>, P2pError> {
        Ok(vec![])
    }

    fn handle_broadcast(&self, _topic: &str, _message: Vec<u8>) {}
}
