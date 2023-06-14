//! A `NetworkBehaviour` that discovers peers in the network.
//!
//! [What is Discovery](https://docs.libp2p.io/concepts/discovery-routing/overview/)
//!
//! This crate uses the [Bootstrap Process](https://docs.libp2p.io/concepts/discovery-routing/kaddht/#bootstrap-process)
//! to maintain a healthy routing table and discover new nodes.
//! The bootstrap process needs a [Boot Node](https://docs.libp2p.io/concepts/glossary/#boot-node)

use std::{
    cmp,
    collections::HashMap,
    task::{Context, Poll},
    time::Duration,
};

use futures_timer::Delay;
use libp2p::{
    core::Endpoint,
    futures::FutureExt,
    kad::{
        handler::KademliaHandler, store::MemoryStore, Kademlia, KademliaConfig, KademliaEvent,
        QueryId,
    },
    swarm::{
        ConnectionDenied, ConnectionId, FromSwarm, NetworkBehaviour, PollParameters, THandler,
        THandlerInEvent, THandlerOutEvent, ToSwarm,
    },
    Multiaddr, PeerId,
};
use log::debug;

/// A `NetworkBehaviour` that discovers peers in the network.
pub struct Behaviour {
    /// The actual Kademlia behaviour.
    kademlia: Kademlia<MemoryStore>,
    /// Timer that fires when we need to perform the next discovery process.
    next_discovery: Delay,
    /// Interval of discovery process.
    discovery_interval: Duration,
}

impl Behaviour {
    pub fn new(local_peer_id: PeerId) -> Behaviour {
        let kademlia = {
            let mut config = KademliaConfig::default();
            let proto_names = vec!["/tinychain/discovery/1.0.0".as_bytes().into()];
            config.set_protocol_names(proto_names);
            let store = MemoryStore::new(local_peer_id);
            Kademlia::with_config(local_peer_id, store, config)
        };

        Behaviour {
            kademlia,
            next_discovery: Delay::new(Duration::new(0, 0)),
            discovery_interval: Duration::from_secs(5),
        }
    }

    pub fn known_peers(&mut self) -> HashMap<PeerId, Vec<Multiaddr>> {
        let mut peers = HashMap::new();
        for b in self.kademlia.kbuckets() {
            for e in b.iter() {
                peers.insert(*e.node.key.preimage(), e.node.value.clone().into_vec());
            }
        }

        peers
    }

    pub fn add_address(&mut self, peer_id: &PeerId, addr: Multiaddr) {
        debug!("☕ Adding address {} from {:?} to the DHT.", addr, peer_id);
        self.kademlia.add_address(peer_id, addr);
    }

    pub fn remove_peer(&mut self, peer_id: &PeerId) {
        debug!("☕ Removing peer {} from the DHT.", peer_id);
        self.kademlia.remove_peer(peer_id);
    }
}

/// We only care about the `poll` function.
/// The other functions just forward to the underlying `Kademlia` behaviour.
impl NetworkBehaviour for Behaviour {
    type ConnectionHandler = KademliaHandler<QueryId>;
    type OutEvent = KademliaEvent;

    fn poll(
        &mut self,
        cx: &mut Context<'_>,
        params: &mut impl PollParameters,
    ) -> Poll<ToSwarm<Self::OutEvent, THandlerInEvent<Self>>> {
        // Poll the stream that fires when we need to start a discovery process.
        while self.next_discovery.poll_unpin(cx).is_ready() {
            if self.known_peers().is_empty() {
                debug!("☕ Discovery process paused due to no boot node");
            } else {
                debug!("☕ Starting a discovery process");
                let _ = self.kademlia.bootstrap();
            }

            // Schedule the next discovery process with exponentially increasing delay,
            // capped at 60s.
            self.next_discovery = Delay::new(self.discovery_interval);
            self.discovery_interval =
                cmp::min(self.discovery_interval * 2, Duration::from_secs(60));
        }

        // Poll the Kademlia behaviour.
        self.kademlia.poll(cx, params)
    }

    fn on_swarm_event(&mut self, event: FromSwarm<Self::ConnectionHandler>) {
        self.kademlia.on_swarm_event(event)
    }

    fn on_connection_handler_event(
        &mut self,
        peer_id: PeerId,
        connection_id: ConnectionId,
        event: THandlerOutEvent<Self>,
    ) {
        self.kademlia
            .on_connection_handler_event(peer_id, connection_id, event)
    }

    fn handle_established_inbound_connection(
        &mut self,
        connection_id: ConnectionId,
        peer: PeerId,
        local_addr: &Multiaddr,
        remote_addr: &Multiaddr,
    ) -> Result<THandler<Self>, ConnectionDenied> {
        self.kademlia.handle_established_inbound_connection(
            connection_id,
            peer,
            local_addr,
            remote_addr,
        )
    }

    fn handle_established_outbound_connection(
        &mut self,
        connection_id: ConnectionId,
        peer: PeerId,
        addr: &Multiaddr,
        role_override: Endpoint,
    ) -> Result<THandler<Self>, ConnectionDenied> {
        self.kademlia.handle_established_outbound_connection(
            connection_id,
            peer,
            addr,
            role_override,
        )
    }

    fn handle_pending_inbound_connection(
        &mut self,
        connection_id: ConnectionId,
        local_addr: &Multiaddr,
        remote_addr: &Multiaddr,
    ) -> Result<(), ConnectionDenied> {
        self.kademlia
            .handle_pending_inbound_connection(connection_id, local_addr, remote_addr)
    }

    fn handle_pending_outbound_connection(
        &mut self,
        connection_id: ConnectionId,
        maybe_peer: Option<PeerId>,
        addresses: &[Multiaddr],
        effective_role: Endpoint,
    ) -> Result<Vec<Multiaddr>, ConnectionDenied> {
        self.kademlia.handle_pending_outbound_connection(
            connection_id,
            maybe_peer,
            addresses,
            effective_role,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_behaviour() {
        let local_peer_id = PeerId::random();
        let mut behaviour = Behaviour::new(local_peer_id);
        assert_eq!(behaviour.discovery_interval, Duration::from_secs(5));
        assert_eq!(behaviour.kademlia.kbuckets().count(), 0);
    }

    #[test]
    fn peer_management() {
        let local_peer_id = PeerId::random();
        let mut behaviour = Behaviour::new(local_peer_id);

        // Before adding any address, the known peers should be empty.
        assert!(behaviour.known_peers().is_empty());

        // Add an address and check that it's in the known peers.
        let peer_id = PeerId::random();
        let addr: Multiaddr = "/ip4/127.0.0.1/tcp/34567".parse().unwrap();
        behaviour.add_address(&peer_id, addr.clone());
        let mut expected = HashMap::new();
        expected.insert(peer_id, vec![addr.clone()]);
        assert_eq!(behaviour.known_peers(), expected);

        // Add another address.
        let addr2: Multiaddr = "/ip4/192.168.0.98/tcp/34567".parse().unwrap();
        behaviour.add_address(&peer_id, addr2.clone());
        expected.insert(peer_id, vec![addr, addr2]);
        assert_eq!(behaviour.known_peers(), expected);

        // Remove the peer.
        behaviour.remove_peer(&peer_id);
        assert!(behaviour.known_peers().is_empty());
    }
}
