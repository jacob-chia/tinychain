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

- [clap](https://docs.rs/crate/clap/4.0.18): 命令行解析

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
RUST_LOG=debug ./target/debug/tinychain new-account
```

演示账户说明：
- `Treasury`: "2bde5a91-6411-46ba-9173-c3e075d32100"
- `Alice`: "3d211869-2505-4394-bd99-0c76eb761bf9"
- `Bob`: "16d5e01e-709a-4536-a4f2-9f069070c51a"
- `Miner`: "346b4cd8-10b6-47ba-a091-6a57bb1afcf9"

#### 关键代码

1. 如果想定义`堆上`常量或`运行时`才能确定值的常量，怎么办？
  - once_cell 或 lazy_static
  - once_cell可能会被纳入std: `https://github.com/rust-lang/rfcs/pull/2788`

比如如下代码，编译器会提示你应该使用`once_cell`：
```rs
use std::collections::HashMap;

static M: HashMap<String, u64> = HashMap::new();

fn main() {
    println!("{:?}", M);
}
```

![](img/static-non-const-fn.png)

- 什么是`const fn`？简单理解为`可以在编译器求值的fn`。

```rs
static M: u64 = num();

fn main() {
    println!("{:?}", M);
}

const fn num() -> u64 {
    10
}
```

2. 如何优雅地处理错误？

- 先上结论
  - [anyhow](https://docs.rs/crate/anyhow/1.0.66): 不关心错误类型，只想简单地处理错误 —— 适用于`bin`
  - [thiserror](https://docs.rs/thiserror/latest/thiserror/): 定义清晰的错误类型 —— 适用于`lib`

- 使用`?`传播错误，前提是`?`处的错误可以自动转为`返回值声明的ErrorType`
  - `使用anyhow::Error`: 任意错误类型均可转为`anyhow::Error`（示例如下）
  - `自定义错误类型`: 需要实现`From` trait来进行类型转换

```rs
// wallet/mod.rs
use anyhow::Result;

pub fn sign(msg: &str, account: &str) -> Result<String> {
    let sig = get_wallet(account)?
        .sign_hash(hash_message(msg))
        .to_string();

    Ok(sig)
}
```

### database | 数据层

#### 相关依赖

- [serde](https://serde.rs/)：序列化/反序列化
- [once-cell](https://docs.rs/crate/once_cell/1.15.0)：lazy static
- [anyhow](https://docs.rs/crate/anyhow/1.0.66)：错误处理

#### 演示

```sh
cargo build
RUST_LOG=debug ./target/debug/tinychain run -m "346b4cd8-10b6-47ba-a091-6a57bb1afcf9"
```

#### 关键代码

1. builder模式（`node/mod.rs`）

```rs
let tx1 = Tx::builder()
    .from(TREASURY)
    .to(ALICE)
    .value(100)
    .nonce(next_nonce)
    .build()
    .sign();
```

2. `derive macro`派生宏怎么用？（几乎所有struct都用到了派生宏）
  - 初始状态：一个都不不需要
  - 在写代码过程中`根据编译器提示`逐个添加

3. 如何从一个结构体实例中“拿走”一个字段的所有权？（`state.rs,block.rs`）

```rs
// block.rs
impl BlockKV {
    pub fn take_block(&mut self) -> Block {
        mem::take(&mut self.value)
    }
}
// state.rs
let mut block_kv: BlockKV = serde_json::from_str(block_str)?;
let block = block_kv.take_block();
self.apply_block(block)?;
```

4. Rust中如何使用`HashMap`？（`state.rs`）

```rs
// balances: HashMap<String, u64>,
// account2nonce: HashMap<String, u64>,

// 确定key一定存在
*self.balances.get_mut(&tx.from).unwrap() -= tx.cost();
// key可能不存在
*self.balances.entry(tx.to.to_owned()).or_default() += tx.value;
*self.account2nonce.entry(tx.from.to_owned()).or_default() = tx.nonce;
```

5. slice的比较（`state.rs`）
  - slice依附于具体的数据结构，就像数据库中`视图`和`表`的关系
  - 两个slice对比，或者 slice与具体的数据结构对比，只取决于`长度`和`内容`，比如下面这块代码：

```rs
fn main() {
    let arr = [1, 2, 3, 4, 5];
    let vec = vec![1, 2, 3, 4, 5];
    let s1 = &arr[..2];
    let s2 = &vec[..2];
    // &[T] 和 &[T] 是否相等取决于长度和内容是否相等
    assert_eq!(s1, s2);
    // &[T] 可以和 Vec<T>/[T;n] 比较，也会看长度和内容
    assert_eq!(&arr[..], vec);
    assert_eq!(&vec[..], arr);
}
```

```rs
// state.rs
fn is_valid_hash(&self, hash: &H256) -> bool {
    // 不需要构造一个H256类型和hash对比
    // 只需要构造一个slice类型是[u8]的任意类型即可，然后比较两个slice
    let hash_prefix = vec![0u8; self.mining_difficulty];
    hash_prefix[..] == hash[..self.mining_difficulty]
}
```

6. 更精细的可见性控制（struct中的字段）
  - [官方文档](https://doc.rust-lang.org/reference/visibility-and-privacy.html)

7. 在iterator上执行`map/filter/take`等骚操作（`block.rs`）

```rs
// Iterator
fn main() {
    let result = vec![1, 2, 3, 4, 5]
        .iter()
        .map(|v| v * v)
        .filter(|v| *v < 16)
        .take(2)
        .collect::<Vec<_>>();

    println!("{:?}", result); // [1, 4]
}
```

```rs
// block.rs
pub fn block_reward(&self) -> u64 {
    let gas_reward: u64 = self.txs.iter().map(|tx| tx.gas_cost()).sum();
    gas_reward + BLOCK_REWORD
}
```

8. 生命周期入门，思路同派生宏（`block.rs`）
  - 初始状态：不用
  - 大部分情况下，编译器可以自己推断
  - 当编译器无法推断时，它会提示你，`按照编译器的提示`加上生命周期注解

----

![](img/substrate.png)
