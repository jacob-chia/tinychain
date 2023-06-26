//! There are three main components in the p2p module:
//!
//! - `P2pClient` is a wrapper around `tinyp2p::Client` that implements the `Peer` trait.
//! - `tinyp2p::Server` handles requests from the `P2pClient`, and notifies `EventLoop` of events.
//! - `EventLoop` handles events from `tinyp2p::Server`.

use std::ops::Deref;

use log::info;
use tinyp2p::{config::P2pConfig, Client, Server};

use crate::{error::Error, node::Peer, schema::*, types::Topic};

// Re-export libp2p functions.
pub use tinyp2p::new_secret_key;

/// Creates a new p2p client, event loop, and server.
pub fn new(config: P2pConfig) -> Result<(P2pClient, Server), Error> {
    let (client, p2p_server) = tinyp2p::new(config)?;
    let p2p_client = P2pClient::new(client.clone());

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

impl Peer for P2pClient {
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
