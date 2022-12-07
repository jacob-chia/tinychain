use log::info;
use std::net::SocketAddr;
use tokio::signal;

mod error;
mod router;
mod temp;

use error::*;

// 挖矿计算难度
const MINING_DIFFICULTY: usize = 3;

pub async fn run(ip: &str, port: u16, miner: &str) -> Result<(), NodeError> {
    temp::temp(miner);

    let app = router::new_router();

    let addr = format!("{ip}:{port}");
    let socket_addr: SocketAddr = addr.parse()?;
    info!("Listening on {}", socket_addr);
    axum::Server::bind(&socket_addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    Ok(())
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
