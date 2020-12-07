use bitcoin_cash::{
    error::{Error, Result},
    ByteArray, Function, Pubkey, ECC,
};
use secp256k1::{All, Message, PublicKey, Secp256k1, SecretKey, Signature};

#[derive(Clone)]
pub struct CECC {
    pub curve: Secp256k1<All>,
}

impl Default for CECC {
    fn default() -> Self {
        CECC {
            curve: Secp256k1::new(),
        }
    }
}

impl ECC for CECC {
    fn sign(&self, secret_key: &[u8], msg_array: impl Into<ByteArray>) -> Result<ByteArray> {
        let msg_array = msg_array.into();
        let sk = SecretKey::from_slice(secret_key).map_err(|_| Error::InvalidSize {
            expected: 32,
            actual: secret_key.len(),
        })?;
        let msg = Message::from_slice(&msg_array).map_err(|_| Error::InvalidSize {
            expected: 32,
            actual: msg_array.len(),
        })?;
        let sig = self.curve.sign(&msg, &sk).serialize_der().to_vec();
        Ok(msg_array.apply_function(sig, Function::EcdsaSign))
    }

    fn verify(&self, pubkey: &[u8], msg_array: &[u8], sig_ser: &[u8]) -> Result<bool> {
        let msg = Message::from_slice(msg_array).map_err(|_| Error::InvalidSize {
            expected: 32,
            actual: msg_array.len(),
        })?;
        let sig = Signature::from_der(sig_ser).map_err(|_| Error::InvalidSignatureFormat)?;
        let pubkey = PublicKey::from_slice(pubkey).map_err(|_| Error::InvalidPubkey)?;
        match self.curve.verify(&msg, &sig, &pubkey) {
            Ok(()) => Ok(true),
            Err(secp256k1::Error::IncorrectSignature) => Ok(false),
            err => {
                err.map_err(|_| Error::InvalidSignatureFormat)?;
                unreachable!()
            }
        }
    }

    fn derive_pubkey(&self, secret_key: &[u8]) -> Result<Pubkey> {
        let sk = SecretKey::from_slice(secret_key).map_err(|_| Error::InvalidSize {
            expected: 32,
            actual: secret_key.len(),
        })?;
        Ok(Pubkey::new(
            PublicKey::from_secret_key(&self.curve, &sk).serialize(),
        ))
    }

    fn normalize_sig(&self, sig_ser: &[u8]) -> Result<Vec<u8>> {
        let mut sig =
            Signature::from_der_lax(sig_ser).map_err(|_| Error::InvalidSignatureFormat)?;
        sig.normalize_s();
        Ok(sig.serialize_der().to_vec())
    }
}
