- [04 | 钱包: 签名与验签](#04--钱包-签名与验签)
  - [1 生成账户](#1-生成账户)
  - [2 签名](#2-签名)
    - [2.1 用 Signature 封装复杂性](#21-用-signature-封装复杂性)
    - [2.2 签名功能](#22-签名功能)
  - [3 验签](#3-验签)
  - [4 小结](#4-小结)

# 04 | 钱包: 签名与验签

> 本文为实战课，需要切换到对应的代码分支，并配合依赖库的文档一起学习。
>
> - Repo: `https://github.com/jacob-chia/tinychain.git`
> - 分支：`git fetch && git switch 04-wallet`
> - [sled](https://docs.rs/sled/latest/sled/): 纯 Rust 编写的嵌入式 KV store， 对标 RocksDB
> - [k256](https://docs.rs/k256/latest/k256/): secp256k1 (K-256) 椭圆曲线
>
> 其他 crates 使用简单，不再一一列举，清单在`wallet/Cargo.toml`中

tinychain 依赖 wallet 和 tinyp2p，我们先从功能最简单的 wallet 开始。我们的钱包要实现三个功能：

- `生成账户`：生成一个新账户私钥，把私钥保存至 sled 中，将私钥地址返回给用户；
- `签名`：签名时需要提供账户地址，根据地址获取私钥后签名。
- `验签`：验签时不应依赖账户公钥，区块链节点不可能保存每个用户的公钥。而是从签名中恢复出公钥再进行验签操作。

## 1 生成账户

1. 生成的账户需要保存到 sled 中，所以我们先创建一个结构体用来封装 sled DB。

```rs
// wallet/src/lib.rs

#[derive(Debug, Clone)]
pub struct Wallet {
    db: sled::Db,
}

impl Wallet {
    /// Create a new wallet.
    pub fn new(keystore_dir: &str) -> Self {
        Self {
            db: sled::open(keystore_dir).unwrap(),
        }
    }
}
```

2. 生成新账户，账户地址的生成规则同以太坊，直接看代码：

```rs
// wallet/src/lib.rs

impl Wallet {
    /// Create a new account.
    pub fn new_account(&self) -> Result<String, WalletError> {
        let privkey = SigningKey::random(&mut OsRng);
        let address = gen_address(&privkey);
        let key_bytes = privkey.to_bytes().to_vec();
        // 注意sled中保存的是bytes
        self.db.insert(address.as_bytes(), key_bytes)?;

        Ok(address)
    }
}

fn gen_address(privkey: &SigningKey) -> String {
    let pubkey = privkey.verifying_key().to_encoded_point(false);
    let pubkey = pubkey.as_bytes();
    let hash = Keccak256::digest(&pubkey[1..]);

    let mut bytes = [0u8; 20];
    bytes.copy_from_slice(&hash[12..]);
    String::from("0x") + &hex::encode(bytes)
}
```

## 2 签名

我们依赖的 secp256k1 库[k256](https://docs.rs/k256/latest/k256/)生成的签名包括两部分：`Signature<Secp256k1>` + `RecoveryId`。验签时需要根据签名消息`msg`、`Signature<Secp256k1>`、和`RecoveryId`三个值恢复出公钥，再进行验签。而对于“用户” tinychain 来说，不需要知道这些细节，所以我们新增一个类型来封装复杂性。

### 2.1 用 Signature 封装复杂性

这里注意不要和上节课中的 Signature 弄混，这两个类型的目的不同：

- `tinychain::Signature`: 用于改善可读性，在 JSON、日志中将 bytes 显示为 hex string。
- `wallet::Signature`: 用户封装复杂性，在返回给 tinychain 时将 `(Signature<Secp256k1>, RecoveryId)` 转为 bytes，在验签时将 bytes 转为`(Signature<Secp256k1>, RecoveryId)`。

1. wallet::Signature 定义如下：

```rs
// wallet/src/signature.rs

// 前64个字节是 Signature<Secp256k1>，最后一个字节是 RecoveryId。
#[derive(Clone, Copy)]
pub struct Signature([u8; 65]);

impl From<(ecdsa::Signature, ecdsa::RecoveryId)> for Signature {
    fn from((sig, recid): (ecdsa::Signature, ecdsa::RecoveryId)) -> Self {
        let mut bytes = [0u8; 65];
        bytes[..64].copy_from_slice(sig.to_bytes().as_ref());
        bytes[64] = recid.to_byte();
        Self(bytes)
    }
}

impl TryFrom<Signature> for (ecdsa::Signature, ecdsa::RecoveryId) {
    type Error = WalletError;

    fn try_from(value: Signature) -> Result<Self, Self::Error> {
        let sig = ecdsa::Signature::from_bytes(value[..64].as_ref().into())
            .map_err(|_| WalletError::InvalidSignature)?;

        let recid = ecdsa::RecoveryId::from_byte(value[64]).ok_or(WalletError::InvalidSignature)?;

        Ok((sig, recid))
    }
}

// 另外还有与 [u8; 65] 的互转代码，略。详见源码。
```

2. 在 tinychain::Signature 中添加与 wallet::Signature 的类型互转代码

```rs
// src/types.rs

impl From<wallet::Signature> for Signature {
    fn from(signature: wallet::Signature) -> Self {
        Self(signature.into())
    }
}

impl From<Signature> for wallet::Signature {
    fn from(signature: Signature) -> Self {
        Self::from(signature.0)
    }
}
```

### 2.2 签名功能

有了 wallet::Signature，我们的签名功能就简单了，直接看代码。

```rs
// wallet/src/lib.rs

impl Wallet {
    /// Sign a message.
    pub fn sign(&self, msg: &[u8], addr: &str) -> Result<Signature, WalletError> {
        let privkey = self.get_privkey(addr)?;
        let digest = Keccak256::new_with_prefix(msg);
        let (sig, recid) = privkey.sign_digest_recoverable(digest)?;

        Ok(Signature::from((sig, recid)))
    }

    fn get_privkey(&self, addr: &str) -> Result<SigningKey, WalletError> {
        let privkey = self
            .db
            .get(addr.as_bytes())?
            .map(|k| k.to_vec())
            .ok_or_else(|| WalletError::AccountNotFound(addr.to_string()))?;

        SigningKey::from_bytes(privkey.as_slice().into())
            .map_err(|_| WalletError::AccountNotFound(addr.to_string()))
    }
}
```

## 3 验签

- 验签功能不依赖 wallet，所以注意该接口不是 `Wallet`结构体的方法，而是一个独立的函数。
- 这里用到了泛型入参 `impl Into<Signature>`，可以让接口支持更多类型的入参。如果不用泛型，那么调用者需要先将参数转为`wallet::Signature`再调用该接口。对于本项目来说，用不用泛型参数其实没多大区别；但如果一个接口被大量的使用，那么使用泛型参数可以减轻一些调用者的工作量。

```rs
// wallet/src/lib.rs

pub fn verify_signature(msg: &[u8], signature: impl Into<Signature>) -> Result<(), WalletError> {
    let signature = signature.into();
    let (sig, recid) = signature.try_into()?;
    let digest = Keccak256::new_with_prefix(msg);

    let recovered_key = VerifyingKey::recover_from_digest(digest.clone(), &sig, recid)
        .map_err(|_| WalletError::InvalidSignature)?;

    recovered_key
        .verify_digest(digest, &sig)
        .map_err(|_| WalletError::InvalidSignature)?;

    Ok(())
}
```

## 4 小结

本课内容较为简单，主要是对椭圆曲线库的封装，但我们依然学到了一些新知识，包括：

- sled 的基本用法；
- 如何通过自定义 Signature 封装内部复杂性；
- 泛型参数的基本用法；

---

| [< 03-定义数据结构与接口](./03-data-structure-api.md) | [05-读取命令行与配置文件 >](./05-cmd-config.md) |
| ----------------------------------------------------- | ----------------------------------------------- |
