//! The main entry point of this crate, composed of three parts:
//!
//! - `Client`: the client side of a p2p node, which is used to send requests to a remote
//!   peer or broadcast messages to the network.
//!
//! - `Server`: the server side of a p2p node, which is used to handle requests / broadcast-messages
//!   from remote peers.
//!
//! - `EventHandler`: the trait that defines how to handle requests / broadcast-messages from remote peers.
//!   The application should implement this trait and pass it to the `Server`.

use std::{cell::OnceCell, collections::HashMap, fmt::Debug, io, time::Duration};

use itertools::Itertools;
use libp2p::{
    futures::prelude::*,
    gossipsub::{self, TopicHash},
    identify,
    identity::ed25519,
    ping,
    request_response::{self, OutboundFailure, RequestId, ResponseChannel},
    swarm::{SwarmBuilder, SwarmEvent},
    Multiaddr, PeerId, Swarm,
};
use log::{error, info, warn};
use tokio::{
    select,
    sync::{
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        oneshot,
    },
    time::{self, Interval},
};

use crate::{config::P2pConfig, error::P2pError, protocol::*, transport};

/// `EventHandler` is the trait that defines how to handle requests / broadcast-messages from remote peers.
pub trait EventHandler: Debug + Send + 'static {
    /// Handles an inbound request from a remote peer.
    fn handle_inbound_request(&self, request: Vec<u8>) -> Result<Vec<u8>, P2pError>;

    /// Handles an broadcast message from a remote peer.
    fn handle_broadcast(&self, topic: &str, message: Vec<u8>);
}

#[derive(Clone, Debug)]
pub struct Client {
    cmd_sender: UnboundedSender<Command>,
}

pub struct Server {
    /// The actual network service.
    network_service: Swarm<Behaviour>,

    /// The local peer id.
    local_peer_id: PeerId,
    /// The addresses that the server is listening on.
    listened_addresses: Vec<Multiaddr>,
    /// The receiver of commands from the client.
    cmd_receiver: UnboundedReceiver<Command>,
    /// The handler of events from remote peers.
    event_handler: OnceCell<Box<dyn EventHandler>>,

    /// The ticker to periodically discover new peers.
    discovery_ticker: Interval,
    /// The pending outbound requests, awaiting for a response from the remote.
    pending_outbound_requests: HashMap<RequestId, oneshot::Sender<ResponseType>>,
    /// The topics will be hashed when subscribing to the gossipsub protocol,
    /// but we need to keep the original topic names for broadcasting.
    pubsub_topics: Vec<String>,
}

/// Create a new secret key for the p2p node.
pub fn new_secret_key() -> String {
    let secret = ed25519::SecretKey::generate();
    bs58::encode(secret.as_ref()).into_string()
}

/// Create a new p2p node, which consists of a `Client` and a `Server`.
pub fn new(config: P2pConfig) -> Result<(Client, Server), P2pError> {
    let (cmd_sender, cmd_receiver) = mpsc::unbounded_channel();

    let server = Server::new(config, cmd_receiver)?;
    let client = Client { cmd_sender };

    Ok((client, server))
}

impl Client {
    /// Send a blocking request to the `target` peer.
    pub fn blocking_request(&self, target: &str, request: Vec<u8>) -> Result<Vec<u8>, P2pError> {
        let target = target.parse().map_err(|_| P2pError::InvalidPeerId)?;

        let (responder, receiver) = oneshot::channel();
        let _ = self.cmd_sender.send(Command::SendRequest {
            target,
            request,
            responder,
        });
        receiver
            .blocking_recv()?
            .map_err(|_| P2pError::RequestRejected)
    }

    /// Publish a message to the given topic.
    pub fn broadcast(&self, topic: impl Into<String>, message: Vec<u8>) {
        let _ = self.cmd_sender.send(Command::Broadcast {
            topic: topic.into(),
            message,
        });
    }

    /// Get known peers of the node.
    pub fn get_known_peers(&self) -> Vec<String> {
        self.get_node_status()
            .known_peers
            .into_keys()
            .map(|id| id.to_base58())
            .collect()
    }

    /// Get status of the node for debugging.
    pub fn get_node_status(&self) -> NodeStatus {
        let (responder, receiver) = oneshot::channel();
        let _ = self.cmd_sender.send(Command::GetStatus(responder));
        receiver.blocking_recv().unwrap_or_default()
    }
}

/// The commands sent by the `Client` to the `Server`.
pub enum Command {
    SendRequest {
        target: PeerId,
        request: Vec<u8>,
        responder: oneshot::Sender<ResponseType>,
    },
    Broadcast {
        topic: String,
        message: Vec<u8>,
    },
    GetStatus(oneshot::Sender<NodeStatus>),
}

