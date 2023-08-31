- [10 | Data Layer \& Demo](#10--data-layer--demo)
  - [1 Sled State](#1-sled-state)
  - [2 Demo](#2-demo)
    - [2.1 Create Demo Accounts](#21-create-demo-accounts)
    - [2.2 Prepare Data](#22-prepare-data)
    - [2.3 Start Three Nodes](#23-start-three-nodes)
    - [2.4 Init State of Blockchain](#24-init-state-of-blockchain)
    - [2.5 Value Transfer](#25-value-transfer)
    - [2.6 Check Balances and Blocks](#26-check-balances-and-blocks)
  - [3 Summary](#3-summary)

# 10 | Data Layer & Demo

> This is a hands-on tutorial, so please switch to the corresponding code branch before reading.
>
> - Repo: `https://github.com/jacob-chia/tinychain.git`
> - Branch：`git fetch && git switch 10-data`
>
> Important crates used in this lesson:
>
> - [sled](https://docs.rs/sled/latest/sled/): a high-performance embedded KV database.

## 1 Sled State

The trait `State` has been defined in the biz layer. Now let's implement it based on `sled`. The source code is always up to date, just jump to [src/data/sled_state.rs](../../src/data/sled_state.rs) for details.

There's just one thing to note: `all keys and values in sled are bytes, even if it's a number`. Our structs can be encoded into bytes via protobuf, but we need to implement u64 encoding/decoding functions ourselves. The code is as follows:

```rs
// src/data/sled_state.rs

fn u64_decode(bytes: &[u8]) -> u64 {
    let mut buf = [0u8; 8];
    buf.copy_from_slice(bytes);
    // Note that big-endian is used so that u64 can be sorted normally
    u64::from_be_bytes(buf)
}

fn u64_encode(n: u64) -> Vec<u8> {
    n.to_be_bytes().to_vec()
}
```

## 2 Demo

### 2.1 Create Demo Accounts

Create 4 accounts with `RUST_LOG=INFO ./target/debug/tinychain new-account`, and note down the addresses displayed in the log. You need to replace all the addresses below with the 4 addresses you created:

- `Treasury`: "0x05c8b9c7d38dc0b0883bc9b7a2952c15899ff07e"
- `Alice`: "0x8d1cbb757610619d74fdca9ee008a007a633a71e"
- `Bob`: "0x707980eaa14b678c3d586a8d62d68bdac752d7d5"
- `Emma`: "0x0bbdab8c4908d1bf58ca21d1316dd604dbad0197"

### 2.2 Prepare Data

1. Modify `genesis.json` in the root directory, which is the initial account balance. Replace the Treasury address in the file with the address of the Treasury above `0x05c8b9c7d38dc0b0883bc9b7a2952c15899ff07e`;
2. Copy `db/` to `db1/` and `db2/`;
3. Modify the configuration file `doc/conf/*.toml` and change the address of the miner `author` to the address of Alice, Bob, and Emma above.

### 2.3 Start Three Nodes

In three terminals, go to the project root directory and execute the following commands to start the nodes.

- Terminal 1, miner Alice: `RUST_LOG=INFO ./target/debug/tinychain run -c ./doc/conf/config-boot.toml`
- Terminal 2, miner Bob: `RUST_LOG=INFO ./target/debug/tinychain run -c ./doc/conf/config1.toml`
- Terminal 3, miner Emma: `RUST_LOG=INFO ./target/debug/tinychain run -c ./doc/conf/config2.toml`

After starting, you can observe the log. The three nodes already know each other; and regularly check whether their block height is behind other nodes:

```log
INFO  tinychain::network::p2p  > 📣 Known peers ["12D3KooWNEG4GYu9pHdv9TcGAHvSgK82a5XsKdh9U96zxXRnFzL6", "12D3KooWSoC2ngFnfgSZcyJibKmZ2G58kbFcpmSPSSvDxeqkBLJc"]
INFO  tinychain::network::p2p  > 📣 >> [OUT] get_block_height from: 12D3KooWNEG4GYu9pHdv9TcGAHvSgK82a5XsKdh9U96zxXRnFzL6
INFO  tinychain::network::p2p  > 📣 << [IN] get_block_height response: Response { method: Height, body: Some(BlockHeightResp(BlockHeightResp { block_height: 0 })) }
INFO  tinychain::network::p2p  > 📣 >> [OUT] get_block_height from: 12D3KooWSoC2ngFnfgSZcyJibKmZ2G58kbFcpmSPSSvDxeqkBLJc
INFO  tinychain::network::p2p  > 📣 << [IN] get_block_height response: Response { method: Height, body: Some(BlockHeightResp(BlockHeightResp { block_height: 0 })) }
```

### 2.4 Init State of Blockchain

```sh
# All the balances
curl http://localhost:8000/balances | jq
# Block information
curl http://localhost:8000/blocks?from_number=0 | jq
```

The query results are as follows:

![](../img/10-genesis-state.png)

### 2.5 Value Transfer

1. Send two identical transactions and check the log

```sh
# First query the next nonce of Treasury, the nonce of each Tx needs to be increased by one
curl -X GET http://localhost:8002/account/nonce?account=0x05c8b9c7d38dc0b0883bc9b7a2952c15899ff07e

# Treasury -> Alice: 5000
curl -X POST http://localhost:8002/transfer \
  -H 'Content-Type: application/json' \
  -d '{"from": "0x05c8b9c7d38dc0b0883bc9b7a2952c15899ff07e", "to": "0x8d1cbb757610619d74fdca9ee008a007a633a71e", "value": 5000, "nonce": 0}'

# Treasury -> Alice: 5000
curl -X POST http://localhost:8002/transfer \
  -H 'Content-Type: application/json' \
  -d '{"from": "0x05c8b9c7d38dc0b0883bc9b7a2952c15899ff07e", "to": "0x8d1cbb757610619d74fdca9ee008a007a633a71e", "value": 5000, "nonce": 0}'
```

We can find the following from the log. The first tx was broadcast to other nodes, and the second tx failed with a `InvalidTxNonce` error.

```log
INFO  tinychain::network::http > 📣 >> transfer: TxReq { from: "0x05c8b9c7d38dc0b0883bc9b7a2952c15899ff07e", to: "0x8d1cbb757610619d74fdca9ee008a007a633a71e", value: 5000, nonce: 0 }
INFO  tinychain::network::http > 📣 << transfer response: Ok(())
INFO  tinychain::network::p2p  > 📣 >> [OUT-BROADCAST] tx: SignedTx{ tx: Tx { from: "0x05c8b9c7d38dc0b0883bc9b7a2952c15899ff07e", to: "0x8d1cbb757610619d74fdca9ee008a007a633a71e", value: 5000, nonce: 0, gas: 21, gas_price: 1, timestamp: 1688635600 }, sig: 0x6f6981004ef7dd0142322cc3f7613ee55d88a36aa74109935c3776eed9d5b90d58378e359b85238c9a1e6b5376006d4530459d8569b464ce3c96cd881089321400 }
INFO  tinychain::network::http > 📣 >> transfer: TxReq { from: "0x05c8b9c7d38dc0b0883bc9b7a2952c15899ff07e", to: "0x8d1cbb757610619d74fdca9ee008a007a633a71e", value: 5000, nonce: 0 }
INFO  tinychain::network::http > 📣 << transfer response: Ok(())
ERROR tinychain::biz::miner    > ❌ Bad tx: InvalidTxNonce("0x05c8b9c7d38dc0b0883bc9b7a2952c15899ff07e", 1, 0)
```

Keep observing the logs of the three nodes. One of the nodes mined a block and broadcast it to other nodes:

```log
INFO  tinychain::network::p2p  > 📣 >> [P2P-IN-BROADCAST] SignedTx{ tx: Tx { from: "0x05c8b9c7d38dc0b0883bc9b7a2952c15899ff07e", to: "0x8d1cbb757610619d74fdca9ee008a007a633a71e", value: 5000, nonce: 0, gas: 21, gas_price: 1, timestamp: 1688635600 }, sig: 0x6f6981004ef7dd0142322cc3f7613ee55d88a36aa74109935c3776eed9d5b90d58378e359b85238c9a1e6b5376006d4530459d8569b464ce3c96cd881089321400 }
INFO  tinychain::biz::miner    > 📣 Mining attempt: 0, elapsed: 92.122µs
INFO  tinychain::biz::miner    > 📣 Mined new Block '0x0000546941b45d7e4e371d93ef7a0adb50b560af9b636bfaeca8b8342d316158' 🎉🎉🎉:
INFO  tinychain::biz::miner    > 📣    Number: '0'
INFO  tinychain::biz::miner    > 📣    Nonce: '6281805050525990748'
INFO  tinychain::biz::miner    > 📣    Created: '1688635617'
INFO  tinychain::biz::miner    > 📣    Miner: '0x8d1cbb757610619d74fdca9ee008a007a633a71e'
INFO  tinychain::biz::miner    > 📣    Parent: '0x0000000000000000000000000000000000000000000000000000000000000000'
INFO  tinychain::biz::miner    > 📣    Attempt: '9670'
INFO  tinychain::biz::miner    > 📣    Time: 9.554963205s
INFO  tinychain::biz::miner    > 🎉🎉🎉🎉🎉🎉🎉🎉🎉🎉🎉🎉🎉🎉🎉🎉🎉🎉

INFO  tinychain::network::p2p  > 📣 >> [OUT-BROADCAST] block: Block { header: BlockHeader { number: 0, parent_hash: 0x0000000000000000000000000000000000000000000000000000000000000000, nonce: 6281805050525990748, timestamp: 1688635617, author: "0x8d1cbb757610619d74fdca9ee008a007a633a71e" }, txs: [SignedTx{ tx: Tx { from: "0x05c8b9c7d38dc0b0883bc9b7a2952c15899ff07e", to: "0x8d1cbb757610619d74fdca9ee008a007a633a71e", value: 5000, nonce: 0, gas: 21, gas_price: 1, timestamp: 1688635600 }, sig: 0x6f6981004ef7dd0142322cc3f7613ee55d88a36aa74109935c3776eed9d5b90d58378e359b85238c9a1e6b5376006d4530459d8569b464ce3c96cd881089321400 }] }
```

The other two nodes received the block and stopped mining:

```log
INFO  tinychain::network::p2p  > 📣 >> [P2P-IN-BROADCAST] Block { header: BlockHeader { number: 0, parent_hash: 0x0000000000000000000000000000000000000000000000000000000000000000, nonce: 6281805050525990748, timestamp: 1688635617, author: "0x8d1cbb757610619d74fdca9ee008a007a633a71e" }, txs: [SignedTx{ tx: Tx { from: "0x05c8b9c7d38dc0b0883bc9b7a2952c15899ff07e", to: "0x8d1cbb757610619d74fdca9ee008a007a633a71e", value: 5000, nonce: 0, gas: 21, gas_price: 1, timestamp: 1688635600 }, sig: 0x6f6981004ef7dd0142322cc3f7613ee55d88a36aa74109935c3776eed9d5b90d58378e359b85238c9a1e6b5376006d4530459d8569b464ce3c96cd881089321400 }] }
INFO  tinychain::biz::miner    > 📣 Received a block from other peers, cancel mining.
```

2. Send a few more transactions

```sh
# First query the next nonce of Treasury, the nonce of each Tx needs to be increased by one
curl -X GET http://localhost:8002/account/nonce?account=0x05c8b9c7d38dc0b0883bc9b7a2952c15899ff07e

# Treasury -> Alice: 5000
curl -X POST http://localhost:8002/transfer \
  -H 'Content-Type: application/json' \
  -d '{"from": "0x05c8b9c7d38dc0b0883bc9b7a2952c15899ff07e", "to": "0x8d1cbb757610619d74fdca9ee008a007a633a71e", "value": 5000, "nonce": 1}'

# Treasury -> Bob: 5000
curl -X POST http://localhost:8002/transfer \
  -H 'Content-Type: application/json' \
  -d '{"from": "0x05c8b9c7d38dc0b0883bc9b7a2952c15899ff07e", "to": "0x707980eaa14b678c3d586a8d62d68bdac752d7d5", "value": 5000, "nonce": 2}'

# Treasury -> Emma: 5000
curl -X POST http://localhost:8002/transfer \
  -H 'Content-Type: application/json' \
  -d '{"from": "0x05c8b9c7d38dc0b0883bc9b7a2952c15899ff07e", "to": "0x0bbdab8c4908d1bf58ca21d1316dd604dbad0197", "value": 5000, "nonce": 3}'
```

### 2.6 Check Balances and Blocks

```sh
# All the balances
curl http://localhost:8000/balances | jq
# Blocks from number 0
curl http://localhost:8000/blocks?from_number=0 | jq
```

The results are as follows:

![](../img/10-block-state.png)

## 3 Summary

```sh
⚡ tokei .
===============================================================================
 Language            Files        Lines         Code     Comments       Blanks
===============================================================================
 JSON                    1            6            6            0            0
 Protocol Buffers        1           70           54            3           13
 TOML                    7          246          153           61           32
-------------------------------------------------------------------------------
 Markdown               11         2636            0         1965          671
 |- TOML                 1           13            6            2            5
 (Total)                           2649            6         1967          676
-------------------------------------------------------------------------------
 Rust                   36         3672         2999           85          588
 |- Markdown            27          178            0          165           13
 (Total)                           3850         2999          250          601
===============================================================================
 Total                  56         6630         3212         2114         1304
===============================================================================
```

We used 10 lessons to build a blockchain with Rust from scratch, and wrote more than 6000 lines of code (including comments) and documentation. And, we learned through this project:

- Layered architecture design
- CSP concurrency model
- How to do read/write separation in the biz layer
- A lot of Rust practical skills

Congratulations! 🎉🎉🎉

---

| [< 09-Biz Layer: How to Do Read/Write Separation?](./09-biz.md) | [README >](../../README.md) |
| --------------------------------------------------------------- | --------------------------- |
