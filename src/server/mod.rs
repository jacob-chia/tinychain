use std::sync::Arc;

use axum::Server;
use log::info;

use crate::node::{Node, Peer, State};

mod router;

pub fn run<S, P>(node: Arc<Node<S, P>>)
where
    S: State + Send + Sync + 'static,
    P: Peer + Send + Sync + 'static,
{
    let addr = node.addr.parse().unwrap();
    info!("HTTP server listening on {addr} ====================");

    // 创建异步运行时处理 I/O 密集型任务
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(async {
        let app = router::new_router(node);

        Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    });
}
