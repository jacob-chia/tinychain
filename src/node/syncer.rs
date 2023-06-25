//! Periodically sync blocks from the best peer.

use std::time::Duration;

use crossbeam_channel::tick;

use super::*;

const SYNC_INTERVAL: u64 = 29;

impl<S: State, P: Peer> NodeInner<S, P> {
    /// Sync blocks from the best peer.
    pub fn sync(&self) {
        let ticker = tick(Duration::from_secs(SYNC_INTERVAL));

        loop {
            ticker.recv().unwrap();

            let local_height = self.block_height();
            let best_peer = self.get_best_peer(local_height);
            if best_peer.is_none() {
                continue;
            }
            let best_peer = best_peer.unwrap();

            let _ = self
                .peer_proxy
                .get_blocks(&best_peer, local_height)
                .map(|blocks| {
                    for block in blocks {
                        self.add_block_stop_mining(block);
                    }
                });
        }
    }

    /// Add a block and stop the current mining process.
    pub fn add_block_stop_mining(&self, block: Block) {
        if self.add_block(block).is_ok() {
            self.cancel_signal_s.send(()).unwrap();
        }
    }

    fn get_best_peer(&self, local_height: u64) -> Option<String> {
        let (mut best_peer, mut best_height) = (None, local_height);
        let peers = self.peer_proxy.known_peers();

        for peer in peers {
            let _ = self.peer_proxy.get_block_height(&peer).map(|height| {
                if best_height < height {
                    best_height = height;
                    best_peer = Some(peer);
                }
            });
        }

        best_peer
    }
}
