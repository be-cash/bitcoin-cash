use bitcoin_cash::{
    error::{Error, Result},
    ByteArray, Function, Pubkey, ECC,
};
use secp256k1::{Message, PublicKey, PublicKeyFormat, SecretKey, Signature};

pub struct RustECC;

impl Default for RustECC {
    fn default() -> Self {
        RustECC
    }
}

impl ECC for RustECC {
    fn sign(&self, secret_key: &[u8], msg_array: impl Into<ByteArray>) -> Result<ByteArray> {
        let msg_array = msg_array.into();
        let sk = SecretKey::parse_slice(secret_key).map_err(|_| Error::InvalidSize {
            expected: 32,
            actual: secret_key.len(),
        })?;
        let msg = Message::parse_slice(&msg_array).map_err(|_| Error::InvalidSize {
            expected: 32,
            actual: msg_array.len(),
        })?;
        let mut sig = secp256k1::sign(&msg, &sk).0;
        sig.normalize_s();
        let sig = sig.serialize_der().as_ref().to_vec();
        Ok(msg_array.apply_function(sig, Function::EcdsaSign))
    }

    fn verify(&self, pubkey: &[u8], msg_array: &[u8], sig_ser: &[u8]) -> Result<bool> {
        let msg = Message::parse_slice(msg_array).map_err(|_| Error::InvalidSize {
            expected: 32,
            actual: msg_array.len(),
        })?;
        let sig = Signature::parse_der(sig_ser).map_err(|_| Error::InvalidSignatureFormat)?;
        let pubkey = PublicKey::parse_slice(pubkey, Some(PublicKeyFormat::Compressed))
            .map_err(|_| Error::InvalidPubkey)?;
        Ok(secp256k1::verify(&msg, &sig, &pubkey))
    }

    fn derive_pubkey(&self, secret_key: &[u8]) -> Result<Pubkey> {
        let sk = SecretKey::parse_slice(secret_key).map_err(|_| Error::InvalidSize {
            expected: 32,
            actual: secret_key.len(),
        })?;
        Ok(Pubkey::new(
            PublicKey::from_secret_key(&sk).serialize_compressed(),
        ))
    }

    fn normalize_sig(&self, sig_ser: &[u8]) -> Result<Vec<u8>> {
        let mut sig =
            Signature::parse_der_lax(sig_ser).map_err(|_| Error::InvalidSignatureFormat)?;
        sig.normalize_s();
        Ok(sig.serialize_der().as_ref().to_vec())
    }
}
