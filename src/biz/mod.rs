//! The entry point of the blockchain node.

use std::marker;

mod peer_client;
mod state;

pub use peer_client::*;
pub use state::*;

#[derive(Debug, Clone)]
pub struct Node<S: State, P: PeerClient> {
    _marker: marker::PhantomData<(S, P)>,
}

impl<S: State, P: PeerClient> Node<S, P> {
    pub fn new() -> Self {
        Self {
            _marker: marker::PhantomData,
        }
    }
}
