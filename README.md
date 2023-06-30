# 跟我一起写区块链

## 目录

- [前言](README.md)
- [01 | 架构设计：谋定而后动](doc/01-architecture.md)
- [02 | 项目初始化：Pre-commit Hooks 与 Github Action](doc/02-init-project.md)
- [03 | 定义数据结构与接口](doc/03-data-structure-api.md)
- [04 | 钱包: 签名与验签](doc/04-wallet.md)
- [05 | 读取命令行与配置文件](doc/05-cmd-config.md)
- [06 | libp2p: 需求分析与两种封装风格的思考](doc/06-libp2p.md)
- [07 | tinyp2p：封装复杂性](doc/07-tinyp2p.md)
- [08 | 网络层](doc/08-network.md)
- [09 | 业务层](doc/09-biz.md)
- [10 | 存储层](doc/10-data.md)
- [11 | 集成测试、性能测试](doc/11-test.md)
- [12 | 测量和监控](doc/12-measure-tracing.md)
- [后记](doc/13-end.md)

## 前言

### 项目简介

本项目旨在通过一个`分布式账本`来演示区块链的基本原理，主要功能包括：

- 通过 `HTTP JSON API` 向用户提供`转账`、`查询`等功能；
- 节点之间通过 `P2P` 协议进行交互，数据通过 `protobuf` 编解码，功能包括`节点发现`、`广播交易`、`广播区块`、从最佳节点（区块高度最高的节点）`同步区块`等；
- 共识机制采用 `POW`；
- 使用 `sled`（纯 Rust 编写的嵌入式 KV store， 对标 RocksDB）存储状态；
- 为了方便演示，节点提供了`钱包`的`签名/验签`功能，用户发送交易时无需对交易签名，签名的动作由节点自动完成。

### 效果演示

TODO
