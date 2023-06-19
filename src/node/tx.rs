use std::ops::Deref;

use serde::{Deserialize, Serialize};

use crate::{error::Error, types::Hash, utils, wallet};

const GAS: u64 = 21;
const GAS_PRICE: u64 = 1;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Tx {
    pub from: String,
    pub to: String,
    pub value: u64,
    pub nonce: u64,
    pub gas: u64,
    pub gas_price: u64,
    pub timestamp: u64,
}

impl Tx {
    pub fn builder() -> TxBuilder {
        TxBuilder::default()
    }

    pub fn gas_cost(&self) -> u64 {
        self.gas * self.gas_price
    }

    pub fn cost(&self) -> u64 {
        self.value + self.gas_cost()
    }

    pub fn encode(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn hash(&self) -> Hash {
        utils::hash_message(&self.encode())
    }

    pub fn sign(self) -> Result<SignedTx, Error> {
        let sig = wallet::sign(&self.encode(), &self.from)?;
        Ok(SignedTx { tx: self, sig })
    }
}

#[derive(Debug, Default, Clone)]
pub struct TxBuilder {
    pub from: String,
    pub to: String,
    pub value: u64,
    pub nonce: u64,
}

impl TxBuilder {
    pub fn from(mut self, from: &str) -> Self {
        self.from = from.to_owned();
        self
    }

    pub fn to(mut self, to: &str) -> Self {
        self.to = to.to_owned();
        self
    }

    pub fn value(mut self, value: u64) -> Self {
        self.value = value;
        self
    }

    pub fn nonce(mut self, nonce: u64) -> Self {
        self.nonce = nonce;
        self
    }

    pub fn build(self) -> Tx {
        Tx {
            from: self.from,
            to: self.to,
            value: self.value,
            nonce: self.nonce,
            gas: GAS,
            gas_price: GAS_PRICE,
            timestamp: utils::unix_timestamp(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct SignedTx {
    pub tx: Tx,
    pub sig: String,
}

impl Deref for SignedTx {
    type Target = Tx;

    fn deref(&self) -> &Self::Target {
        &self.tx
    }
}

impl SignedTx {
    pub fn check_signature(&self) -> Result<(), Error> {
        wallet::verify(&self.tx.encode(), &self.sig, &self.tx.from)
            .map_err(|_| Error::InvalidTxSignature(self.from.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx_builder() {
        let tx = Tx::builder()
            .from("alice")
            .to("bob")
            .value(100)
            .nonce(1)
            .build();

        assert_eq!("alice", tx.from);
        assert_eq!("bob", tx.to);
        assert_eq!(100, tx.value);
        assert_eq!(1, tx.nonce);
    }
}
