use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use log::{debug, info};
use std::net::SocketAddr;

use crate::{database::*, utils};

// 挖矿计算难度
const MINING_DIFFICULTY: usize = 3;

pub async fn run(_ip: &str, _port: u16, miner: &str) {
    temp(miner);

    let app = Router::new().route("/", get(hello));
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn hello() -> Result<(), NodeError> {
    Ok(())
}

// Make our own error that wraps `anyhow::Error`.
struct NodeError(anyhow::Error);

// Tell axum how to convert `NodeError` into a response.
impl IntoResponse for NodeError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, NodeError>`. That way you don't need to do that manually.
impl<E> From<E> for NodeError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

// 演示账户
const TREASURY: &'static str = "2bde5a91-6411-46ba-9173-c3e075d32100";
const ALICE: &'static str = "3d211869-2505-4394-bd99-0c76eb761bf9";
const BOB: &'static str = "16d5e01e-709a-4536-a4f2-9f069070c51a";

fn temp(miner: &str) {
    let mut state = State::new(MINING_DIFFICULTY).unwrap();

    debug!("Accounts =========================================");
    debug!("TREASURY: {}", TREASURY);
    debug!("ALICE   : {}", ALICE);
    debug!("BOB     : {}", BOB);
    debug!("MINER   : {}", miner);

    print_state(&state);
    airdrops(&mut state, miner);
    print_state(&state);
}

fn airdrops(state: &mut State, miner: &str) {
    debug!("Airdrops =========================================");
    debug!("TREASURY -> ALICE: 100");
    debug!("TREASURY -> BOB  : 100");

    let next_nonce = state.next_account_nonce(TREASURY);
    let tx1 = Tx::builder()
        .from(TREASURY)
        .to(ALICE)
        .value(100)
        .nonce(next_nonce)
        .build()
        .sign();

    let tx2 = Tx::builder()
        .from(TREASURY)
        .to(BOB)
        .value(100)
        .nonce(next_nonce + 1)
        .build()
        .sign();

    let txs = vec![tx1, tx2];
    let time = utils::unix_timestamp();

    let parent = state.latest_block_hash().to_owned();
    let block_number = state.next_block_number();
    let mut block = Block::builder()
        .parent(parent)
        .number(block_number)
        .nonce(1)
        .time(time)
        .miner(miner)
        .txs(txs)
        .build();

    // 需要不断update_nonce -> 计算hash -> 直到hash满足要求
    block.update_nonce(2);
    state.add_block(block).unwrap();
}

fn print_state(state: &State) {
    debug!("Current state =========================================");
    debug!("balances         : {:?}", state.get_balances());
    debug!("latest_block     : {:?}", state.latest_block());
    debug!("latest_block_hash: {:?}", state.latest_block_hash());
}
