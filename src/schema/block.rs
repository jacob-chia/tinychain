use std::fmt;

use prost::Message;

use crate::{error::Error, types::Hash, utils};

use super::{Block, BlockHeader, SignedTx};

const BLOCK_REWORD: u64 = 100;

impl Block {
    pub fn new(parent_hash: Hash, number: u64, author: String, txs: Vec<SignedTx>) -> Self {
        Self {
            header: Some(BlockHeader {
                number,
                parent_hash: parent_hash.into(),
                nonce: utils::gen_random_number(),
                timestamp: utils::unix_timestamp(),
                author,
            }),
            txs,
        }
    }

    pub fn hash(&self) -> crate::types::Hash {
        utils::hash_message(&self.encode_to_vec())
    }

    pub fn block_reward(&self) -> u64 {
        let gas_reward: u64 = self.txs.iter().map(|tx| tx.gas_cost()).sum();
        gas_reward + BLOCK_REWORD
    }

    pub fn update_nonce_and_time(&mut self) {
        self.header.as_mut().unwrap().nonce = utils::gen_random_number();
        self.header.as_mut().unwrap().timestamp = utils::unix_timestamp();
    }

    pub fn nonce(&self) -> u64 {
        self.header.as_ref().unwrap().nonce
    }

    pub fn timestamp(&self) -> u64 {
        self.header.as_ref().unwrap().timestamp
    }

    pub fn number(&self) -> u64 {
        self.header.as_ref().unwrap().number
    }

    pub fn parent_hash(&self) -> Hash {
        Hash::from(self.header.as_ref().unwrap().parent_hash.clone())
    }

    pub fn author(&self) -> &str {
        self.header.as_ref().unwrap().author.as_str()
    }
}

impl TryFrom<Vec<u8>> for Block {
    type Error = Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(Self::decode(value.as_slice())?)
    }
}

impl From<&Block> for Vec<u8> {
    fn from(value: &Block) -> Self {
        value.encode_to_vec()
    }
}

// For better logging.
impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let header = self.header.as_ref().unwrap();
        write!(f, "Block {{ header: {}, txs: [", header)?;
        for i in 0..self.txs.len() {
            write!(f, "{}", self.txs[i])?;
            if i != self.txs.len() - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, "] }}")
    }
}

// For better logging.
impl fmt::Display for BlockHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "BlockHeader {{ number: {}, parent_hash: 0x{}, nonce: {}, timestamp: {}, author: \"{}\" }}",
            self.number,
            &hex::encode(&self.parent_hash),
            self.nonce,
            self.timestamp,
            self.author,
        )
    }
}
