use std::time::Duration;

use crossbeam_channel::{tick, Sender};
use log::error;

use super::*;

impl<S, P> Node<S, P>
where
    S: State + Send + Sync + 'static,
    P: Peer + Send + Sync + 'static,
{
    pub fn sync(&self, block_sender: Sender<Block>) {
        let ticker = tick(Duration::from_secs(10));

        loop {
            ticker.recv().unwrap();
            for peer in &self.peers {
                let (peer_addr, connected) = (peer.key(), peer.value());
                if let Err(err) = self.sync_from_peer(peer_addr, connected, block_sender.clone()) {
                    error!("Failed to sync from {peer_addr}: {err}. Disconnect from it.",);
                    self.remove_peer(peer_addr);
                }
            }
        }
    }

    fn sync_from_peer(
        &self,
        peer_addr: &str,
        connected: &Connected,
        block_sender: Sender<Block>,
    ) -> Result<(), ChainError> {
        info!("Syncing from {} ...", peer_addr);
        if connected.0 == false {
            self.connect_to_peer(peer_addr)?;
        }

        let peer_status = self.peer_proxy.get_status(peer_addr)?;
        self.sync_peers(peer_status.peers);
        self.sync_blocks(peer_status.number, peer_addr, block_sender)?;
        self.sync_pending_txs(peer_status.pending_txs, peer_addr)?;

        Ok(())
    }

    fn connect_to_peer(&self, peer_addr: &str) -> Result<(), ChainError> {
        self.peer_proxy.ping(&self.addr, peer_addr)?;
        *self.peers.get_mut(peer_addr).unwrap() = Connected(true);

        Ok(())
    }

    fn sync_peers(&self, peers: Vec<String>) {
        for peer in peers {
            self.peers.entry(peer).or_insert(Connected(false));
        }
    }

    fn sync_blocks(
        &self,
        peer_block_number: u64,
        peer_addr: &str,
        block_sender: Sender<Block>,
    ) -> Result<(), ChainError> {
        let local_block_number = self.latest_block_number();
        if local_block_number >= peer_block_number {
            return Ok(());
        }

        let count = peer_block_number - local_block_number;
        info!("Found {} new blocks from peer {}", count, peer_addr);
        let blocks = self.peer_proxy.get_blocks(peer_addr, local_block_number)?;
        for block in blocks {
            block_sender.send(block).unwrap();
        }

        Ok(())
    }

    fn sync_pending_txs(&self, txs: Vec<SignedTx>, peer_addr: &str) -> Result<(), ChainError> {
        for tx in txs {
            self.add_pending_tx(tx, peer_addr)?;
        }

        Ok(())
    }
}
