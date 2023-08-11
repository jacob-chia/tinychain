- [03 | Defining Data Structure \& API](#03--defining-data-structure--api)
  - [1 Defining Protobuf](#1-defining-protobuf)
  - [2 Adding Features to Generated Structs](#2-adding-features-to-generated-structs)
  - [3 Beautifying the Log and HTTP Response](#3-beautifying-the-log-and-http-response)
    - [3.1 Defining a Generic Length Array](#31-defining-a-generic-length-array)
    - [3.2 Defining JSON Serialization Format](#32-defining-json-serialization-format)
    - [3.3 Defining Log Format](#33-defining-log-format)
    - [3.4 Deref Trait](#34-deref-trait)
  - [4 Defining Interfaces/Traits](#4-defining-interfacestraits)
  - [5 Summary](#5-summary)

# 03 | Defining Data Structure & API

> This is a hands-on tutorial, so please switch to the corresponding code branch before reading.
>
> - Repo: `https://github.com/jacob-chia/tinychain.git`
> - Branch：`git fetch && git switch 03-data-structure-api`
>
> Important crates used in this lesson:
>
> - [axum](https://docs.rs/axum/latest/axum/): a web application framework that focuses on ergonomics and modularity.
> - [prost](https://docs.rs/prost/latest/prost/): a Protocol Buffers implementation for the Rust Language.
> - [prost-build](https://docs.rs/prost-build/latest/prost_build/): build-time code generation as part of a Cargo build-script.
> - [thiserror](https://docs.rs/thiserror/latest/thiserror/): provides a convenient derive macro for the standard library’s std::error::Error trait

## 1 Defining Protobuf

Follow the steps below:

1. Define `src/schema/api.v1.proto` according to the definition in the first lesson // [src/schema/api.v1.proto](../../src/schema/api.v1.proto)
2. Write a build script `build.rs` in the project root directory // [build.rs](../../build.rs)
3. Run `cargo build`, a file `src/schema/v1.rs` will be generated. Do not modify this file. // [src/schema/v1.rs](../../src/schema/v1.rs)

## 2 Adding Features to Generated Structs

There are only the definitions of the Rust structs in `src/schema/v1.rs`. We need to add some functions to these structs for convenience. No complex logic, just look at the source code.

- [src/schema/block.rs](../../src/schema/block.rs)
- [src/schema/tx.rs](../../src/schema/tx.rs)
- [src/schema/req_resp.rs](../../src/schema/req_resp.rs)

## 3 Beautifying the Log and HTTP Response

Read more in `src/schema/v1.rs`. The `BlockHeader.parent_hash` and `SignedTx.sig` fields are both `Vec<u8>`. Whether printed in the logs or returned in the HTTP response, they are unreadable. Let's solve this problem.

### 3.1 Defining a Generic Length Array

We need to define two types: `Hash` and `Signature`. Both types perform the same functions, but differ in the length of their byte arrays. To simplify things, we can define an array with a generic length and create two aliases.

```rs
// src/types.rs

pub type Hash = Bytes<32>;
pub type Signature = Bytes<65>;

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Bytes<const T: usize>([u8; T]);
```

### 3.2 Defining JSON Serialization Format

The meaning of `#[serde(try_from = "String", into = "String")]` above is: serde will serialize `Bytes` to `String`, and deserialize `String` to `Bytes`. But `Bytes` needs to implement two traits: `TryFrom<String>` and `Into<String>`:

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

// https://doc.rust-lang.org/std/convert/trait.Into.html told us:
// One should avoid implementing Into and implement From instead.
impl<const T: usize> From<Bytes<T>> for String {
    fn from(bytes: Bytes<T>) -> Self {
        String::from("0x") + &hex::encode(bytes.0)
    }
}
```

Now we can use `Hash` and `Signature` in HTTP request/response. The corresponding code is placed in [src/network/http/dto.rs](../../src/network/http/dto.rs). There is no complexity in this section and the code can be viewed directly from the source file.

### 3.3 Defining Log Format

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

Now we can change the log format of `Block` and `Tx`. Since prost-build automatically implements `fmt::Debug` for `Block` and `Tx`, we can only implement `fmt::Display` for them, see [src/schema/block.rs](../../src/schema/block.rs) and [src/schema/tx.rs](../../src/schema/tx.rs).

### 3.4 Deref Trait

You may have noticed that we have implemented `Deref` for `Bytes`. More information about `Deref` can be found in [the official Rust book](https://doc.rust-lang.org/book/ch15-02-deref.html).
Deref makes `&Bytes` behave like `&[u8; T]`. See the following test code:

```rs
// src/types.rs

// `Deref` makes `&Bytes` behave like `&[u8; T]`.
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
        // `&b[..]` is the usage of array, now Bytes can also be used like this
        assert_eq!(&arr[..], &b[..]);

        let v: Vec<u8> = b.into();
        // `as_slice()` is a method of array, now Bytes can also be used like this
        assert_eq!(v.as_slice(), b.as_slice());

        let s2: String = b.into();
        assert_eq!(s, s2);
    }
}
```

## 4 Defining Interfaces/Traits

The following source files define the interfaces/traits we need. This part has no business logic, just follow the design document of the first lesson.

- `HTTP API`: [src/network/http/mod.rs](../../src/network/http/mod.rs). All are axum boilerplate code and should be read in conjunction with the [axum official documentation](https://docs.rs/axum/latest/axum/).
- trait `PeerClient`: [src/biz/peer_client.rs](../../src/biz/peer_client.rs)
- trait `State`: [src/biz/state.rs](../../src/biz/state.rs)

## 5 Summary

We learned through defining data structures and interfaces:

- Using `prost` to handle Protobuf;
- Using generics `pub struct Bytes<const T: usize>([u8; T]);` to reduce duplicate code;
- Some important traits: `Deref`, `From`, `Debug`, and `Display`.

---

| [< 02-Initialization: Pre-commit Hooks & Github Action](./02-init-project.md) | [04-Wallet: Sign & Verify >](./04-wallet.md) |
| ----------------------------------------------------------------------------- | -------------------------------------------- |
