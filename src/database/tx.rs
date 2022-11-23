use serde::{Deserialize, Serialize};
use std::time::SystemTime;

use crate::wallet;

const GAS: u64 = 21;
const GAS_PRICE: u64 = 21;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tx {
    pub(super) from: String,
    pub(super) to: String,
    pub(super) value: u64,
    pub(super) data: String,
    pub(super) nonce: u64,
    pub(super) gas: u64,
    pub(super) gas_price: u64,
    pub(super) time: u64,
}

impl Tx {
    pub fn builder() -> TxBuilder {
        TxBuilder::default()
    }

    pub fn gas_cost(&self) -> u64 {
        self.gas * self.gas_price
    }

    pub fn cost(&self) -> u64 {
        self.value * self.gas_cost()
    }

    pub fn encode(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn sign(self) -> SignedTx {
        let sig = wallet::sign(&self.encode(), &self.from).unwrap();
        SignedTx { tx: self, sig: sig }
    }
}

#[derive(Debug, Default)]
pub struct TxBuilder {
    pub(super) from: String,
    pub(super) to: String,
    pub(super) value: u64,
    pub(super) data: String,
    pub(super) nonce: u64,
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

    pub fn data(mut self, data: &str) -> Self {
        self.data = data.to_owned();
        self
    }

    pub fn nonce(mut self, nonce: u64) -> Self {
        self.nonce = nonce;
        self
    }

    pub fn build(self) -> Tx {
        let time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Tx {
            from: self.from,
            to: self.to,
            value: self.value,
            data: self.data,
            nonce: self.nonce,
            gas: GAS,
            gas_price: GAS_PRICE,
            time: time,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SignedTx {
    pub(super) tx: Tx,
    pub(super) sig: String,
}

impl SignedTx {
    pub fn is_valid_signature(&self) -> bool {
        wallet::verify(&self.tx.encode(), &self.sig, &self.tx.from).is_ok()
    }

    pub fn gas_cost(&self) -> u64 {
        self.tx.gas_cost()
    }

    pub fn cost(&self) -> u64 {
        self.tx.cost()
    }
}
