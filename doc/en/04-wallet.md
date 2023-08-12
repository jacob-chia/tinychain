- [04 | Wallet: Sign \& Verify](#04--wallet-sign--verify)
  - [1 Creating Accounts](#1-creating-accounts)
  - [2 Signing Messages](#2-signing-messages)
    - [2.1 Defining A New Signature Type](#21-defining-a-new-signature-type)
    - [2.2 Defining the Signing Function](#22-defining-the-signing-function)
  - [3 Verifying Signatures](#3-verifying-signatures)
  - [4 Summary](#4-summary)

# 04 | Wallet: Sign & Verify

> This is a hands-on tutorial, so please switch to the corresponding code branch before reading.
>
> - Repo: `https://github.com/jacob-chia/tinychain.git`
> - Branch：`git fetch && git switch 04-wallet`
>
> Important crates used in this lesson:
>
> - [sled](https://docs.rs/sled/latest/sled/): a high-performance embedded KV database.
> - [k256](https://docs.rs/k256/latest/k256/): secp256k1 (K-256) elliptic curve

We start with the simplest part of tinychain: wallet. The wallet is responsible for creating accounts, signing messages, and verifying signatures.

## 1 Creating Accounts

1. Define a struct that wraps the sled DB.

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

2. Implement `new_account`. The format of `address` is ethereum compatible.

```rs
// wallet/src/lib.rs

impl Wallet {
    /// Create a new account.
    pub fn new_account(&self) -> Result<String, WalletError> {
        let privkey = SigningKey::random(&mut OsRng);
        let address = gen_address(&privkey);
        let key_bytes = privkey.to_bytes().to_vec();
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

## 2 Signing Messages

The message signing function is based on [k256](https://docs.rs/k256/latest/k256/) which returns a `signature` and a `recovery id`. When verifying a signature, the `public key` should be recovered from the `message`, `signature` and `recovery id` and then used for verification. However, tinychain should not be aware of the `recovery id` and `public key`, so we introduce a new type to shed the complexity.

### 2.1 Defining A New Signature Type

Note that the `wallet::Signature` in this lesson is different from the `tinychain::Signature` in the previous lesson. The `tinychain::Signature` is for better readability, while the `wallet::Signature` is used to hide complexity.

1. Define the [wallet::Signature](../../wallet/src/signature.rs)

```rs
// wallet/src/signature.rs

// Signature[..64] is the `ecdsa::Signature`, and Signature[64] is the `ecdsa::RecoveryId`.
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
```

2. Add the type conversion code with `wallet::Signature` in `tinychain::Signature`.

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

### 2.2 Defining the Signing Function

```rs
// wallet/src/lib.rs

use k256::ecdsa::SigningKey;
use sha3::Keccak256;

mod error;
mod signature;

pub use error::WalletError;
pub use signature::Signature;

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

## 3 Verifying Signatures

Note that the verification function is not a method of `Wallet`, but an independent function, since it does not rely on the private key of the accounts.

```rs
// wallet/src/lib.rs

// `impl Into<Signature>` is a generic type.
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

## 4 Summary

By implementing a wallet, we have learned:

- The use of the `sled` and `k256` libraries.
- How to get rid of complexity with a new type.

---

| [< 03-Defining Data Structure & API](./03-data-structure-api.md) | [05-Command Line & Config File >](./05-cmd-config.md) |
| ---------------------------------------------------------------- | ----------------------------------------------------- |
