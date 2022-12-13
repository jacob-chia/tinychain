use ethers_core::utils::hash_message;
use serde::{Deserialize, Serialize};

use crate::{error::ChainError, types::Hash, utils, wallet};

const GAS: u64 = 21;
const GAS_PRICE: u64 = 1;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tx {
    pub from: String,
    pub to: String,
    pub value: u64,
    pub nonce: u64,
    pub gas: u64,
    pub gas_price: u64,
    pub time: u64,
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
        hash_message(self.encode())
    }

    pub fn sign(self) -> Result<SignedTx, ChainError> {
        let sig = wallet::sign(&self.encode(), &self.from)?;
        Ok(SignedTx { tx: self, sig: sig })
    }
}

#[derive(Debug, Default)]
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
            time: utils::unix_timestamp(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SignedTx {
    pub tx: Tx,
    pub sig: String,
}

impl SignedTx {
    pub fn check_signature(&self) -> Result<(), ChainError> {
        wallet::verify(&self.tx.encode(), &self.sig, &self.tx.from)
    }

    pub fn gas_cost(&self) -> u64 {
        self.tx.gas_cost()
    }

    pub fn hash(&self) -> Hash {
        self.tx.hash()
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
