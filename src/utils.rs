use std::time::SystemTime;

use rand::{thread_rng, Rng};
use tiny_keccak::{Hasher, Keccak};

use crate::types::{Bytes, Hash};

pub fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn hash_message(msg: &[u8]) -> Hash {
    let mut output = [0u8; 32];

    let mut hasher = Keccak::v256();
    hasher.update(msg);
    hasher.finalize(&mut output);

    Bytes::from(output)
}

pub fn gen_random_number() -> u64 {
    thread_rng().gen::<u64>()
}
