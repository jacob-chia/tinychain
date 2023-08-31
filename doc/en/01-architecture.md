- [01 | Architecture](#01--architecture)
  - [1 When you are doing architecture, what are you doing?](#1-when-you-are-doing-architecture-what-are-you-doing)
  - [2 Layering](#2-layering)
  - [3 Data Structure](#3-data-structure)
  - [4 Interface](#4-interface)
    - [4.1 HTTP JSON API](#41-http-json-api)
    - [4.2 P2P API](#42-p2p-api)
    - [4.3 Storage Interface](#43-storage-interface)
  - [5 Summary](#5-summary)

# 01 | Architecture

> - Repo: `https://github.com/jacob-chia/tinychain.git`
> - Branch: `git fetch && git switch 01-architecture`

## 1 When you are doing architecture, what are you doing?

Although agile development does not prioritize architecture, it is still essential. Neglecting design and focusing only on coding can lead to two problems:

- Spending days writing code only to find it doesn't work and have to start over.
- Difficulty augmenting or refactoring functionality due to poorly written code.

Therefore, it is critical to have a minimal but essential architecture in place before you start coding. The architecture design for this project focuses on three key areas:

1. **Layering** definition, including

- Layering based on responsibilities.
- Identifying the relationships between layers. Note that the business layer should not depend on the network or data layer. This can make the core business logic easier to test and more stable.

2. **Data structure** definition, including

- Designing the structures that will be exchanged with external parties (i.e., users or peers).
- Designing the structures that will be stored locally.

3. **Interface** definition, including

- Designing the interfaces that interact with external parties (i.e., users or peers).
- Designing the interfaces that interact with local storage.

## 2 Layering

![](../img/01-architecture.png)

In Rust, you can use a workspace to organize multiple crates within a project. This approach has several benefits:

- `Higher compilation efficiency`, since only modified crates are compiled each time.
- `Clearer boundaries` between crates than layering in a single crate.

From a holistic perspective, this project is a workspace consisting of three crates: `tinychain`, `tinyp2p`, and `wallet`.

- `tinychain`: the core business component.
- `tinyp2p`: the tinychain-specific p2p protocol based on rust-libp2p.
- `wallet`: the user private key management component.

Let's dive into the tinychain, which is divided into three layers according to their responsibilities:

- `Network Layer`: responsible for interacting with the outside world, including processing HTTP requests and interacting with other peers.
- ⭐`Biz Layer`: based on the principle of **dependency inversion**, it defines the behavior (traits) of the network and data layers, removing the dependency on them.
  - `PeerClient trait`: the network must implement this trait to send data to other nodes.
  - `State trait`: the data must implement this trait to store the local state.
- `Data layer`: responsible for storing the state.

## 3 Data Structure

The data structure is defined in the `schema` on the far right in the figure above. For large projects, data may be divided into `Data Transfer Objects (DTO)`, `Domain Objects (DO)`, and `Persistent Objects (PO)`, requiring type conversions when data is passed between layers. For tinychain, however, this complexity is unnecessary.

Encoding/decoding do not require readability for data storage or p2p node interaction, so `protobuf` is used for higher performance and smaller data size. However, the `JSON` format is used for interaction between nodes and users, allowing easy viewing of request/response data during demonstrations.

The two core data structures are `Block` and `SignedTx`, both defined in protobuf format.

```proto
// Schema definition for tinychain.

syntax = "proto3";

package v1;

message Block {
 BlockHeader header = 1;
 repeated SignedTx txs = 2;
}

message BlockHeader {
 bytes parent_hash = 1;
 uint64 number = 2;
 uint64 nonce = 3;
 uint64 timestamp = 4;
 string author = 5;
}

message SignedTx {
 Tx tx = 1;
 bytes sig = 2;
}

message Tx {
 string from = 1;
 string to = 2;
 uint64 value = 3;
 uint64 nonce = 4;
 uint64 gas = 5;
 uint64 gas_price = 6;
 uint64 timestamp = 7;
}
```

In addition, unlike HTTP, P2P node communication does not use a URL and methods are conveyed within messages. Two methods need to be implemented:

- `Method::HEIGHT`: Getting the block height of a peer to determin if it is the 'best peer'.
- `Method::BLOCKS`: Retrieving blocks from the 'best peer' to synchronize with.

```proto
// Request/response methods.
enum Method {
 HEIGHT = 0;
 BLOCKS = 1;
}

message Request {
 Method method = 1;
 oneof body {
  BlockHeightReq block_height_req = 2;
  BlocksReq blocks_req = 3;
 }
}

message Response {
 Method method = 1;
 oneof body {
  BlockHeightResp block_height_resp = 2;
  BlocksResp blocks_resp = 3;
 }
}

message BlockHeightReq {}

message BlockHeightResp {
 uint64 block_height = 1;
}

message BlocksReq {
 // Start with given block number.
 uint64 from_number = 2;
}

message BlocksResp {
 repeated Block blocks = 1;
}
```

## 4 Interface

### 4.1 HTTP JSON API

| METHOD | URL                             | BODY                                                                      | DESCRIPTION                            |
| ------ | ------------------------------- | ------------------------------------------------------------------------- | -------------------------------------- |
| GET    | `/blocks?from_number=<number>`  | None                                                                      | Get blocks starting from given number. |
| GET    | `/blocks/<number>`              | None                                                                      | Get block with given number.           |
| GET    | `/balances`                     | None                                                                      | Get balances of all accounts.          |
| GET    | `/account/nonce?account=<addr>` | None                                                                      | Get nonce of given account.            |
| POST   | `/transfer`                     | `{"from": "<alice-addr>", "to": "<bob-addr>", "value": 5000, "nonce": 0}` | Send a transfer transaction.           |

Note that the `nonce` field in the `/transfer` interface is maintained by the backend for each account. Starting from 0 and increasing by 1 for each transaction, it is used to prevent `replay attacks`.

### 4.2 P2P API

For a P2P node, it is both a `client` that needs to send requests to other peers, and a `server` that needs to process requests/broadcasts from other peers. The server side needs to implement the P2P protocol which is a big topic and will be discussed later. For now, let's focus on the client side. The behavior of a p2p client is defined by the `PeerClient` trait in the biz layer as follows:

```rs
pub trait PeerClient: Debug + Clone + Send + Sync + 'static {
    /// Return the peers (base58 encoded peer ids) that this node knows about.
    fn known_peers(&self) -> Vec<String>;

    /// Get the block height from a peer.
    fn get_block_height(&self, peer_id: &str) -> Result<u64, Error>;

    /// Get blocks from a peer, starting from the `from_number`.
    fn get_blocks(&self, peer_id: &str, from_number: u64) -> Result<Vec<Block>, Error>;

    /// Broadcast a transaction to the network.
    fn broadcast_tx(&self, tx: SignedTx);

    /// Broadcast a block to the network.
    fn broadcast_block(&self, block: Block);
}
```

### 4.3 Storage Interface

The behavior of the data layer is defined by the `State` trait in the biz layer as follows:

```rs
pub trait State: Debug + Clone + Send + Sync + 'static {
    /// Current block height.
    fn block_height(&self) -> u64;

    /// Next account nonce to be used.
    fn next_account_nonce(&self, account: &str) -> u64;

    /// Get the last block hash.
    fn last_block_hash(&self) -> Option<Hash>;

    /// Add a block to the state.
    fn add_block(&self, block: Block) -> Result<(), Error>;

    /// Get blocks, starting from the `from_number`.
    fn get_blocks(&self, from_number: u64) -> Vec<Block>;

    /// Get a block by its number.
    fn get_block(&self, number: u64) -> Option<Block>;

    /// Get the balance of the account.
    fn get_balance(&self, account: &str) -> u64;

    /// Get all the balances.
    fn get_balances(&self) -> HashMap<String, u64>;

    /// Get all the nonces of the accounts.
    fn get_account2nonce(&self) -> HashMap<String, u64>;
}
```

## 5 Summary

We have a whole picture of the project by `separating responsibilities` -> `defining data structures` -> `defining interfaces`. In addition, we have split a requirement into multiple sub-problems (each crate, each layer is a sub-problem). Next, we only need to divide and conquer the sub-problems.

---

| [< 00-Overview](../../README.md) | [02-Initialization: Pre-commit Hooks & Github Action >](./02-init-project.md) |
| -------------------------------- | ----------------------------------------------------------------------------- |
