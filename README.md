# tinychain

[Substrate入门课](https://appbhteffsi3308.h5.xiaoeknow.com/v1/goods/goods_detail/p_62ac1ea6e4b0ba331dc9542c?type=3&type=3)演示项目。

仅用于演示区块链的基本原理，不包含substrate相关内容。tinychain包含以下特性：
- 重点关注如何`生成区块`和如何`与其他节点达成共识`，其他逻辑尽量简化。
- 共识：`PoW`
- 网络协议：使用`HTTP`模拟P2P
- 账户管理：账户用于签署、验签Transaction（下文简称`Tx`），不是本项目的重点，所以直接使用[ethers-signers](https://docs.rs/crate/ethers-signers/0.17.0)中的`LocalWallet`来管理账户。用户只需持有`account_id`，密码、账户地址、公私钥都存储在后端。
- 存储：`文件`存储

## 模块说明

```sh
src
├── database       # 数据层
│   ├── block.rs   # 区块相关
│   ├── genesis.rs # 区块链的初始状态
│   ├── mod.rs     # 入口
│   ├── state.rs   # 区块链的当前状态（所有账户的余额、Nonce，最新的区块，当前挖矿难度等）
│   └── tx.rs      # Transaction相关
├── main.rs        # 入口。命令行解析，执行对应的命令
├── node           # TODO 挖矿、节点间区块同步、HTTP Server等
└── wallet         # 钱包。账户管理、签名、验签
```

## 代码导读

### main | 命令行解析

#### 相关依赖

- [clap文档](https://docs.rs/crate/clap/4.0.18)

#### 演示

```sh
cargo build
./target/debug/tinychain
./target/debug/tinychain new-account -h
./target/debug/tinychain run -h
```

### wallet | 账户管理

#### 相关依赖

- [ethers-signers](https://docs.rs/crate/ethers-signers/0.17.0)：账户管理
- [ethers-core](https://docs.rs/crate/ethers-core/0.17.0)：只用到了Hash函数和Hash结果的类型
- [once-cell](https://docs.rs/crate/once_cell/1.15.0)：lazy static
- [anyhow](https://docs.rs/crate/anyhow/1.0.66)：错误处理

#### 演示

```sh
./target/debug/tinychain new-account
```

#### 问题

1. once_cell和lazy_static应该用哪个？
2. 错误处理

### database

#### block

[database/block.rs](src/database/block.rs)

- builder模式
- miner字段`&'static str` vs `String`
- mem::take

#### genesis

- anyhow::Error

#### state

- slice 的比较，is_valid_hash
- hashmap的操作
- add_block 为什么需要clone -> 操作失败需要回滚，暂不考虑并发
- 哪些struct需要clone
