use std::sync::Arc;

use axum::Server;
use log::info;

use crate::node::{Node, Peer, State};

mod router;

pub async fn run<S, P>(node: Arc<Node<S, P>>)
where
    S: State + Send + Sync + 'static,
    P: Peer + Send + Sync + 'static,
{
    let addr = node.addr.parse().unwrap();
    info!("Listening on {addr} ====================");

    let app = router::new_router(node);

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
