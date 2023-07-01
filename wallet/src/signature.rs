//! Signature type for transactions.
//!
//! A signature is a 65-byte array, where the first 64 bytes is the `ecdsa::Signature`,
//! and the last byte is the `ecdsa::RecoveryId`.

use std::ops::Deref;

use k256::ecdsa;

use crate::WalletError;

#[derive(Clone, Copy)]
pub struct Signature([u8; 65]);

impl Deref for Signature {
    type Target = [u8; 65];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<(ecdsa::Signature, ecdsa::RecoveryId)> for Signature {
    fn from((sig, recid): (ecdsa::Signature, ecdsa::RecoveryId)) -> Self {
        let mut bytes = [0u8; 65];
        bytes[..64].copy_from_slice(sig.to_bytes().as_ref());
        bytes[64] = recid.to_byte();
        Self(bytes)
    }
}

impl TryFrom<&Signature> for (ecdsa::Signature, ecdsa::RecoveryId) {
    type Error = WalletError;

    fn try_from(value: &Signature) -> Result<Self, Self::Error> {
        let sig = ecdsa::Signature::from_bytes(value[..64].as_ref().into())
            .map_err(|_| WalletError::InvalidSignature)?;

        let recid = ecdsa::RecoveryId::from_byte(value[64]).ok_or(WalletError::InvalidSignature)?;

        Ok((sig, recid))
    }
}

impl From<[u8; 65]> for Signature {
    fn from(bytes: [u8; 65]) -> Self {
        Self(bytes)
    }
}

impl From<Signature> for [u8; 65] {
    fn from(signature: Signature) -> Self {
        signature.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use hex_literal::hex;

    #[test]
    fn signature_from() {
        let bytes = hex!(
                "ce53abb3721bafc561408ce8ff99c909f7f0b18a2f788649d6470162ab1aa032"
                "3971edc523a6d6453f3fb6128d318d9db1a5ff3386feb1047d9816e780039d5201"
        );

        let sig = ecdsa::Signature::from_bytes((&bytes[..64]).into()).unwrap();
        let recid = ecdsa::RecoveryId::from_byte(0x01).unwrap();

        let s1 = Signature::from((sig, recid));
        assert_eq!(&s1[..], &bytes[..]);
        let s2 = Signature::from(bytes);
        assert_eq!(&s2[..], &bytes[..]);
    }

    #[test]
    fn signature_into() {
        let bytes = hex!(
                "ce53abb3721bafc561408ce8ff99c909f7f0b18a2f788649d6470162ab1aa032"
                "3971edc523a6d6453f3fb6128d318d9db1a5ff3386feb1047d9816e780039d5201"
        );
        let signature = Signature(bytes);

        let (sig, recid): (ecdsa::Signature, ecdsa::RecoveryId) = (&signature).try_into().unwrap();
        assert_eq!(&bytes[..64], &sig.to_bytes()[..]);
        assert_eq!(bytes[64], recid.to_byte());

        let bytes2: [u8; 65] = signature.into();
        assert_eq!(&bytes[..], &bytes2[..]);
    }
}
