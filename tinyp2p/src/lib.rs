//! A tiny P2P library that has limited functionality but is easy to use.
//!
//! See the [examples](../examples/) directory for usage.

pub mod config;
pub mod error;

mod behaviour;
mod service;
mod transport;

pub use behaviour::Topic;
pub use config::*;
pub use error::Error;
pub use service::{new, new_secret_key, Client, OutEvent, Server};

// Re-export libp2p types.
pub use libp2p::{request_response::RequestId, Multiaddr, PeerId};