impl Server {
    /// Create a new `Server`.
    pub fn new(
        config: P2pConfig,
        cmd_receiver: UnboundedReceiver<Command>,
    ) -> Result<Self, P2pError> {
        let addr: Multiaddr = config.addr.parse()?;
        let local_key = config.gen_keypair()?;
        let local_peer_id = local_key.public().to_peer_id();
        info!("📣 Local peer id: {local_peer_id:?}");

        let pubsub_topics = config.pubsub_topics;
        // Build the [swarm](https://docs.rs/libp2p/latest/libp2p/struct.Swarm.html)
        let mut swarm = {
            let transport = transport::build_transport(local_key.clone());
            let behaviour = Behaviour::new(local_key, pubsub_topics.clone(), config.req_resp)?;
            SwarmBuilder::with_tokio_executor(transport, behaviour, local_peer_id).build()
        };
        // Switch to server mode.
        // TODO remove this when server mode can be configured.
        swarm.add_external_address(addr.clone());
        swarm.listen_on(addr)?;

        // Connect to the boot node if specified.
        if let Some(boot_node) = config.boot_node {
            swarm.dial(boot_node.address())?;
        }

        // Create a ticker to periodically discover new peers.
        let interval_secs = config.discovery_interval.unwrap_or(30);
        let instant = time::Instant::now() + Duration::from_secs(5);
        let discovery_ticker = time::interval_at(instant, Duration::from_secs(interval_secs));

        Ok(Self {
            network_service: swarm,
            local_peer_id,
            listened_addresses: Vec::new(),
            cmd_receiver,
            event_handler: OnceCell::new(),
            discovery_ticker,
            pending_outbound_requests: HashMap::new(),
            pubsub_topics,
        })
    }

    /// Set the handler of events from remote peers.
    pub fn set_event_handler(&mut self, handler: impl EventHandler) {
        self.event_handler.set(Box::new(handler)).unwrap();
    }

    /// Run the `Server`.
    pub async fn run(mut self) {
        loop {
            select! {
                // Next discovery process.
                _ = self.discovery_ticker.tick() => {
                    self.network_service.behaviour_mut().discover_peers();
                },

                // Next command from the `Client`.
                msg = self.cmd_receiver.recv() => {
                    if let Some(cmd) = msg {
                        self.handle_command(cmd);
                    }
                },
                // Next event from `Swarm`.
                event = self.network_service.select_next_some() => {
                    self.handle_swarm_event(event);
                },
            }
        }
    }

    // Process the next command coming from `Client`.
    fn handle_command(&mut self, cmd: Command) {
        match cmd {
            Command::SendRequest {
                target,
                request,
                responder,
            } => self.handle_outbound_request(target, request, responder),
            Command::Broadcast { topic, message } => self.handle_outbound_broadcast(topic, message),
            Command::GetStatus(responder) => responder.send(self.get_status()).unwrap(),
        }
    }

    // Process the next event coming from `Swarm`.
    fn handle_swarm_event(&mut self, event: SwarmEvent<BehaviourEvent, BehaviourErr>) {
        let behaviour_ev = match event {
            SwarmEvent::Behaviour(ev) => ev,

            SwarmEvent::NewListenAddr { address, .. } => {
                info!("📣 P2P node listening on {:?}", address);
                return self.update_listened_addresses();
            }

            SwarmEvent::ListenerClosed {
                reason, addresses, ..
            } => return Self::log_listener_close(reason, addresses),

            // Can't connect to the `peer`, remove it from the DHT.
            SwarmEvent::OutgoingConnectionError {
                peer_id: Some(peer),
                ..
            } => return self.network_service.behaviour_mut().remove_peer(&peer),

            _ => return,
        };

        self.handle_behaviour_event(behaviour_ev);
    }

    fn handle_behaviour_event(&mut self, ev: BehaviourEvent) {
        match ev {
            // See https://docs.rs/libp2p/latest/libp2p/kad/index.html#important-discrepancies
            BehaviourEvent::Identify(identify::Event::Received {
                peer_id,
                info: identify::Info { listen_addrs, .. },
            }) => self.add_addresses(&peer_id, listen_addrs),

            // The remote peer is unreachable, remove it from the DHT.
            BehaviourEvent::Ping(ping::Event {
                peer,
                result: Err(_),
                ..
            }) => self.network_service.behaviour_mut().remove_peer(&peer),

            BehaviourEvent::ReqResp(request_response::Event::Message {
                message:
                    request_response::Message::Request {
                        request, channel, ..
                    },
                ..
            }) => self.handle_inbound_request(request, channel),

            BehaviourEvent::ReqResp(request_response::Event::Message {
                message:
                    request_response::Message::Response {
                        request_id,
                        response,
                    },
                ..
            }) => self.handle_inbound_response(request_id, response),

            BehaviourEvent::ReqResp(request_response::Event::OutboundFailure {
                request_id,
                error,
                ..
            }) => self.handle_outbound_failure(request_id, error),

            BehaviourEvent::Pubsub(gossipsub::Event::Message {
                propagation_source: _,
                message_id: _,
                message,
            }) => self.handle_inbound_broadcast(message),

            _ => {}
        }
    }

