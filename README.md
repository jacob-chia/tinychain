# tinychain

[Substrate入门课](https://appbhteffsi3308.h5.xiaoeknow.com/v1/goods/goods_detail/p_62ac1ea6e4b0ba331dc9542c?type=3&type=3)演示项目。

仅用于演示区块链的基本原理，不包含substrate相关内容。tinychain包含以下特性：
- 重点关注如何`生成区块`和如何`与其他节点达成共识`，其他逻辑尽量简化。
- 共识：`PoW`
- 网络协议：使用`HTTP`模拟P2P
- 账户管理：账户用于签署、验签Transaction（下文简称`Tx`），不是本项目的重点，所以直接使用[ethers-core](https://docs.rs/crate/ethers-core/0.17.0)和[ethers-signers](https://docs.rs/crate/ethers-signers/0.17.0)中的`LocalWallet`来管理账户，且用户不持有私钥，所有签名、验签都在后端完成。
- 存储：`文件`存储

## 模块说明

## 代码导读

### database

#### block

[database/block.rs](src/database/block.rs)

- builder模式
- miner字段`&'static str` vs `String`
- mem::take

#### genesis

- anyhow::Error
