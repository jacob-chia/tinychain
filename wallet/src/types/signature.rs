//! Signature type for transactions.
//!
//! A signature is a 65-byte array, where the first 64 bytes is the `ecdsa::Signature`,
//! and the last byte is the `ecdsa::RecoveryId`.

use std::{
    fmt::{self, Debug},
    ops::Deref,
};

use k256::ecdsa;
use serde::{Deserialize, Serialize};

use crate::WalletError;

// Serialize and deserialize Signature as a '0x'-prefixed hex string.
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Signature([u8; 65]);

impl Signature {
    fn fmt_as_hex(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x")?;
        for byte in self.0.iter() {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

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

impl From<&[u8]> for Signature {
    fn from(bytes: &[u8]) -> Self {
        let mut sig = [0u8; 65];
        sig.copy_from_slice(bytes);
        Self(sig)
    }
}

impl TryFrom<String> for Signature {
    type Error = hex::FromHexError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let val = if let Some(val) = value.strip_prefix("0x") {
            val
        } else {
            &value
        };

        hex::decode(val).map(|bytes| Self::from(&bytes[..]))
    }
}

impl From<Signature> for Vec<u8> {
    fn from(sig: Signature) -> Self {
        sig.0.to_vec()
    }
}

impl From<Signature> for String {
    fn from(sig: Signature) -> Self {
        let mut s = String::from("0x");
        s.push_str(&hex::encode(sig.0));
        s
    }
}

// Implement Debug so that Signature can be printed as a hex string.
impl Debug for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_as_hex(f)
    }
}

// Implement Display so that Signature can be printed as a hex string.
impl fmt::Display for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_as_hex(f)
    }
}

// Implement Deref so that Signature can be used as &[u8; 65]
impl Deref for Signature {
    type Target = [u8; 65];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signature_from() {
        let mut sig_str = "ce53abb3721bafc561408ce8ff99c909f7f0b18a2f788649d6470162ab1aa0323971edc523a6d6453f3fb6128d318d9db1a5ff3386feb1047d9816e780039d52"
            .to_string();
        let mut sig_bytes = hex::decode(&sig_str).unwrap();

        let sig = ecdsa::Signature::from_bytes(sig_bytes.as_slice().into()).unwrap();
        let recid = ecdsa::RecoveryId::from_byte(0x01).unwrap();

        sig_str.push_str("01");
        sig_bytes.push(recid.to_byte());

        let signature1 = Signature::from((sig, recid));
        assert_eq!(&signature1[..], &sig_bytes[..]);
        let signature2 = Signature::from(&sig_bytes[..]);
        assert_eq!(signature1, signature2);
        let signature3 = Signature::try_from(sig_str).unwrap();
        assert_eq!(signature1, signature3);
    }

    #[test]
    fn signature_into() {
        let sig_str = "0xce53abb3721bafc561408ce8ff99c909f7f0b18a2f788649d6470162ab1aa0323971edc523a6d6453f3fb6128d318d9db1a5ff3386feb1047d9816e780039d5201";

        let signature = Signature::try_from(sig_str.to_string()).unwrap();
        let (sig, recid): (ecdsa::Signature, ecdsa::RecoveryId) = signature.try_into().unwrap();
        let bytes: Vec<u8> = signature.into();
        assert_eq!(&bytes[..64], &sig.to_bytes()[..]);
        assert_eq!(bytes[64], recid.to_byte());

        let s: String = signature.into();
        assert_eq!(s, sig_str);
    }

    #[test]
    fn signature_debug_display() {
        let sig_str = "0xce53abb3721bafc561408ce8ff99c909f7f0b18a2f788649d6470162ab1aa0323971edc523a6d6453f3fb6128d318d9db1a5ff3386feb1047d9816e780039d5201";

        let signature = Signature::try_from(sig_str.to_string()).unwrap();
        assert_eq!(format!("{:?}", signature), sig_str);
        assert_eq!(format!("{}", signature), sig_str);
    }
}
