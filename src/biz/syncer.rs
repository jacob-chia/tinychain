//! Periodically sync blocks from the best peer.

use std::time::Duration;

use crossbeam_channel::{tick, Sender};

use crate::schema::Block;

use super::{PeerClient, State};

const SYNC_INTERVAL: u64 = 29;

#[derive(Debug)]
pub struct Syncer<S: State, P: PeerClient> {
    /// The state of the blockchain.
    state: S,
    /// The client to interact with other peers.
    peer_client: P,
    /// The channel to send blocks to the miner.
    block_sender: Sender<Block>,
}

impl<S: State, P: PeerClient> Syncer<S, P> {
    pub fn new(state: S, peer_client: P, block_sender: Sender<Block>) -> Self {
        Self {
            state,
            peer_client,
            block_sender,
        }
    }

    pub fn sync(&self) {
        let ticker = tick(Duration::from_secs(SYNC_INTERVAL));

        loop {
            ticker.recv().unwrap();

            let local_height = self.state.block_height();
            let best_peer = self.get_best_peer(local_height);
            if best_peer.is_none() {
                continue;
            }
            let best_peer = best_peer.unwrap();

            let _ = self
                .peer_client
                .get_blocks(&best_peer, local_height)
                .map(|blocks| {
                    for block in blocks {
                        let _ = self.block_sender.send(block);
                    }
                });
        }
    }

    fn get_best_peer(&self, local_height: u64) -> Option<String> {
        let (mut best_peer, mut best_height) = (None, local_height);
        let peers = self.peer_client.known_peers();

        for peer in peers {
            let _ = self.peer_client.get_block_height(&peer).map(|height| {
                if best_height < height {
                    best_height = height;
                    best_peer = Some(peer);
                }
            });
        }

        best_peer
    }
}
