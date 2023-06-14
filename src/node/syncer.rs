//! Periodically sync blocks from the best peer.

use std::time::Duration;

use crossbeam_channel::tick;

use super::*;

const SYNC_INTERVAL: u64 = 29;

impl<S, P> Node<S, P>
where
    S: State + Send + Sync + 'static,
    P: Peer + Send + Sync + 'static,
{
    /// Sync blocks from the best peer.
    pub fn sync(&self) {
        let ticker = tick(Duration::from_secs(SYNC_INTERVAL));

        loop {
            ticker.recv().unwrap();

            let local_number = self.latest_block_number();
            let best_peer = self.get_best_peer(local_number);
            if best_peer.is_none() {
                continue;
            }
            let best_peer = best_peer.unwrap();

            let from_number = local_number.map(|num| num + 1).unwrap_or_default();
            let _ = self
                .peer_proxy
                .get_blocks(&best_peer, from_number)
                .map(|blocks| {
                    for block in blocks {
                        self.add_block_stop_mining(block);
                    }
                });
        }
    }

    /// Add a block and stop the current mining process.
    pub fn add_block_stop_mining(&self, block: Block) {
        if self.add_block(block) {
            self.cancel_signal_s.send(()).unwrap();
        }
    }

    fn get_best_peer(&self, local_number: Option<u64>) -> Option<String> {
        let (mut best_peer, mut best_number) = (None, local_number);
        let peers = self.peer_proxy.known_peers();

        for peer in peers {
            let _ = self.peer_proxy.get_best_number(&peer).map(|number| {
                if number.is_none() {
                    return;
                }
                if best_number.is_none() || number.unwrap() > best_number.unwrap() {
                    best_number = number;
                    best_peer = Some(peer);
                }
            });
        }

        best_peer
    }
}
