use std::time::Duration;

use crossbeam_channel::{tick, Sender};

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
            self.peers.iter().for_each(|ref peer| {
                if self.sync_from_peer(peer.key(), peer.value()).is_err() {
                    self.remove_peer(peer.key());
                }
            });
        }
    }

    fn sync_from_peer(&self, peer_addr: &str, connected: &Connected) -> Result<(), ChainError> {
        info!("Syncing from {} ...", peer_addr);
        if connected.0 == false {
            self.connect_to_peer(peer_addr)?;
        }

        Ok(())
    }

    fn connect_to_peer(&self, peer_addr: &str) -> Result<(), ChainError> {
        self.peer_proxy.ping(&self.addr, peer_addr)?;
        *self.peers.get_mut(peer_addr).unwrap() = Connected(true);

        Ok(())
    }
}
