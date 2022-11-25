# tinychain

[Substrate入门课](https://appbhteffsi3308.h5.xiaoeknow.com/v1/goods/goods_detail/p_62ac1ea6e4b0ba331dc9542c?type=3&type=3) Office Hour 演示项目。

## 项目介绍

本项目旨在介绍区块链基本原理，重点关注如何`生成区块`和如何`与其他节点达成共识`，会简化其他逻辑。
- 只有`转账`功能，不包含智能合约，也不包含Substrate相关内容
- 共识机制：`PoW`
- 网络协议：使用`HTTP`模拟P2P
- 存储：`文件`存储
- 账户管理：为了简化演示流程，`所有账户密码、地址、公私钥均存储在后端`，用户只持有account_id（UUID格式）。用户发送Tx（即Transaction）时不需要签署，节点在生成区块时自动签署。

## 代码结构

```sh
src
├── database       # 数据层
│   ├── block.rs   # 区块相关
│   ├── genesis.rs # 区块链的初始状态
│   ├── mod.rs     # 入口
│   ├── state.rs   # 区块链的当前状态（所有账户的余额、Nonce，最新的区块，当前挖矿难度等）
│   └── tx.rs      # Transaction相关
├── main.rs        # 入口。命令行解析，执行对应的命令
├── node           # TODO 生成区块、Peers管理、Peers间的共识、区块同步等
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
cargo build
./target/debug/tinychain new-account
```

演示账户说明：
- `Treasury`: "2bde5a91-6411-46ba-9173-c3e075d32100"
- `Alice`: "3d211869-2505-4394-bd99-0c76eb761bf9"
- `Bob`: "16d5e01e-709a-4536-a4f2-9f069070c51a"
- `Miner`: "346b4cd8-10b6-47ba-a091-6a57bb1afcf9"

#### 关键代码

1. 如果想定义`堆上`常量或`运行时`才能确定值的常量，怎么办？
2. 如何优雅地处理错误？



----

![](img/substrate.png)
