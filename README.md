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
├── node           # 生成区块、Peers间区块同步、Http Server
└── wallet         # 钱包。账户管理、签名、验签
```

## 代码导读

- [Office Hour 1](doc/office-hour-1.md): 命令行解析、账户管理、状态管理。

----

![](img/substrate.png)
