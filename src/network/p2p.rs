//! There are two main components in this module:
//!
//! - `P2pClient` is a wrapper around `tinyp2p::Client` that implements the `Peer` trait.
//! - `EventHandlerImpl` is an implementation of `EventHandler` that handles inbound requests and
//!   broadcasts.

use std::ops::Deref;

use log::{error, info};
use tinyp2p::{config::P2pConfig, Client, EventHandler, P2pError, Server};

use crate::{
    biz::{Node, PeerClient, State},
    error::Error,
    schema::*,
};

// Re-export libp2p functions.
pub use tinyp2p::new_secret_key;

/// Creates a new p2p client, event loop, and server.
pub fn new<S: State>(config: P2pConfig) -> Result<(P2pClient, Server<EventHandlerImpl<S>>), Error> {
    let (client, p2p_server) = tinyp2p::new(config)?;
    let p2p_client = P2pClient::new(client);

    Ok((p2p_client, p2p_server))
}

/// `P2pClient` is a wrapper around `tinyp2p::Client` that implements the `Peer` trait.
#[derive(Debug, Clone)]
pub struct P2pClient(Client);

impl P2pClient {
    pub fn new(client: Client) -> Self {
        Self(client)
    }
}

// Implement `Deref` so that we can call `Client` methods on `P2pClient`.
impl Deref for P2pClient {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PeerClient for P2pClient {
    fn known_peers(&self) -> Vec<String> {
        let peers = self.get_known_peers();
        // Getting self known peers doesn't involve any network calls,
        // so the log is not tagged with `[IN]/[OUT]`.
        info!("📣 Known peers {:?}", peers);
        peers
    }

    fn get_block_height(&self, peer_id: &str) -> Result<u64, Error> {
        let req = Request::new_block_height_req();
        info!("📣 >> [OUT] get_block_height from: {}", peer_id);
        let resp: Response = self.blocking_request(peer_id, req.into())?.try_into()?;
        info!("📣 << [IN] get_block_height response: {:?}", resp);

        Ok(BlockHeightResp::from(resp).block_height)
    }

    fn get_blocks(&self, peer_id: &str, from_number: u64) -> Result<Vec<Block>, Error> {
        let req = Request::new_blocks_req(from_number);
        info!("📣 >> [OUT] get_blocks from: {}, by: {:?}", peer_id, req);
        let resp: Response = self.blocking_request(peer_id, req.into())?.try_into()?;
        let blocks = BlocksResp::from(resp).blocks;
        info!("📣 << [IN] get_blocks count: {:?}", blocks.len());

        Ok(blocks)
    }

    fn broadcast_tx(&self, tx: SignedTx) {
        info!("📣 >> [OUT-BROADCAST] tx: {}", tx);
        self.broadcast(Topic::Tx, tx.into());
    }

    fn broadcast_block(&self, block: Block) {
        info!("📣 >> [OUT-BROADCAST] block: {}", block);
        self.broadcast(Topic::Block, Vec::from(&block));
    }
}

#[derive(Debug, Clone)]
pub struct EventHandlerImpl<S: State>(Node<S, P2pClient>);

impl<S: State> EventHandlerImpl<S> {
    pub fn new(node: Node<S, P2pClient>) -> Self {
        Self(node)
    }
}

impl<S: State> Deref for EventHandlerImpl<S> {
    type Target = Node<S, P2pClient>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S: State> EventHandler for EventHandlerImpl<S> {
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
                    self.handle_broadcast_block(block);
                } else {
                    error!("❌ >> [P2P-IN-BROADCAST] Invalid block");
                }
            }
            Topic::Tx => {
                if let Ok(tx) = SignedTx::try_from(message) {
                    info!("📣 >> [P2P-IN-BROADCAST] {}", tx);
                    let _ = self.handle_broadcast_tx(tx);
                } else {
                    error!("❌ >> [P2P-IN-BROADCAST] Invalid tx");
                }
            }
        }
    }
}

#[derive(Debug)]
enum Topic {
    Block,
    Tx,
}

impl From<&str> for Topic {
    fn from(topic: &str) -> Self {
        if topic == "tx" {
            Self::Tx
        } else {
            Self::Block
        }
    }
}

impl From<Topic> for String {
    fn from(topic: Topic) -> Self {
        match topic {
            Topic::Block => "block".into(),
            Topic::Tx => "tx".into(),
        }
    }
}
