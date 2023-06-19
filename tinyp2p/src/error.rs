use std::io;

use libp2p::{gossipsub, multiaddr, request_response, swarm, TransportError};
use tokio::sync::oneshot;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid secret key: {0}")]
    InvalidSecretKey(String),
    #[error("Invalid address")]
    InvalidAddress(#[from] multiaddr::Error),
    #[error("Invalid peer ID")]
    InvalidPeerId,
    #[error(transparent)]
    DialError(#[from] swarm::DialError),
    #[error(transparent)]
    ListenError(#[from] TransportError<io::Error>),
    #[error("The remote peer rejected the request")]
    RequestRejected,
    #[error("Problem on the network: {0}")]
    NetworkError(request_response::OutboundFailure),
    #[error(transparent)]
    ChanError(#[from] oneshot::error::RecvError),
    #[error("Failed to build pub/sub behaviour: {0}")]
    PubsubBuildError(String),
    #[error(transparent)]
    SubscribeError(#[from] gossipsub::SubscriptionError),
    #[error(transparent)]
    PublishError(#[from] gossipsub::PublishError),
}
