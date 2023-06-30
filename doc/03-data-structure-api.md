- [03 | 定义数据结构与接口](#03--定义数据结构与接口)
  - [1 写在前面](#1-写在前面)
    - [1.1 关于依赖库](#11-关于依赖库)
    - [1.2 关于错误处理](#12-关于错误处理)
  - [2 定义/编译 protobuf](#2-定义编译-protobuf)
  - [3 给生成的结构体添加一些功能](#3-给生成的结构体添加一些功能)
  - [4 自定义日志/序列化格式](#4-自定义日志序列化格式)
    - [4.1 泛型长度的数组](#41-泛型长度的数组)
    - [4.2 自定义 JSON 序列化格式](#42-自定义-json-序列化格式)
    - [4.3 自定义日志格式](#43-自定义日志格式)
    - [4.4 Deref trait](#44-deref-trait)
  - [5 定义接口/Traits](#5-定义接口traits)
  - [6 小结](#6-小结)

# 03 | 定义数据结构与接口

> 本文为实战课，需要切换到对应的代码分支，并配合依赖库的文档一起学习。
>
> - 代码分支：`git fetch && git switch 03-data-structure-api`
> - [axum](https://docs.rs/axum/latest/axum/): HTTP Server 框架，由 tokio 团队维护
> - [prost](https://docs.rs/prost/latest/prost/): 用来处理 protobuf
> - [prost-build](https://docs.rs/prost-build/latest/prost_build/): 将 protobuf 文件编译成 rust 文件
> - [thiserror](https://docs.rs/thiserror/latest/thiserror/): 通过宏来简化错误处理
>
> 其他 crates 使用简单，不再一一列举，清单在`./Cargo.toml`中

## 1 写在前面

### 1.1 关于依赖库

> tldr： 学习一个 crate 的最好方法是阅读**官方文档** + **源代码中的 examples**

从本课开始，我们就要依赖各种库了，要学习一个库如何使用，没有什么比阅读官方文档更好的方法了。任何转述概括类的文章都具有时效性，尤其是 Rust 生态还在快速发展中，各种库可能会做出向后不兼容的更新，所以你看到的转述类的文章很可能是过时的。就以本项目为例，我在开发过程中就遇到过两次向后不兼容的更新，一次是 http 框架 `axum`，另一次是 p2p 框架 `rust-libp2p`。

另外，对于一些功能复杂的 crates, 一般会给出不同使用场景的示例代码（在源码根目录的 examples 目录下），可以结合 examples 学习。

### 1.2 关于错误处理

错误处理贯穿整个开发过程中，这里提到前面做个统一说明。

本项目的风格是每个 crate 一个全局的 Error，所有“外部错误”（包括 std 和依赖库返回的错误）都先转为“内部错误”（自定义的错误）再向上传播。所以错误处理的繁琐之处在于要处理各种错误类型的转换，利用`thiserror`可以很方便的做到这一点。假设自定义的错误如下：

```rs
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Config file not exist: {0}")]
    ConfigNotExist(String),
    // 自动为 Error 实现了 `From<toml::de::Error>` trait.
    #[error(transparent)]
    InvalidConfig(#[from] toml::de::Error),
}
```

之后，当我们遇到错误时，可以利用`“?”`自动转换，也可以利用`map_err`手动转换，比如下面这块代码：

```rs
// 读取配置文件
impl Config {
    pub fn load(path: &str) -> Result<Self, Error> {
        // fs::read_to_string() 出错时返回 io::Error，这个错误非常常见，
        // 并不能将所有IO错误都视为读取配置文件出错，所以这里使用`map_err`手动转换错误。
        let content =
            fs::read_to_string(path).map_err(|_| Error::ConfigNotExist(path.to_string()))?;

        // toml::from_str()出错时返回 toml::de::Error
        // 由于我们的自定义错误实现了 `From<toml::de::Error>` trait，所以可以使用 `?` 自动转换错误类型
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}
```

好了，开始写代码！

## 2 定义/编译 protobuf

按照步骤操作即可：

1. 按照第一课架构设计中的定义来写`src/schema/api.v1.proto` // [源码](../src/schema/api.v1.proto)
2. 编写编译脚本，在项目根目录添加`build.rs` // [源码](../build.rs)
3. 执行`cargo build`，会生成`src/schema/v1.rs`，不要修改这个文件。 // [源码](../src/schema/v1.rs)

## 3 给生成的结构体添加一些功能

在`src/schema/v1.rs`中，只有 Rust 结构体的定义，要想更方便的使用这些结构体，我们还需要添加一些功能。没有复杂逻辑，直接看源码吧。

- 给`Block`添加功能：[src/schema/block.rs](../src/schema/block.rs)
- 给`Tx`添加功能：[src/schema/tx.rs](../src/schema/tx.rs)
- 给 p2p 的`Request/Response`附加功能：[src/schema/req_resp.rs](../src/schema/req_resp.rs)

其中有两个方法需要说明：

```rs
// src/schema/tx.rs
impl Tx {
    /// 每笔交易都需要消耗一定的 Gas 费用，这笔费用会从交易发送者账户扣除、并奖励给矿工。
    /// 在本项目中，Gas 费用是个固定值。
    pub fn gas_cost(&self) -> u64 {
        self.gas * self.gas_price
    }
}

// src/schema/block.rs
impl Block {
    /// 矿工的挖矿奖励，在本项目中为该区块中所有交易的 Gas 费用总和。
    pub fn block_reward(&self) -> u64 {
        self.txs.iter().map(|tx| tx.gas_cost()).sum()
    }
}
```

## 4 自定义日志/序列化格式

继续看`src/schema/v1.rs`，其中`BlockHeader.parent_hash`和`SignedTx.sig`两个字段都是`Vec<u8>`字节流类型，不管是打印日志还是在 HTTP Response 中返回，都是不可读的。接下来我们解决这个问题。

### 4.1 泛型长度的数组

我们需要定义`Hash`和`Signature`两个类型，但这两个类型所做的事情是一样的，唯一区别是字节流的长度不同。所以我们可以定义一个数组长度为泛型的 bytes 数组，然后再定义两个别名：

```rs
// src/types.rs

pub type Hash = Bytes<32>;
pub type Signature = Bytes<65>;

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Bytes<const T: usize>([u8; T]);
```

### 4.2 自定义 JSON 序列化格式

上面`#[serde(try_from = "String", into = "String")]`的意思是：serde 会将 Bytes 序列化为 String, 会把 String 反序列化为 Bytes。但需要 Bytes 实现 `TryFrom<String>` 和 `Into<String>` 两个 traits：

```rs
// src/types.rs

impl<const T: usize> TryFrom<String> for Bytes<T> {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let val = if let Some(val) = value.strip_prefix("0x") {
            val
        } else {
            &value
        };

        Ok(hex::decode(val).map(Self::from)?)
    }
}

// 当你为 String 实现了 From<Bytes<T>>时，Rust会自动为 Bytes<T> 实现 Into<String>
// 所以，永远只手动实现 From trait.
impl<const T: usize> From<Bytes<T>> for String {
    fn from(bytes: Bytes<T>) -> Self {
        let mut s = String::from("0x");
        s.push_str(&hex::encode(bytes.0));
        s
    }
}
```

做完这些工作后，就可以在 HTTP 的 Request/Response 中使用`Hash`和`Signature`两个类型了。JSON 格式的数据只和 http 相关，所以我把相关的代码放在了[src/network/http/dto.rs](../src/network/http/dto.rs)中，这块儿没有难点，直接跳转到源文件看代码吧。

### 4.3 自定义日志格式

```rs
// src/types.rs

impl<const T: usize> Bytes<T> {
    fn fmt_as_hex(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x")?;

        for byte in self.0.iter() {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

// For better logging.
impl<const T: usize> fmt::Debug for Bytes<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_as_hex(f)
    }
}

// For better logging.
impl<const T: usize> fmt::Display for Bytes<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_as_hex(f)
    }
}
```

做完这些工作后，我们就可以修改`Block`和`Tx`的日志格式了。因为 prost-build 会自动为`Block`和`Tx`实现`fmt::Debug`，我们再实现`fmt::Debug`会报错，只能为其实现`fmt::Display`，详见源码[src/schema/block.rs](../src/schema/block.rs) 和 [src/schema/tx.rs](../src/schema/tx.rs)。

### 4.4 Deref trait

> [更详细的 Deref 介绍看 The Book](https://doc.rust-lang.org/book/ch15-02-deref.html)

简单来说，Deref 可以让 `&Bytes<T>` 使用起来和 `&[u8; T]` 一样。我们看下方的测试代码，Bytes 可以像 array 一样使用 `&b[..]` 或 `b.as_slice()`。

```rs
// src/types.rs

// `Deref` makes `&Bytes` behave like a `&[u8; T]`.
impl<const T: usize> Deref for Bytes<T> {
    type Target = [u8; T];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_into() {
        let s = "0x000036755a024ef491b6710fe765e06e33a616f83b8a33c6a1963ab20f6e5bdb";
        let b = Bytes::<32>::try_from(s.to_string()).unwrap();

        let arr: [u8; 32] = b.into();
        // `&b[..]` 是 array 的用法，现在 Bytes 也可以这么用了
        assert_eq!(&arr[..], &b[..]);

        let v: Vec<u8> = b.into();
        // `as_slice()` 是 array 的方法，现在 Bytes 也可以这么用了
        assert_eq!(v.as_slice(), b.as_slice());

        let s2: String = b.into();
        assert_eq!(s, s2);
    }
}
```

## 5 定义接口/Traits

这部分没有业务逻辑，按照第一课的设计文档定义即可。

- `HTTP API` 源文件：[src/network/http/mod.rs](../src/network/http/mod.rs)。全是 axum 样板代码，配合 [axum 官方文档](https://docs.rs/axum/latest/axum/) 食用。
- `trait PeerClient` 源文件：[src/biz/peer_client.rs](../src/biz/peer_client.rs)
- `trait State` 源文件：[src/biz/state.rs](../src/biz/state.rs)

## 6 小结

我们通过定义数据结构和接口，学到了：

- 使用 `thiserror` 优雅地处理错误；
- 使用 `prost` 处理 Protobuf;
- 使用泛型 `pub struct Bytes<const T: usize>([u8; T]);` 消除重复代码；
- 一些常用的 trait `Deref`, `From`, `Debug`, 和 `Display`。

---

| [< 02-项目初始化：Pre-commit Hooks 与 Github Action](./02-init-project.md) | [04-钱包: 签名与验签 >](./04-wallet.md) |
| -------------------------------------------------------------------------- | --------------------------------------- |
