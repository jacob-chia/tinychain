use serde::{Deserialize, Serialize};
use std::time::SystemTime;

use crate::wallet;

const GAS: u64 = 21;
const GAS_PRICE: u64 = 21;

#[derive(Debug, Serialize, Deserialize)]
pub struct Tx {
    from: String,
    to: String,
    value: u64,
    data: String,
    nonce: u64,
    gas: u64,
    gas_price: u64,
    time: u64,
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
    from: String,
    to: String,
    value: u64,
    data: String,
    nonce: u64,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct SignedTx {
    tx: Tx,
    sig: String,
}

impl SignedTx {
    pub fn is_authentic(&self) -> bool {
        wallet::verify(&self.tx.encode(), &self.sig, &self.tx.from).is_ok()
    }

    pub fn gas_cost(&self) -> u64 {
        self.tx.gas_cost()
    }

    pub fn cost(&self) -> u64 {
        self.tx.cost()
    }
}
