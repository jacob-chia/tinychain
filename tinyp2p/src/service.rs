//! The main entry point of this crate, composed of three parts:
//!
//! - `Client`: the client side of the crate, which is used to send requests to the server.
//!
//! - `Server`: the server side of the crate, which is used to handle requests from the
//!    client, and notify the application layer of events.
//!
//! - `OutEvent`: the events sent by the server, which is used to notify the application.
//!
//! ## How to handle swarm events?
//!
//! > See `handle_swarm_event` function below.
//!
//! 1. When to add an address to the DHT?
//!
//! See [Discovery Discrepancies](https://docs.rs/libp2p/latest/libp2p/kad/index.html#important-discrepancies) first.
//! So, every time we receive a `identify::Event::Received` event, we should manually add the peer's addresses to the DHT.
//!
//! 2. When to remove the peer from the DHT?
//!
//! See [source code](https://github.com/libp2p/rust-libp2p/blob/master/protocols/kad/src/behaviour.rs#L1765) first.
//!
//! Currently (libp2p-0.51.3), kad never removes peers from the DHT, so we manually remove the peer when:
//! - A connected peer is unreachable (received a `ping::Event {result: Err(_), ..}`).
//! - Cannot connect to a peer that is in the DHT (received a `SwarmEvent::OutgoingConnectionError`).

use std::{collections::HashMap, io};

use itertools::Itertools;
use libp2p::{
    futures::prelude::*,
    gossipsub, identify,
    identity::ed25519,
    ping,
    request_response::RequestId,
    swarm::{SwarmBuilder, SwarmEvent},
    Multiaddr, PeerId, Swarm,
};
use log::{error, info};
use tokio::{
    select,
    sync::{
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        oneshot,
    },
};
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::{behaviour::*, config::P2pConfig, error::Error, transport};

#[derive(Clone, Debug)]
pub struct Client {
    cmd_sender: UnboundedSender<Command>,
}

#[derive(Debug)]
pub enum OutEvent {
    InboundRequest {
        request_id: RequestId,
        payload: Vec<u8>,
    },
    Broadcast {
        source: PeerId,
        topic: Topic,
        message: Vec<u8>,
    },
}

pub struct Server {
    /// The local peer id.
    local_peer_id: PeerId,
    /// The addresses that the server is listening on.
    listened_addresses: Vec<Multiaddr>,
    /// The actual network service.
    network_service: Swarm<Behaviour>,
    /// The receiver of commands from the client.
    cmd_receiver: UnboundedReceiver<Command>,
    /// The sender of events to the application layer.
    event_sender: UnboundedSender<OutEvent>,
}

/// Create a new secret key.
pub fn new_secret_key() -> String {
    let secret = ed25519::SecretKey::generate();
    bs58::encode(secret.as_ref()).into_string()
}

/// Create the `Client`, `Server` and `OutEvent` stream.
pub fn new(config: P2pConfig) -> Result<(Client, impl Stream<Item = OutEvent>, Server), Error> {
    let (cmd_sender, cmd_receiver) = mpsc::unbounded_channel();
    let (event_sender, event_receiver) = mpsc::unbounded_channel();
    let event_stream = UnboundedReceiverStream::new(event_receiver);

    let server = Server::new(config, cmd_receiver, event_sender)?;
    let client = Client { cmd_sender };

    Ok((client, event_stream, server))
}

impl Client {
    /// Send a blocking request to the `target` peer.
    pub fn blocking_request(&self, target: &str, request: Vec<u8>) -> Result<Vec<u8>, Error> {
        let target = target.parse().map_err(|_| Error::InvalidPeerId)?;

        let (responder, receiver) = oneshot::channel();
        let _ = self.cmd_sender.send(Command::SendRequest {
            target,
            request,
            responder,
        });
        receiver.blocking_recv()?
    }

    /// Send a response to the peer that sent the request.
    pub fn send_response(&self, request_id: RequestId, response: Result<Vec<u8>, ()>) {
        let _ = self.cmd_sender.send(Command::SendResponse {
            request_id,
            response,
        });
    }

    /// Publish a message to the given topic.
    pub fn broadcast(&self, topic: Topic, message: Vec<u8>) {
        let _ = self.cmd_sender.send(Command::Broadcast { topic, message });
    }

    /// Get status of the node.
    pub fn get_node_status(&self) -> NodeStatus {
        let (responder, receiver) = oneshot::channel();
        let _ = self.cmd_sender.send(Command::GetStatus(responder));
        receiver.blocking_recv().unwrap_or_default()
    }

    /// Get known peers of the node.
    pub fn get_known_peers(&self) -> Vec<String> {
        self.get_node_status()
            .known_peers
            .into_keys()
            .map(|id| id.to_base58())
            .collect()
    }
}

