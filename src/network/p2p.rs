//! There are three main components in the p2p module:
//!
//! - `P2pClient` is a wrapper around `tinyp2p::Client` that implements the `Peer` trait.
//! - `tinyp2p::Server` handles requests from the `P2pClient`, and notifies `EventLoop` of events.
//! - `EventLoop` handles events from `tinyp2p::Server`.

use std::{ops::Deref, sync::Arc};

use log::{error, info};
use tinyp2p::{config::P2pConfig, Client, OutEvent, Server, Topic};
use tokio_stream::{Stream, StreamExt};

use crate::{
    error::Error,
    node::{Block, Node, Peer, SignedTx, State},
};

use super::schema::{self, request, BestNumberResp, BlocksResp, Method, Request, Response};

// Re-export libp2p functions.
pub use tinyp2p::new_secret_key;

/// Creates a new p2p client, event loop, and server.
pub fn new(config: P2pConfig) -> Result<(P2pClient, EventLoop, Server), Error> {
    let (client, event_stream, p2p_server) = tinyp2p::new(config)?;
    let p2p_client = P2pClient::new(client.clone());
    let event_loop = EventLoop {
        event_stream: Box::new(event_stream),
        client: client.clone(),
    };

    Ok((p2p_client, event_loop, p2p_server))
}

/// `EventLoop` handles events from the p2p server.
pub struct EventLoop {
    /// Stream of events from the p2p server.
    event_stream: Box<dyn Stream<Item = OutEvent> + Unpin + Send>,
    /// Client to send responses to the p2p server.
    client: Client,
}

impl EventLoop {
    pub async fn run<S, P>(mut self, node: Arc<Node<S, P>>)
    where
        S: State + Send + Sync + 'static,
        P: Peer + Send + Sync + 'static,
    {
        let client = self.client.clone();
        loop {
            match self.event_stream.next().await {
                Some(OutEvent::InboundRequest {
                    request_id,
                    payload,
                }) => {
                    self.handle_inbound_request(node.clone(), client.clone(), request_id, payload)
                }
                Some(OutEvent::Broadcast { topic, message, .. }) => {
                    self.handle_broadcast(node.clone(), topic, message)
                }
                None => continue,
            }
        }
    }

    fn handle_inbound_request<S, P>(
        &self,
        node: Arc<Node<S, P>>,
        client: Client,
        request_id: tinyp2p::RequestId,
        payload: Vec<u8>,
    ) where
        S: State + Send + Sync + 'static,
        P: Peer + Send + Sync + 'static,
    {
        let req = Request::try_from(payload);
        if req.is_err() {
            error!("❌ >> [IN] Invalid request: {:?}", req.err());
            return client.send_response(request_id, Err(()));
        }

        let req = req.unwrap();
        info!("📣 >> [IN] {:?}", req);
        let resp = match req.method() {
            Method::BestNumber => {
                let best_number = node.latest_block_number();
                Response::new_best_number_resp(best_number)
            }
            Method::Blocks => {
                let blocks = match req.body.unwrap() {
                    request::Body::BlocksReq(req) => {
                        node.get_blocks(req.from_number).unwrap_or_default()
                    }
                    _ => vec![],
                };
                Response::new_blocks_resp(blocks)
            }
        };
        info!("📣 << [OUT] {:?}", resp);

        client.send_response(request_id, Ok(resp.into()));
    }

    fn handle_broadcast<S, P>(&self, node: Arc<Node<S, P>>, topic: Topic, message: Vec<u8>)
    where
        S: State + Send + Sync + 'static,
        P: Peer + Send + Sync + 'static,
    {
        match topic {
            Topic::Block => {
                let block = schema::Block::try_from(message);
                if block.is_err() {
                    error!("❌ >> [IN-BROADCAST] Invalid block: {:?}", block.err());
                    return;
                }

                let block = block.unwrap().into();
                info!("📣 >> [IN-BROADCAST] {:?}", block);
                node.add_block_stop_mining(block);
            }
            Topic::Tx => {
                let tx = schema::SignedTx::try_from(message);
                if tx.is_err() {
                    error!("❌ [IN-BROADCAST] Invalid tx: {:?}", tx.err());
                    return;
                }
                let tx = tx.unwrap();
                info!("📣 >> [IN-BROADCAST] {:?}", tx);
                let _ = node.add_pending_tx(tx.into());
            }
        }
    }
}

/// `P2pClient` is a wrapper around `tinyp2p::Client` that implements the `Peer` trait.
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
        // so the log is not tagged with `[IN]` or `[OUT]`.
        info!("📣 Known peers {:?}", peers);
        peers
    }

    fn get_best_number(&self, peer_id: &str) -> Result<Option<u64>, Error> {
        let req = Request::new_best_number_req();
        info!("📣 >> [OUT] get_best_number from: {}", peer_id);
        let resp: Response = self.blocking_request(peer_id, req.into())?.try_into()?;
        info!("📣 << [IN] get_best_number response: {:?}", resp);

        Ok(BestNumberResp::from(resp).best_number)
    }

    fn get_blocks(&self, peer_id: &str, from_number: u64) -> Result<Vec<Block>, Error> {
        let req = Request::new_blocks_req(from_number);
        info!("📣 >> [OUT] get_blocks from: {}, by: {:?}", peer_id, req);
        let resp: Response = self.blocking_request(peer_id, req.into())?.try_into()?;
        info!("📣 << [IN] get_blocks response: {:?}", resp);

        // DTO (schema::Block) -> DO (node::Block)
        let blocks = BlocksResp::from(resp)
            .blocks
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(blocks)
    }

    fn broadcast_tx(&self, tx: SignedTx) {
        info!("📣 >> [OUT-BROADCAST] tx: {:?}", tx);
        let tx = schema::SignedTx::from(tx);
        self.broadcast(Topic::Tx, tx.into());
    }

    fn broadcast_block(&self, block: Block) {
        info!("📣 >> [OUT-BROADCAST] block: {:?}", block);
        let block = schema::Block::from(block);
        self.broadcast(Topic::Block, block.into());
    }
}
