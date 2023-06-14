use std::{
    fmt::{self, Debug},
    ops::Deref,
};

use serde::{Deserialize, Serialize};

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

// Implement Debug so that Hash can be printed as a hex string.
impl Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_as_hex(f)
    }
}

// Implement Display so that Hash can be printed as a hex string.
impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_as_hex(f)
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
        hash.copy_from_slice(&bytes);
        Self(hash)
    }
}

impl From<Hash> for Vec<u8> {
    fn from(hash: Hash) -> Self {
        hash.0.to_vec()
    }
}

impl TryFrom<String> for Hash {
    type Error = hex::FromHexError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let val = if value.starts_with("0x") {
            &value[2..]
        } else {
            &value
        };

        hex::decode(val).map(Self::from)
    }
}

impl From<Hash> for String {
    fn from(hash: Hash) -> Self {
        let mut s = String::from("0x");
        s.push_str(&hex::encode(hash.0));
        s
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
    fn hash_from_array_and_deref() {
        let mut arr = [0u8; 32];
        arr[0] = 1;
        arr[31] = 7;

        let hash = Hash::from(arr);
        // `Deref` allows us to treat `Hash` as a slice.
        assert_eq!(&hash[..], &arr[..]);
    }

    #[test]
    fn convert_between_hash_and_vec() {
        let mut vec = vec![0u8; 32];
        vec[0] = 1;
        vec[31] = 7;
        let hash = Hash::from(vec.clone());
        assert_eq!(&hash[..], &vec[..]);

        // Convert back to Vec<u8>
        let vec2: Vec<u8> = hash.into();
        assert_eq!(vec, vec2);
    }

    #[test]
    fn convert_between_hash_and_string() {
        // Test with '0x' prefix
        let str = "0x000036755a024ef491b6710fe765e06e33a616f83b8a33c6a1963ab20f6e5bdb";
        let expected = hex::decode(&str[2..]).unwrap();
        let hash = Hash::try_from(str.to_string()).unwrap();
        assert_eq!(&hash[..], &expected[..]);

        // Test without '0x' prefix
        let str1 = str[2..].to_string();
        let hash = Hash::try_from(str1).unwrap();
        assert_eq!(&hash[..], &expected[..]);

        // Convert back to String
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