/// The commands sent by the `Client` to the `Server`.
pub enum Command {
    SendRequest {
        target: PeerId,
        request: Vec<u8>,
        responder: oneshot::Sender<Result<Vec<u8>, Error>>,
    },
    SendResponse {
        request_id: RequestId,
        response: Result<Vec<u8>, ()>,
    },
    Broadcast {
        topic: Topic,
        message: Vec<u8>,
    },
    GetStatus(oneshot::Sender<NodeStatus>),
}

/// The node status, for debugging.
#[derive(Clone, Debug, Default)]
pub struct NodeStatus {
    pub local_peer_id: String,
    pub listened_addresses: Vec<Multiaddr>,
    pub known_peers_count: usize,
    pub known_peers: HashMap<PeerId, Vec<Multiaddr>>,
}

impl Server {
    /// Create a new `Server`.
    pub fn new(
        config: P2pConfig,
        cmd_receiver: UnboundedReceiver<Command>,
        event_sender: UnboundedSender<OutEvent>,
    ) -> Result<Self, Error> {
        let addr = config.addr.parse()?;
        let local_key = config.gen_keypair()?;
        let local_peer_id = local_key.public().to_peer_id();
        info!("📣 Local peer id: {local_peer_id:?}");

        // Build the [swarm](https://docs.rs/libp2p/latest/libp2p/struct.Swarm.html)
        let mut swarm = {
            let transport = transport::build_transport(local_key.clone());
            let behaviour = Behaviour::new(local_key, config.req_resp)?;
            SwarmBuilder::with_tokio_executor(transport, behaviour, local_peer_id.clone()).build()
        };

        // Tell the swarm to listen on all interfaces and a random, OS-assigned port.
        swarm.listen_on(addr)?;

        // Connect to boot node if specified.
        if let Some(boot_node) = config.boot_node {
            swarm.dial(boot_node.address())?;
        }

        Ok(Self {
            local_peer_id,
            listened_addresses: Vec::new(),
            network_service: swarm,
            cmd_receiver,
            event_sender,
        })
    }

    /// Run the `Server`.
    pub async fn run(mut self) {
        loop {
            self.next_action().await;
        }
    }

    async fn next_action(&mut self) {
        select! {
            // Next command from the `Client`.
            msg = self.cmd_receiver.recv() => {
                if let Some(cmd) = msg {
                    self.handle_command(cmd);
                }
            },
            // Next event from `Swarm` (the stream guaranteed to never terminate).
            event = self.network_service.select_next_some() => {
                self.handle_swarm_event(event);
            },
        }
    }

    /// Process the next command coming from `Client`.
    fn handle_command(&mut self, cmd: Command) {
        match cmd {
            Command::SendRequest {
                target,
                request,
                responder,
            } => {
                self.network_service
                    .behaviour_mut()
                    .send_request(&target, request, responder);
            }
            Command::SendResponse {
                request_id,
                response,
            } => {
                self.network_service
                    .behaviour_mut()
                    .send_response(request_id, response);
            }
            Command::Broadcast { topic, message } => {
                let _ = self
                    .network_service
                    .behaviour_mut()
                    .broadcast(topic, message);
            }
            Command::GetStatus(responder) => {
                let _ = responder.send(self.get_status());
            }
        }
    }

    /// Process the next event coming from `Swarm`.
    fn handle_swarm_event(&mut self, event: SwarmEvent<BehaviourEvent, BehaviourErr>) {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                info!("📣 P2P node listening on {:?}", address);
                self.update_listened_addresses()
            }

            SwarmEvent::ListenerClosed {
                reason, addresses, ..
            } => Self::log_listener_close(reason, addresses),

            SwarmEvent::OutgoingConnectionError {
                peer_id: Some(peer),
                ..
            } => self.network_service.behaviour_mut().remove_peer(&peer),

            SwarmEvent::Behaviour(BehaviourEvent::Identify(identify::Event::Received {
                peer_id,
                info: identify::Info { listen_addrs, .. },
            })) => self.add_addresses(&peer_id, listen_addrs),

            SwarmEvent::Behaviour(BehaviourEvent::Ping(ping::Event {
                peer,
                result: Err(_),
            })) => self.network_service.behaviour_mut().remove_peer(&peer),

            SwarmEvent::Behaviour(BehaviourEvent::ReqResp(req_resp::Event::InboundRequest {
                request_id,
                payload,
            })) => {
                let _ = self.event_sender.send(OutEvent::InboundRequest {
                    request_id,
                    payload,
                });
            }

            SwarmEvent::Behaviour(BehaviourEvent::Gossipsub(gossipsub::Event::Message {
                propagation_source: source,
                message_id: _,
                message,
            })) => {
                let _ = self.event_sender.send(OutEvent::Broadcast {
                    source,
                    topic: message.topic.into(),
                    message: message.data,
                });
            }

            // Ignore other events
            _ => {}
        }
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
