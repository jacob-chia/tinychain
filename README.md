# tinychain

[Substrate入门课](https://appbhteffsi3308.h5.xiaoeknow.com/v1/goods/goods_detail/p_62ac1ea6e4b0ba331dc9542c?type=3&type=3) Office Hour 演示项目。

## 项目介绍

本项目旨在介绍区块链基本原理，重点关注如何`生成区块`和如何`与其他节点达成共识`，会简化其他逻辑。
- 只有`转账`功能，不包含智能合约，也不包含Substrate相关内容
- 共识机制：`PoW`
- 网络协议：使用`HTTP`模拟P2P
- 存储：`文件`存储
- 账户管理：为了简化演示流程，`所有账户密码、地址、公私钥均存储在后端`，用户只持有account_id（UUID格式）。用户发送Tx（即Transaction）时不需要签署，节点在生成区块时自动签署。

## 架构设计

### 分层



## 代码结构

```sh
src
├── main.rs       # 程序入口
├── node          # ⚡核心业务逻辑
│   ├── block.rs  # Block 结构与方法
│   ├── miner.rs  # 生成区块的相关功能
│   ├── mod.rs    # Node 结构与方法
│   ├── syncer.rs # 与其他Peer同步数据
│   ├── tx.rs     # Tx 结构与方法
│   ├── state.rs  # 区块链状态 trait 定义，依赖倒置原则
│   └── peer.rs   # Peer trait 定义，定义其他Peer应实现的方法
├── server/       # HTTP Server，定义路由、请求/响应格式，不包含业务逻辑
├── data/         # 业务数据源，包含本地数据和其他 Peer 中的数据。实现了 Node 定义的 traits
├── error.rs      # 错误类型，不同的错误类型可返回不同的 HTTP Status
├── types.rs      # 自定义类型，目前只包含 Hash 类型
├── utils.rs      # 工具方法
└── wallet/       # 钱包。生成账户、签名、验签等
```

## 代码导读

- [Office Hour 1](doc/01-main-wallet-structs.md): main 命令行解析、wallet 账户管理、基本数据结构与方法。
- [Office Hour 2](doc/02-syncer-miner-server.md): syncer 同步其他节点的数据、miner 生成区块、HTTP Server。

----

![](img/substrate.png)
