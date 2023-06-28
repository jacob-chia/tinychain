- [01 | 架构设计：谋定而后动](#01--架构设计谋定而后动)
  - [1 当你在做架构时，你在做什么？](#1-当你在做架构时你在做什么)
  - [2 按职责分层](#2-按职责分层)
  - [3 数据结构](#3-数据结构)
  - [4 接口](#4-接口)
    - [4.1 HTTP JSON API](#41-http-json-api)
    - [4.2 P2P API](#42-p2p-api)
    - [4.3 存储层接口](#43-存储层接口)
  - [5 小结](#5-小结)

# 01 | 架构设计：谋定而后动

## 1 当你在做架构时，你在做什么？

虽然敏捷开发中不提倡**过度**架构，但并不等于不需要架构。如果什么都不想，上来就撸起袖子写代码，很可能出现两种结果：

- 写了两天发现路子不通，全部推到重来。
- 将来想对功能做一些扩展或重构时，发现代码像一坨\*一样改不动。

所以，本项目的架构设计只做最基本、但最重要的三件事：

1. 基于职责**分层**，包括：

- 确定层与层的`依赖关系`，只和依赖的层交互，减少不必要的依赖；
- 层间`面向接口编程`，层内的迭代不要影响到层外；
- 业务层应减少对网络和本地存储的依赖。因为核心业务逻辑在这一层，这么做可以`方便测试、提升系统稳定性`（做法详见[2-按职责分层](#2-按职责分层)）。

2. 定义**数据结构**，包括：

- 本地存储什么数据
- 与外界（用户/其他节点）交互什么数据

3. 定义**接口**，包括：

- 与外界（用户/其他节点）交互的接口
- 与本地存储交互的接口

## 2 按职责分层

![](img/01-architecture.png)

在 Rust 项目中，不仅可以分层，还支持 workspace，即一个项目中包含多个 crates，这么做的好处有两个：

- `更高的编译效率`：每次只编译修改过的 crates；
- `更清晰的边界`：尤其是在多人协作的项目中，各 crates 之间互不干涉；

从整体看，本项目由三个 crates 组成：`tinychain`、`tinyp2p`、和 `wallet`。

- `tinychain`: 一个区块链节点。
- `tinyp2p`: p2p 相关职责本应由 tinychain 的网络层负责，但 rust-libp2p 并非开箱即用且功能尚未稳定，需要大量的封装工作才能使用。为了`保持网络层的简单、稳定`，故将封装 rust-libp2p 的代码拆分成独立的 crate。这样随着 rust-libp2p 的更新，tinyp2p 的迭代不会影响到 tinychain，tinychain 甚至都不需要重新编译。
- `wallet`: wallet 本不属于区块链节点的功能，仅为了方便演示，节点可以代替用户为交易签名。所以将其拆分成独立的 crate。

接下来我们继续看`tinychain`，最右侧的`common`中是全局可见的辅助模块，暂时忽略。我们重点关注红色虚线框内的部分，该部分划分为三层：

- `网络层（network）`：负责与外界交互，包括处理 HTTP 请求和与其他 Peer 交互
- ⭐`业务层（biz）`：基于**依赖倒置**的原则，需要使用 network 和 data 提供的功能，但不依赖它们，而是让它们依赖自己。
  - `Node`: 负责核心业务逻辑
  - `trait PeerClient`: Node 需要通过 p2p 向其他节点发送数据。但 Node 不依赖网络层，而是通过`trait PeerClient`定义 p2p 的行为，然后由 p2p 实现该 trait。
  - `trait State`: Node 需要通过存储层保存本地状态，但不依赖存储层，而是通过`trait State`定义存储层的行为，由存储层实现该 trait。
- `存储层（data）`：实现 trait State 定义的接口。

## 3 数据结构

- 数据结构在上图最右侧的`schema`中定义，各层通用。对于有些大型项目，可能会把数据分为数据传输对象（DTO, Data Transfer Object）、业务领域对象（DO, Domain Object）、数据存储对象（PO, Persistant Object），然后数据在不同的层之间传递时要做类型转换，但小项目没必要整那么复杂。
- 关于`编解码`：`data`存储和`p2p`节点交互都不需要考虑可读性，所以采用性能更高、更节省空间的`protobuf`；而节点与用户的交互，为了方便演示时查看请求/响应数据，采用`JSON`格式。

- 核心数据结构只有两个：`Block`（区块）和`SignedTx`（已签名的交易），定义如下（protobuf 格式）：

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

- 另外，P2P 节点之间的请求响没有像 HTTP 那样的 URL，只能把请求方法编码到 message 中，需要实现两个方法，相当于两个 API：
  - `Method::HEIGHT`：获取对方节点的区块高度，用来判断是否是“最佳 Peer”
  - `Method::BLOCKS`：获取对方节点中从指定的 number 开始的区块列表，用来同步区块。

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

## 4 接口

### 4.1 HTTP JSON API

| METHOD | URL                             | BODY                                                                      | DESCRIPTION                            |
| ------ | ------------------------------- | ------------------------------------------------------------------------- | -------------------------------------- |
| GET    | `/blocks?from_number=<number>`  | None                                                                      | 获取从指定的 number 开始的区块列表     |
| GET    | `/blocks/<number>`              | None                                                                      | 获取指定 number 的区块                 |
| GET    | `/balances`                     | None                                                                      | 获取所有账户的余额                     |
| GET    | `/account/nonce?account=<addr>` | None                                                                      | 获取指定 addr 的 nonce，用于交易防重放 |
| POST   | `/transfer`                     | `{"from": "<alice-addr>", "to": "<bob-addr>", "value": 5000, "nonce": 0}` | 发送一笔转账交易                       |

其中需要注意的是`/transfer`接口中的`nonce`字段，由后台维护每个账户的 nonce，从 0 开始每笔交易递增 1，每个 nonce 只能使用一次，用于防止`重放攻击`。

### 4.2 P2P API

对于一个 P2P 节点来说，它既是`客户端`，需要向其他节点发送请求；又是`服务端`，需要处理来自其他节点的请求。所以我们分两部分来定义。

1. P2P 作为客户端，在 Node 节点中使用。由业务层通过 `trait PeerClient` 来定义 P2P 的行为，trait 定义如下：

```rs
/// PeerClient is a trait that defines the behaviour of a peer client.
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

- 这里需要注意的是，该 trait 不应依赖任何外部类型，比如`known_peers`需要返回节点的`PeerID`，这是 libp2p 的类型，但业务层不应该知道`libp2p`，所以这个接口返回的是 String 类型的 PeerID。

2. P2P 作为服务端，需要处理来自其他节点的数据，这部分功能由 网络层 + tinyp2p 实现。我们现阶段不需要考虑内部细节，只需要知道，tinyp2p 自己不处理业务数据，而是通过注册一系列事件处理函数 `event_handlers`，当收到其他节点的数据时，调用这些 `event_handlers` 来处理。这些 event_handlers 由 tinyp2p 定义在`trait EventHandler`中，将来由网络层的`p2p`模块实现：

```rs
/// `EventHandler` is the trait that defines how to handle requests / broadcast-messages from remote peers.
pub trait EventHandler: Debug + Send + 'static {
    /// Handles an inbound request from a remote peer.
    fn handle_inbound_request(&self, request: Vec<u8>) -> Result<Vec<u8>, P2pError>;

    /// Handles an broadcast message from a remote peer.
    fn handle_broadcast(&self, topic: &str, message: Vec<u8>);
}
```

### 4.3 存储层接口

正如上文所说，存储层的行为由业务层定义在`trait State`中：

```rs
/// State is a trait that defines the behaviour of a state.
pub trait State: Debug + Clone + Send + Sync + 'static {
    /// Current block height.
    fn block_height(&self) -> u64;

    /// Next account nonce to be used.
    fn next_account_nonce(&self, account: &str) -> u64;

    /// Get the last block.
    fn last_block(&self) -> Option<Block>;

    /// Add a block to the state.
    fn add_block(&self, block: Block) -> Result<Hash, Error>;

    /// Get blocks, starting from the `from_number`.
    fn get_blocks(&self, from_number: u64) -> Vec<Block>;

    /// Get a block by its number.
    fn get_block(&self, number: u64) -> Option<Block>;

    /// Get the balance of the account.
    fn get_balance(&self, account: &str) -> u64;

    /// Get all the balances for debugging.
    fn get_balances(&self) -> HashMap<String, u64>;
}
```

## 5 小结

我们通过 `按职责分层` -> 定义`数据结构` -> 定义`接口`这一系列操作，不仅对项目有了整体认识，而且把一个需求拆分成了多个子问题（每个 crate，每个分层都是一个子问题），接下来只需要对子问题分而治之、各个击破就好。

另外，如果在具体实现时没有清晰的思路，可以以上文定义的`接口`为粒度，画一画每个接口的`时序图`，即数据在各层之间是如何流转和处理的。画完时序图之后，各层所需要实现的接口就定义出来了。这部分内容留给感兴趣的同学自行完成，本课程不会涉及时序图的内容。

---

| [< 00-前言](../README.md) | [02-项目初始化：Pre-commit hooks 与 CI/CD >](./02-init-project.md) |
| ------------------------- | ------------------------------------------------------------------ |
