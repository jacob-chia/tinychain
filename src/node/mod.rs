use crate::{database::*, utils};
use anyhow::Result;
use ethers_core::types::H256;
use tracing::debug;

const MINING_DIFFICULTY: usize = 3;
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
    let tx1 = Tx::builder()
        .from(miner)
        .to(ALICE)
        .value(100)
        .nonce(1)
        .build()
        .sign();

    let tx2 = Tx::builder()
        .from(miner)
        .to(BOB)
        .value(100)
        .nonce(2)
        .build()
        .sign();

    let txs = vec![tx1, tx2];
    let time = utils::unix_timestamp();

    let mut block = Block::builder()
        .parent(H256::default())
        .number(0)
        .nonce(1)
        .time(time)
        .miner(miner)
        .txs(txs)
        .build();

    debug!("block: {:?}", block);
    block.update_nonce(2);
    debug!("block: {:?}", block);

    state.add_block(block)?;

    Ok(())
}

fn print_state(state: &State) {
    debug!("balances: {:?}", state.get_balances());
    debug!("next_block_number: {}", state.next_block_number());
    debug!("next_account_nonce: {}", state.next_account_nonce(ALICE));
    debug!("latest_block: {:?}", state.latest_block());
    debug!("latest_block_hash: {:?}", state.latest_block_hash());
}
