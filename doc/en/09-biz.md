- [09 | Biz Layer: How to Do Read/Write Separation?](#09--biz-layer-how-to-do-readwrite-separation)
  - [1 Architecture](#1-architecture)
  - [2 Why Read/Write Separation?](#2-why-readwrite-separation)
    - [2.1 User Transfer](#21-user-transfer)
    - [2.2 Adding Blocks to the Database](#22-adding-blocks-to-the-database)
  - [3 Implementation](#3-implementation)
    - [3.1 Node](#31-node)
    - [3.2 Miner](#32-miner)
    - [3.3 Syncer](#33-syncer)
  - [4 Summary](#4-summary)

# 09 | Biz Layer: How to Do Read/Write Separation?

> This is a hands-on tutorial, so please switch to the corresponding code branch before reading.
>
> - Repo: `https://github.com/jacob-chia/tinychain.git`
> - Branch：`git fetch && git switch 09-biz`
>
> Important crates used in this lesson:
>
> - [crossbeam-channel](https://docs.rs/crossbeam-channel/latest/crossbeam_channel/): Multi-producer multi-consumer channels for message passing.

In the last lesson, we identified the interfaces that the `biz::Node` needs to provide. But in addition to these interfaces, the `biz` layer also needs to implement two background tasks: mining and synchronizing blocks from other nodes. To sum up, the `biz` layer needs to implement three core modules:

- `Node`: provides interfaces to the outside world and processes data from the network (HTTP/P2P);
- `Miner`: background task, mining at regular intervals;
- `Syncer`: background task, synchronizing blocks from other nodes at regular intervals;

## 1 Architecture

![](../img/09-biz.png)

The `biz` layer adopts the architecture of "read/write separation". That is to say, `any thread can "read", but only one thread can "write"`. In this project, there are two main write operations: (1) Adding user transfer data to the transaction pool; (2) Adding blocks to the database. From the above figure, only the `Miner` thread has write permission. When other threads need to write, they send the data to the `Miner` to write via the channel, including:

- When the `Node` receives a transaction from the user (or other nodes), it sends it to the `Miner` via `tx_sender`;
- When the `Node` receives a block broadcast from other nodes, it sends it to the `Miner` via `block_sender`;
- When the `Syncer` finds that the local block height is lagging behind, it synchronizes blocks from other nodes and sends them to the `Miner` via `block_sender`;

Let's explain the benefits of doing so before we start writing code.

## 2 Why Read/Write Separation?

### 2.1 User Transfer

User transfer transactions are added to the "transaction pool", and the background `Miner` thread periodically takes data from the pool for "mining". Semantically, it is a `multi-threaded write, single-threaded read` pool. That is to say, we need to frequently get the "write lock" on the pool, and in order to verify whether a transaction is valid, we need to lock multiple fields.

We can let the transaction pool only be managed by the `Miner` thread, and all the transactions coming in are sent to the `Miner` via a channel, and the `Miner` itself is responsible for adding the transactions to the pool, avoiding the use of locks.

### 2.2 Adding Blocks to the Database

Before a block is added to the database (`add_block`), it needs to be validated (`check_block`). To minimise database operations, `check_block` should be executed before `add_block`. But there are multiple threads that need to call `add_block`:

- `Miner`: Mine blocks locally;
- `Syncer`: Synchronize blocks from other nodes when the local block height lags behind;
- `Node`: Receive blocks that broadcast from other nodes;

So in order to ensure the atomicity of `check_block` and `add_block`, we have to put `check_block` and `add_block` into the same DB transaction. This is not good, because:

- The timing of the check is a bit late, and the unqualified data should be discovered as early as possible;
- It does not conform to our layered concept, and business logic should be placed in the business layer as much as possible, keeping the storage layer simple.

A better solution is to let the `Miner` thread to be responsible for adding blocks, while other threads can send the received blocks via a channel to the `Miner` for processing. This way the `Miner` can do `check_block` in the business layer and do `add_block` in the data layer without fear of data collision.

## 3 Implementation

### 3.1 Node

The responsibility of the `Node` is to handle requests from the network, including HTTP and P2P requests. We have identified the interfaces that the `Node` needs to provide in the last lesson. The full code is in: [src/biz/node.rs](../../src/biz/node.rs). There is only one thing to explain here, let's look into the definition of the `Node` structure:

```rs
// src/biz/node.rs

#[derive(Debug, Clone)]
pub struct Node<S: State> {
    inner: Arc<NodeInner<S>>,
}

#[derive(Debug, Clone)]
pub struct NodeInner<S: State> {
    // A state machine that holds the state of the blockchain.
    state: S,
    // A channel to send a signed transaction to the miner.
    tx_sender: Sender<TxMsg>,
    // A channel to send a block to the miner.
    block_sender: Sender<Block>,

    // For facilitating a smooth demonstration, the node holds a wallet that stores all
    // the keys of the users, so that it can sign transactions on behalf of the users.
    // In the real world, every user should have their own wallet.
    wallet: Wallet,
}
```

`NodeInner` is the structure that actually provides the service, but because it needs to share references in multiple threads, it needs to be wrapped in an `Arc<>` outside. However, this "wrapping in an `Arc<>`" should not be done by the user, but by the `Node` itself, so we define another `Node` to encapsulate the complexity inside, which is a common practice in Rust code.

The `NodeInner` acts as a service provider, but needs to share references across multiple threads, requiring an `Arc<>` wrapper outside. However, this wrapping work should not be done by the end user, but by the `Node` itself. To enhance user experience, it's common practice in Rust to define an additional `InnerNode` to handle the complexity.

If you think this is redundant, I can give you a more complex example. In the `data` layer, I implemented a `MemoryState` to debug the functionality of the `biz` layer (which will be replaced by `SledState` in the future), the source code is in [src/data/memory_state.rs](../../src/data/memory_state.rs). For the `biz` layer, `MemoryState` is "ready to use" without having to worry about the internal implementation, but in fact `MemoryState` is defined as follows:

```rs
// src/data/memory_state.rs

#[derive(Debug, Clone)]
pub struct MemoryState {
    inner: Arc<RwLock<InnerState>>,
}

#[derive(Debug, Clone)]
struct InnerState {
    blocks: BTreeMap<u64, Block>,
    balances: HashMap<String, u64>,
    account2nonce: HashMap<String, u64>,
}
```

Without this "inner" pattern, we would have to write `Arc<RwLock<MemoryState>>` everywhere in the `biz` layer, which would be very cumbersome.

### 3.2 Miner

The `Miner` is responsible for:

- When receiving transactions from other threads, add them to the transaction pool `pending_txs`;
- Periodically pack transactions in `pending_txs` into a block for mining;
- When receiving blocks from other threads during mining, cancel the current mining and add the block;

After knowing the responsibilities of the `Miner`, the source code becomes much easier: [src/biz/miner.rs](../../src/biz/miner.rs). And there is one thing to explain here, let's look into the definition of the `Miner` structure:

```rs
// src/biz/miner.rs

#[derive(Debug)]
pub struct Miner<S: State, P: PeerClient> {
    /// The pending transactions that are not yet included in a block.
    pending_txs: HashMap<Hash, SignedTx>,
    /// The pending state that is used to check if a transaction is valid.
    pending_state: PendingState,
    // The state of the blockchain.
    state: S,
    // ...
}

/// `PendingState` merges the current `state` and the `pending_txs`.
/// It is used to check if an incoming transaction is valid.
#[derive(Debug, Default)]
struct PendingState {
    // The balances of all accounts.
    balances: HashMap<String, u64>,
    // The nonce of all accounts.
    account2nonce: HashMap<String, u64>,
}
```

Assuming the following scenario:

- In the database `state`: `alice`: balance = 1000, nonce = 10; `bob`: balance = 0, nonce = 0;
- In the transaction pool `pending_txs`: there is a transaction `{from: alice, to: bob, value: 900, nonce: 10}`; this is a valid transaction;
- Now there is another transaction `{from: alice, to: bob, value: 900, nonce: 10}`, how to check if this tx is valid? We must calculate the data of both `state` and `pending_txs` together.

So we define a field `pending_state` that merges the `state` and `pending_txs`. In the above scenario, the state in `pending_state` is as follows: `alice`: balance = 100, nonce = 11; `bob`: balance = 900, nonce = 0; This way we can check whether a transaction is valid or not.

Note that every time a new block is added to the database, `pending_state` needs to be reset. The relevant code is as follows:

```rs
// src/biz/miner.rs

let result = self.state.add_block(block.clone());
if result.is_ok() {
    self.remove_mined_txs(&block);
    self.reset_pending_state();
}

fn reset_pending_state(&mut self) {
    // load from `state`
    self.pending_state.balances = self.state.get_balances();
    self.pending_state.account2nonce = self.state.get_account2nonce();

    // load from `pending_txs`
    for tx in self.get_sorted_txs() {
        self.update_pending_state(&tx);
    }
}
```

### 3.3 Syncer

The `Syncer` is responsible for synchronizing blocks from other nodes. The code is as follows and nothing special: [src/biz/syncer.rs](../../src/biz/syncer.rs):

```rs
// src/biz/syncer.rs

#[derive(Debug)]
pub struct Syncer<S: State, P: PeerClient> {
    /// The state of the blockchain.
    state: S,
    /// The client to interact with other peers.
    peer_client: P,
    /// The channel to send blocks to the miner.
    block_sender: Sender<Block>,
}

impl<S: State, P: PeerClient> Syncer<S, P> {
    pub fn new(state: S, peer_client: P, block_sender: Sender<Block>) -> Self {
        Self {
            state,
            peer_client,
            block_sender,
        }
    }

    pub fn sync(&self) {
        let ticker = tick(Duration::from_secs(SYNC_INTERVAL));

        loop {
            ticker.recv().unwrap();

            let local_height = self.state.block_height();
            let best_peer = self.get_best_peer(local_height);
            if best_peer.is_none() {
                continue;
            }
            let best_peer = best_peer.unwrap();

            let _ = self
                .peer_client
                .get_blocks(&best_peer, local_height)
                .map(|blocks| {
                    for block in blocks {
                        let _ = self.block_sender.send(block);
                    }
                });
        }
    }

    fn get_best_peer(&self, local_height: u64) -> Option<String> {
        let (mut best_peer, mut best_height) = (None, local_height);
        let peers = self.peer_client.known_peers();

        for peer in peers {
            let _ = self.peer_client.get_block_height(&peer).map(|height| {
                if best_height < height {
                    best_height = height;
                    best_peer = Some(peer);
                }
            });
        }

        best_peer
    }
}
```

## 4 Summary

In this lesson we explained the benefits of having read/write separation in the business layer: it eliminates the need to use locks and reduces the load on the data layer.

---

| [< 08-Network Layer](./08-network.md) | [10-Data Layer & Demo >](./10-data.md) |
| ------------------------------------- | -------------------------------------- |
