//! A wallet for managing accounts and signing messages.
//!
//! This project is primarily focused on the `tinychain` (the root package),
//! so for simplicity this crate wraps the Ethereum wallet instead of implementing a new one.
//!
//! In addition, to facilitate a smooth demonstration of tinychain functionality, we store the keystore locally and use a
//! default user password (774411), allowing the tinychain to sign a transaction or block without user intervention.

use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use ethers_core::{rand::thread_rng, types::Signature, utils::hash_message};
use ethers_signers::{LocalWallet, Signer};
use once_cell::sync::OnceCell;

pub mod error;
pub use error::WalletError;

const PASSWORD: &str = "774411";
static KEYSTORE_DIR: OnceCell<PathBuf> = OnceCell::new();

pub fn init_keystore_dir(datadir: impl AsRef<Path>) {
    let mut path = PathBuf::from(datadir.as_ref());
    path.push("keystore");

    KEYSTORE_DIR.get_or_init(|| path);
}

pub fn new_account() -> Result<String, WalletError> {
    let dir = get_keystore_dir();
    fs::create_dir_all(&dir)?;

    let (_, account) = LocalWallet::new_keystore(&dir, &mut thread_rng(), PASSWORD, None).unwrap();
    Ok(account)
}

pub fn sign(msg: &str, account: &str) -> Result<String, WalletError> {
    let sig = get_wallet(account)?
        .sign_hash(hash_message(msg))?
        .to_string();

    Ok(sig)
}

pub fn verify(msg: &str, sig: &str, account: &str) -> Result<(), WalletError> {
    let wallet = get_wallet(account)?;
    let sig = Signature::from_str(sig)?;
    sig.verify(msg, wallet.address())?;

    Ok(())
}

pub fn get_keystore_dir() -> PathBuf {
    KEYSTORE_DIR.get().unwrap().to_owned()
}

fn get_wallet(account: &str) -> Result<LocalWallet, WalletError> {
    let mut keypath = get_keystore_dir();
    keypath.push(account);

    LocalWallet::decrypt_keystore(&keypath, PASSWORD)
        .map_err(|_| WalletError::AccountNotFound(account.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keystore_dir_can_only_be_initialized_once() {
        let mut tmpdir = tempdir_with_prefix("tmp");
        let _ = fs::remove_dir_all(&tmpdir);

        init_keystore_dir(tmpdir.to_str().unwrap());
        tmpdir.push("keystore");
        assert_eq!(tmpdir, get_keystore_dir());

        init_keystore_dir("/another/dir");
        assert_eq!(tmpdir, get_keystore_dir());
    }

    #[test]
    fn wallet_works() {
        let tmpdir = tempdir_with_prefix("tmp");
        let _ = fs::remove_dir_all(&tmpdir);

        init_keystore_dir(tmpdir.to_str().unwrap());
        let acc = new_account().unwrap();
        let msg = "hello world";
        let sig = sign(msg, &acc).unwrap();
        assert!(verify(msg, &sig, &acc).is_ok());
    }

    fn tempdir_with_prefix(prefix: &str) -> PathBuf {
        tempfile::Builder::new()
            .prefix(prefix)
            .tempdir()
            .unwrap()
            .into_path()
    }
}
