use std::{fmt, ops::Deref};

use prost::Message;

use crate::{
    error::Error,
    types::{Hash, Signature},
    utils,
};

use super::{SignedTx, Tx};

const GAS: u64 = 21;
const GAS_PRICE: u64 = 1;

impl Tx {
    pub fn new(from: &str, to: &str, value: u64, nonce: u64) -> Self {
        Self {
            from: from.to_owned(),
            to: to.to_owned(),
            value,
            nonce,
            gas: GAS,
            gas_price: GAS_PRICE,
            timestamp: utils::unix_timestamp(),
        }
    }

    /// The gas cost will be charged from the sender and given to the miner.
    pub fn gas_cost(&self) -> u64 {
        self.gas * self.gas_price
    }

    /// The total cost of the transaction.
    pub fn cost(&self) -> u64 {
        self.value + self.gas_cost()
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        self.encode_to_vec()
    }

    pub fn hash(&self) -> Hash {
        utils::hash_message(&self.as_bytes())
    }
}

impl Deref for SignedTx {
    type Target = Tx;

    fn deref(&self) -> &Self::Target {
        self.tx.as_ref().unwrap()
    }
}

impl TryFrom<Vec<u8>> for SignedTx {
    type Error = Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(Self::decode(value.as_slice())?)
    }
}

impl From<SignedTx> for Vec<u8> {
    fn from(value: SignedTx) -> Self {
        value.encode_to_vec()
    }
}

impl From<Tx> for Vec<u8> {
    fn from(value: Tx) -> Self {
        value.encode_to_vec()
    }
}

// For better logging.
// `fmt::Debug` is implemented by prost, we can't implement it manually.
impl fmt::Display for SignedTx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SignedTx{{ tx: {:?}, sig: {:?} }}",
            self.tx.as_ref().unwrap(),
            Signature::from(self.sig.clone())
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx() {
        let tx = Tx::new("from", "to", 100, 0);
        assert_eq!(tx.gas_cost(), 21);
        assert_eq!(tx.cost(), 121);
    }
}
