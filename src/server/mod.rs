use std::sync::Arc;

use axum::Server;
use log::info;
use tokio::signal;

use crate::node::{Node, Peer, State};

mod router;

pub async fn run<S, P>(node: Node<S, P>)
where
    S: State + Send + Sync + 'static,
    P: Peer + Send + Sync + 'static,
{
    let addr = node.addr.parse().unwrap();
    info!("Listening on {addr} ====================");

    let app = router::new_router(Arc::new(node));

    Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
