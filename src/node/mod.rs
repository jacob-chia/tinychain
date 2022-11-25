use crate::{database::*, utils};
use anyhow::Result;
use log::debug;

// 挖矿计算难度
const MINING_DIFFICULTY: usize = 3;
// 演示账户
const TREASURY: &'static str = "2bde5a91-6411-46ba-9173-c3e075d32100";
const ALICE: &'static str = "3d211869-2505-4394-bd99-0c76eb761bf9";
const BOB: &'static str = "16d5e01e-709a-4536-a4f2-9f069070c51a";

pub fn run(_ip: &str, _port: u16, miner: &str) -> Result<()> {
    let mut state = State::new(MINING_DIFFICULTY)?;
    print_state(&state);

    air_drops(&mut state, miner)?;
    print_state(&state);

    Ok(())
}

fn air_drops(state: &mut State, miner: &str) -> Result<()> {
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
    state.add_block(block)?;

    Ok(())
}

fn print_state(state: &State) {
    debug!("=========================================");
    debug!("balances: {:?}", state.get_balances());
    debug!("latest_block: {:?}", state.latest_block());
    debug!("latest_block_hash: {:?}", state.latest_block_hash());
}
