use std::{
    fmt::{self, Debug},
    ops::Deref,
};

use serde::{Deserialize, Serialize};

use crate::error::Error;

// Serialize and deserialize Hash as a '0x'-prefixed hex string.
#[derive(Default, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Hash([u8; 32]);

impl Hash {
    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|&x| x == 0u8)
    }

    fn fmt_as_hex(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x")?;

        for byte in self.0.iter() {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

// Implement Deref so that Hash can be used as &[u8; 32]
impl Deref for Hash {
    type Target = [u8; 32];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<[u8; 32]> for Hash {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl From<Vec<u8>> for Hash {
    fn from(bytes: Vec<u8>) -> Self {
        let mut hash = [0u8; 32];
        hash.copy_from_slice(bytes.as_slice());
        Self(hash)
    }
}

impl TryFrom<String> for Hash {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let val = if value.starts_with("0x") {
            &value[2..]
        } else {
            &value
        };

        Ok(hex::decode(val).map(Self::from)?)
    }
}

impl From<Hash> for Vec<u8> {
    fn from(hash: Hash) -> Self {
        hash.0.to_vec()
    }
}

impl From<Hash> for String {
    fn from(hash: Hash) -> Self {
        let mut s = String::from("0x");
        s.push_str(&hex::encode(hash.0));
        s
    }
}

// Implement Debug so that Hash can be printed as a hex string.
impl Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x")?;

        for byte in self.0.iter() {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

// Implement Display so that Hash can be printed as a hex string.
impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_as_hex(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_default_value_is_zero() {
        let hash = Hash::default();
        assert!(hash.is_zero());
    }

    #[test]
    fn hash_from() {
        let str = "0x000036755a024ef491b6710fe765e06e33a616f83b8a33c6a1963ab20f6e5bdb";
        let bytes = hex::decode(&str[2..]).unwrap();
        let arr: [u8; 32] = bytes.as_slice().try_into().unwrap();

        let hash1 = Hash::from(arr);
        assert_eq!(&hash1[..], &arr[..]);

        let hash2 = Hash::from(bytes);
        assert_eq!(hash1, hash2);

        let hash3 = Hash::try_from(str.to_string()).unwrap();
        assert_eq!(hash2, hash3);
    }

    #[test]
    fn hash_into() {
        let str = "0x000036755a024ef491b6710fe765e06e33a616f83b8a33c6a1963ab20f6e5bdb";
        let hash = Hash::try_from(str.to_string()).unwrap();

        let bytes: Vec<u8> = hash.into();
        assert_eq!(&bytes[..], &hash[..]);

        let str2: String = hash.into();
        assert_eq!(str, str2);
    }

    #[test]
    fn hash_debug_display() {
        let str = "0x000036755a024ef491b6710fe765e06e33a616f83b8a33c6a1963ab20f6e5bdb";
        let hash = Hash::try_from(str.to_string()).unwrap();
        assert_eq!(str, format!("{:?}", hash));
        assert_eq!(str, format!("{}", hash));
    }
}
