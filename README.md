# Build A Blockchain with Rust

[English](README.md) | [简体中文](README_ZH.md)

## Table of Contents

- [00 | Overview](README.md)
- [01 | Architecture](doc/en/01-architecture.md)
- [02 | Initialization: Pre-commit Hooks & Github Action](doc/en/02-init-project.md)
- [03 | Defining Data Structure & API](doc/en/03-data-structure-api.md)
- [04 | Wallet: Sign & Verify](doc/en/04-wallet.md)
- [05 | Command Line & Config File](doc/en/05-cmd-config.md)
- [06 | Thinking in Libp2p](doc/en/06-libp2p.md)
- [07 | Tinyp2p: A CSP Concurrency Model](doc/en/07-tinyp2p.md)
- [08 | Network Layer](doc/en/08-network.md)
- [09 | Biz Layer: How to Do Read/Write Separation?](doc/en/09-biz.md)
- [10 | Data Layer & Demo](doc/en/10-data.md)

## Intro

This project aims to demonstrate the basic principles of blockchain through a `distributed ledger`. The main features include:

- `HTTP JSON API` provides users with interfaces such as `transfer` and some `query` apis;
- `P2P Protocol` is used for interaction between nodes, and data is serialized/deserialized by `protobuf`. The functions include `peer discovery`, `transaction broadcast`, `block broadcast`, and `block synchronization`;
- `PoW` is used as the consensus mechanism;
- `Sled`, an embedded key-value database, is used as the storage backend;
- For the convenience of demonstration, there is a `wallet` in each node that stores the users' private keys, so that the node can sign the transaction on behalf of users.

## Architecture

> See [01 | Architecture](doc/en/01-architecture.md) for details.

![](doc/img/01-architecture.png)

From a holistic perspective, this project is a workspace, consisting of three crates: `tinychain`, `tinyp2p`, and `wallet`.

- `tinychain`: core business.
- `tinyp2p`: a tinychain-specific p2p protocol based on rust-libp2p.
- `wallet`: user private key management.

### tinychain | Dependency Inversion

> See [01 | Architecture](doc/en/01-architecture.md) for details.

In `tinychain`, it is divided into three layers according to responsibilities:

- `Network Layer`: responsible for interacting with the outside world, including processing HTTP requests and interacting with other peers.
- ⭐`Biz Layer`: based on the principle of **Dependency Inversion**, it defines the behavior (traits) of the network and data layers, getting rid of the dependency on them.
  - `trait PeerClient`: the `network` needs to implement this trait to send data to other nodes.
  - `trait State`: the `data` needs to implement this trait to save the local state.
- `Data Layer`: responsible for saving the state.

### tinychain::biz | Read/Write Separation

> See [09 | Biz Layer: How to Do Read/Write Separation?](doc/en/09-biz.md) for details.

![](doc/img/09-biz.png)

The `biz` layer achieves lock-free programming through read/write separation. That is to say, any thread can "read", but only one thread can "write". In this project, there are two main write operations: (1) Adding user transfer data to the transaction pool; (2) Adding blocks to the database. From the above figure, only the `Miner` thread has write permission. When other threads need to write, they send the data to the `Miner` to write via the channel.

### tinyp2p | CSP Concurrency Model

> See [07 | tinyp2p: A CSP Concurrency Model](doc/en/07-tinyp2p.md) for details.

![](doc/img/07-csp.png)

- `p2p_client` is used to process user requests. In `p2p_client`, the request is converted to `cmd` and sent to the channel.
- A background thread exclusively owns `mut p2p_server`, and gets `cmd` from the channel one by one to execute.
- Users can register `event_handlers` in p2p_server. When data is received from a remote node, `event_handlers` are called to process the data.

## Demo

> See [10 | Data Layer & Demo](doc/en/10-data.md) for details.

1. View the commands: `RUST_LOG=info ./target/debug/tinychain`：

   ![](doc/img/05-cmd-help.png)

2. Create an account: `RUST_LOG=info ./target/debug/tinychain new-account`：

   ![](doc/img/05-cmd-new-account.png)

3. Query account balance and block information

   ![](doc/img/10-block-state.png)
