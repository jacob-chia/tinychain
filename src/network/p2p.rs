use crate::{
    biz::PeerClient,
    error::Error,
    schema::{Block, SignedTx},
};

#[derive(Debug, Clone)]
pub struct P2pClient;

impl PeerClient for P2pClient {
    fn known_peers(&self) -> Vec<String> {
        todo!()
    }

    fn get_block_height(&self, _peer_id: &str) -> Result<u64, Error> {
        todo!()
    }

    fn get_blocks(&self, _peer_id: &str, _from_number: u64) -> Result<Vec<Block>, Error> {
        todo!()
    }

    fn broadcast_tx(&self, _tx: SignedTx) {
        todo!()
    }

    fn broadcast_block(&self, _block: Block) {
        todo!()
    }
}
