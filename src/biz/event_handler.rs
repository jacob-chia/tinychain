use log::{error, info};

use crate::{
    network::p2p::{EventHandler, P2pError},
    schema::{request, Block, Method, Request, Response, SignedTx},
    types::Topic,
};

use super::{Node, Peer, State};

impl<S: State, P: Peer> EventHandler for Node<S, P> {
    fn handle_inbound_request(&self, request: Vec<u8>) -> Result<Vec<u8>, P2pError> {
        let req = Request::try_from(request);
        if req.is_err() {
            error!("❌ >> [P2P-IN] Invalid request: {:?}", req.err());
            return Err(P2pError::RequestRejected);
        }
        let req = req.unwrap();

        info!("📣 >> [P2P-IN] {:?}", req);
        let resp = match req.method() {
            Method::Height => {
                let block_height = self.block_height();
                Response::new_block_height_resp(block_height)
            }
            Method::Blocks => {
                let blocks = match req.body.unwrap() {
                    request::Body::BlocksReq(req) => self.get_blocks(req.from_number),
                    _ => vec![],
                };
                Response::new_blocks_resp(blocks)
            }
        };
        info!("📣 << [P2P-OUT] {:?}", resp);

        Ok(resp.into())
    }

    fn handle_broadcast(&self, topic: &str, message: Vec<u8>) {
        match Topic::from(topic) {
            Topic::Block => {
                if let Ok(block) = Block::try_from(message) {
                    info!("📣 >> [P2P-IN-BROADCAST] {}", block);
                    self.add_block_stop_mining(block);
                } else {
                    error!("❌ >> [P2P-IN-BROADCAST] Invalid block");
                }
            }
            Topic::Tx => {
                if let Ok(tx) = SignedTx::try_from(message) {
                    info!("📣 >> [P2P-IN-BROADCAST] {}", tx);
                    let _ = self.add_pending_tx(tx);
                } else {
                    error!("❌ >> [P2P-IN-BROADCAST] Invalid tx");
                }
            }
        }
    }
}
