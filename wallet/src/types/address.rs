//! Address type for accounts.

use std::{
    fmt::{self, Debug},
    ops::Deref,
};

use serde::{Deserialize, Serialize};

// Serialize and deserialize Address as a '0x'-prefixed hex string.
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Address([u8; 20]);

impl Address {
    fn fmt_as_hex(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x")?;
        for byte in self.0.iter() {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

// Implement Debug so that Address can be printed as a hex string.
impl Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_as_hex(f)
    }
}

// Implement Display so that Address can be printed as a hex string.
impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_as_hex(f)
    }
}

// Implement Deref so that Address can be used as &[u8; 20]
impl Deref for Address {
    type Target = [u8; 20];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<[u8; 20]> for Address {
    fn from(bytes: [u8; 20]) -> Self {
        Self(bytes)
    }
}

impl From<Vec<u8>> for Address {
    fn from(bytes: Vec<u8>) -> Self {
        let mut addr = [0u8; 20];
        addr.copy_from_slice(bytes.as_slice());
        Self(addr)
    }
}

impl TryFrom<&str> for Address {
    type Error = hex::FromHexError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let val = if value.starts_with("0x") {
            &value[2..]
        } else {
            &value
        };

        hex::decode(val).map(Self::from)
    }
}

impl TryFrom<String> for Address {
    type Error = hex::FromHexError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl From<Address> for Vec<u8> {
    fn from(addr: Address) -> Self {
        addr.0.to_vec()
    }
}

impl From<Address> for String {
    fn from(addr: Address) -> Self {
        let mut s = String::from("0x");
        s.push_str(&hex::encode(addr.0));
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_from() {
        let bytes = [
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd,
            0xee, 0xff, 0x11, 0x22, 0x33, 0x44,
        ];

        let addr1 = Address::from(bytes);
        let addr2 = Address::from(bytes.to_vec());
        assert_eq!(&addr1[..], &bytes[..]);
        assert_eq!(&addr2[..], &bytes[..]);

        let addr3 = hex::encode(&bytes);
        let addr4 = Address::try_from(addr3).unwrap();
        assert_eq!(&addr4[..], &bytes[..]);
    }

    #[test]
    fn address_into() {
        let addr = Address::try_from("0x00112233445566778899aabbccddeeff11223344").unwrap();

        let addr_vec: Vec<u8> = addr.into();
        assert_eq!(&addr_vec[..], &addr[..]);

        let addr_str: String = addr.into();
        assert_eq!(addr_str, "0x00112233445566778899aabbccddeeff11223344");
    }

    #[test]
    fn address_debug_display() {
        let str = "0x00112233445566778899aabbccddeeff11223344";
        let addr = Address::try_from(str).unwrap();

        let debug_str = format!("{:?}", addr);
        assert_eq!(debug_str, str);
        let display_str = format!("{}", addr);
        assert_eq!(display_str, str);
    }
}
