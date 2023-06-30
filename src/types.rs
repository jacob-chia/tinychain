//! Types for better readability.
//!
//! When the bytes are serialized or logged, they are represented as a '0x'-prefixed hex string.

use std::{fmt, ops::Deref};

use serde::{Deserialize, Serialize};

use crate::error::Error;

pub type Hash = Bytes<32>;
pub type Signature = Bytes<65>;

// Serialize and deserialize Bytes as a '0x'-prefixed hex string.
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Bytes<const T: usize>([u8; T]);

impl<const T: usize> Bytes<T> {
    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|&b| b == 0)
    }

    fn fmt_as_hex(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x")?;

        for byte in self.0.iter() {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

// `Deref` makes `&Bytes` behave like a `&[u8; T]`.
impl<const T: usize> Deref for Bytes<T> {
    type Target = [u8; T];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const T: usize> From<[u8; T]> for Bytes<T> {
    fn from(bytes: [u8; T]) -> Self {
        Self(bytes)
    }
}

impl<const T: usize> From<Vec<u8>> for Bytes<T> {
    fn from(bytes: Vec<u8>) -> Self {
        let mut array = [0u8; T];
        array.copy_from_slice(bytes.as_slice());
        Self(array)
    }
}

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

impl<const T: usize> From<Bytes<T>> for [u8; T] {
    fn from(bytes: Bytes<T>) -> Self {
        bytes.0
    }
}

impl<const T: usize> From<Bytes<T>> for Vec<u8> {
    fn from(bytes: Bytes<T>) -> Self {
        bytes.0.to_vec()
    }
}

impl<const T: usize> From<Bytes<T>> for String {
    fn from(bytes: Bytes<T>) -> Self {
        let mut s = String::from("0x");
        s.push_str(&hex::encode(bytes.0));
        s
    }
}

impl<const T: usize> Default for Bytes<T> {
    fn default() -> Self {
        Self([0u8; T])
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_value_is_zero() {
        let bytes = Bytes::<32>::default();
        assert!(bytes.is_zero());
    }

    #[test]
    fn bytes_from() {
        let s = "0x000036755a024ef491b6710fe765e06e33a616f83b8a33c6a1963ab20f6e5bdb";
        let v = hex::decode(&s[2..]).unwrap();
        let arr: [u8; 32] = v.as_slice().try_into().unwrap();

        let b1 = Bytes::<32>::from(arr);
        assert_eq!(&b1[..], &arr[..]);

        let b2 = Bytes::<32>::from(v);
        assert_eq!(b1, b2);

        let b3 = Bytes::<32>::try_from(s.to_string()).unwrap();
        assert_eq!(b2, b3);
    }

    #[test]
    fn bytes_into() {
        let s = "0x000036755a024ef491b6710fe765e06e33a616f83b8a33c6a1963ab20f6e5bdb";
        let b = Bytes::<32>::try_from(s.to_string()).unwrap();

        let arr: [u8; 32] = b.into();
        assert_eq!(&arr[..], &b[..]);

        let v: Vec<u8> = b.into();
        assert_eq!(v.as_slice(), b.as_slice());

        let s2: String = b.into();
        assert_eq!(s, s2);
    }

    #[test]
    fn bytes_logging_format() {
        let s = "0x000036755a024ef491b6710fe765e06e33a616f83b8a33c6a1963ab20f6e5bdb";
        let b = Bytes::<32>::try_from(s.to_string()).unwrap();
        assert_eq!(s, format!("{:?}", b));
        assert_eq!(s, format!("{}", b));
    }
}
