use std::time::Duration;

use crossbeam_channel::{tick, Sender};
use log::error;

use super::*;

const SYNC_INTERVAL: u64 = 30;

impl<S, P> Node<S, P>
where
    S: State + Send + Sync + 'static,
    P: Peer + Send + Sync + 'static,
{
    pub fn sync(&self, block_sender: Sender<Block>) {
        info!("Syncer is running");
        let ticker = tick(Duration::from_secs(SYNC_INTERVAL));

        loop {
            ticker.recv().unwrap();
            // 遍历self.peers时会加“读锁”，在遍历过程中需要修改 self.peers，会尝试加“写锁”，此时会出现死锁
            // 所以先克隆一份，在 peers_clone 上遍历，在 self.peers 上修改，这样就不会死锁了。
            let peers = self.peers.clone();

            for (peer_addr, connected) in peers {
                if let Err(err) = self.sync_from_peer(&peer_addr, connected, block_sender.clone()) {
                    error!("{err}. Disconnect from {peer_addr}");
                    self.remove_peer(&peer_addr);
                }
            }
        }
    }

    fn sync_from_peer(
        &self,
        peer_addr: &str,
        connected: Connected,
        block_sender: Sender<Block>,
    ) -> Result<(), ChainError> {
        if self.addr == peer_addr {
            return Ok(());
        }
        if connected.0 == false {
            self.connect_to_peer(peer_addr)?;
        }

        let peer_status = self.peer_proxy.get_status(peer_addr)?;
        self.sync_peers(peer_status.peers);

        if let Some(count) = self.height_difference(peer_status.number, peer_status.hash) {
            info!("Found {} new blocks from {}", count, peer_addr);

            self.sync_blocks(peer_addr, block_sender)?;
            self.sync_pending_txs(peer_status.pending_txs, peer_addr)?;
        }

        Ok(())
    }

    fn connect_to_peer(&self, peer_addr: &str) -> Result<(), ChainError> {
        self.peer_proxy.ping(&self.addr, peer_addr)?;
        *self.peers.get_mut(peer_addr).unwrap() = Connected(true);
        info!("Connected to {peer_addr}");

        Ok(())
    }

    fn sync_peers(&self, peers: Vec<String>) {
        for peer in peers {
            self.peers.entry(peer).or_insert(Connected(false));
        }
    }

    fn height_difference(&self, peer_number: u64, peer_hash: Hash) -> Option<u64> {
        let local_number = self.latest_block_number();
        if peer_hash.is_zero() || local_number > peer_number {
            return None;
        }
        if self.latest_block_hash().is_zero() {
            return Some(peer_number + 1);
        }

        let count = peer_number - local_number;
        if count > 0 {
            return Some(count);
        }

        None
    }

    fn sync_blocks(&self, peer_addr: &str, block_sender: Sender<Block>) -> Result<(), ChainError> {
        let offset = self.latest_block_number();

        let blocks = self.peer_proxy.get_blocks(peer_addr, offset)?;
        for block in blocks {
            block_sender.send(block).unwrap();
        }

        Ok(())
    }

    fn sync_pending_txs(&self, txs: Vec<SignedTx>, peer_addr: &str) -> Result<(), ChainError> {
        if !txs.is_empty() {
            info!("Found new pending_txs from {}: {:?}", peer_addr, txs);
        }

        for tx in txs {
            self.add_pending_tx(tx, peer_addr)?;
        }

        Ok(())
    }
}
