use crossbeam_channel::{select, tick};

use super::*;

impl<S, P> Node<S, P>
where
    S: State + Send + Sync + 'static,
    P: Peer + Send + Sync + 'static,
{
    pub fn sync(&self) {
        let ticker = tick(Duration::from_secs(10));

        loop {
            select! {
                recv(ticker) -> _ => {
                }
            }
        }
    }
}