    // Inbound requests are handled by the `EventHandler` which is provided by the application layer.
    fn handle_inbound_request(&mut self, request: Vec<u8>, ch: ResponseChannel<ResponseType>) {
        if let Some(handler) = self.event_handler.get() {
            let response = handler.handle_inbound_request(request).map_err(|_| ());
            self.network_service
                .behaviour_mut()
                .send_response(ch, response);
        }
    }

    // Store the request_id with the responder so that we can send the response later.
    fn handle_outbound_request(
        &mut self,
        target: PeerId,
        request: Vec<u8>,
        responder: oneshot::Sender<ResponseType>,
    ) {
        let req_id = self
            .network_service
            .behaviour_mut()
            .send_request(&target, request);
        self.pending_outbound_requests.insert(req_id, responder);
    }

    // An outbound request failed, notify the application layer.
    fn handle_outbound_failure(&mut self, request_id: RequestId, error: OutboundFailure) {
        if let Some(responder) = self.pending_outbound_requests.remove(&request_id) {
            error!("❌ Outbound request failed: {:?}", error);
            let _ = responder.send(Err(()));
        } else {
            warn!("❗ Received failure for unknown request: {}", request_id);
            debug_assert!(false);
        }
    }

    // An inbound response was received, notify the application layer.
    fn handle_inbound_response(&mut self, request_id: RequestId, response: ResponseType) {
        if let Some(responder) = self.pending_outbound_requests.remove(&request_id) {
            let _ = responder.send(response);
        } else {
            warn!("❗ Received response for unknown request: {}", request_id);
            debug_assert!(false);
        }
    }

    // Inbound broadcasts are handled by the `EventHandler` which is provided by the application layer.
    fn handle_inbound_broadcast(&mut self, message: gossipsub::Message) {
        if let Some(handler) = self.event_handler.get() {
            let topic_hash = message.topic;
            match self.get_topic(&topic_hash) {
                Some(topic) => handler.handle_broadcast(&topic, message.data),
                None => {
                    warn!("❗ Received broadcast for unknown topic: {:?}", topic_hash);
                    debug_assert!(false);
                }
            }
        }
    }

    // Broadcast a message to all peers subscribed to the given topic.
    fn handle_outbound_broadcast(&mut self, topic: String, message: Vec<u8>) {
        let _ = self
            .network_service
            .behaviour_mut()
            .broadcast(topic, message);
    }

    fn add_addresses(&mut self, peer_id: &PeerId, addresses: Vec<Multiaddr>) {
        for addr in addresses.into_iter().unique() {
            self.network_service
                .behaviour_mut()
                .add_address(peer_id, addr);
        }
    }

    fn get_status(&mut self) -> NodeStatus {
        let known_peers = self.network_service.behaviour_mut().known_peers();
        NodeStatus {
            local_peer_id: self.local_peer_id.to_base58(),
            listened_addresses: self.listened_addresses.clone(),
            known_peers_count: known_peers.len(),
            known_peers,
        }
    }

    fn update_listened_addresses(&mut self) {
        self.listened_addresses = self
            .network_service
            .listeners()
            .map(ToOwned::to_owned)
            .collect();
    }

    /// Returns the topic name for the given topic hash.
    fn get_topic(&self, topic_hash: &TopicHash) -> Option<String> {
        for t in &self.pubsub_topics {
            let topic = gossipsub::IdentTopic::new(t);
            if topic.hash() == *topic_hash {
                return Some(t.clone());
            }
        }

        None
    }

    fn log_listener_close(reason: io::Result<()>, addresses: Vec<Multiaddr>) {
        let addrs = addresses
            .into_iter()
            .map(|a| a.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        match reason {
            Ok(()) => {
                info!("📣 Listener ({}) closed gracefully", addrs)
            }
            Err(e) => {
                error!("❌ Listener ({}) closed: {}", addrs, e)
            }
        }
    }
}

/// The node status, for debugging.
#[derive(Clone, Debug, Default)]
pub struct NodeStatus {
    pub local_peer_id: String,
    pub listened_addresses: Vec<Multiaddr>,
    pub known_peers_count: usize,
    pub known_peers: HashMap<PeerId, Vec<Multiaddr>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_secret_key_works() {
        let key1 = new_secret_key();
        let key2 = new_secret_key();
        assert_ne!(key1, key2);

        let secret = bs58::decode(key1).into_vec().unwrap();
        assert_eq!(secret.len(), 32);
    }
}
