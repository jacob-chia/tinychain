use std::time::Duration;

use crossbeam_channel::{tick, Sender};
use log::{debug, error};

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
            // 遍历self.peers，在遍历过程中需要修改 self.peers (添加新发现的peers，删除通信出错的peers)
            // 这种操作会引发死锁 （https://docs.rs/dashmap/latest/dashmap/struct.DashMap.html#method.remove）
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
        debug!("Sync from {peer_addr}, peer_status: {:?}", peer_status);
        self.sync_peers(peer_status.peers);
        self.sync_blocks(peer_status.number, peer_addr, block_sender)?;
        self.sync_pending_txs(peer_status.pending_txs, peer_addr)?;

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
