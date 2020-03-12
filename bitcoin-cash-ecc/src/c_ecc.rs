use bitcoin_cash::{
    error::{ErrorKind, Result, ResultExt},
    ByteArray, Function, Pubkey, ECC,
};
use secp256k1::{All, Error, Message, PublicKey, Secp256k1, SecretKey, Signature};

pub struct CECC {
    curve: Secp256k1<All>,
}

impl Default for CECC {
    fn default() -> Self {
        CECC {
            curve: Secp256k1::new(),
        }
    }
}

impl ECC for CECC {
    fn sign(&self, secret_key: &[u8], msg_array: ByteArray) -> Result<ByteArray> {
        let sk = SecretKey::from_slice(secret_key)
            .chain_err(|| ErrorKind::InvalidSize((32, secret_key.len())))?;
        let msg = Message::from_slice(&msg_array)
            .chain_err(|| ErrorKind::InvalidSize((32, msg_array.len())))?;
        let sig = self.curve.sign(&msg, &sk).serialize_der().to_vec();
        Ok(msg_array.apply_function(sig, Function::EcdsaSign))
    }

    fn verify(&self, pubkey: &[u8], msg_array: &[u8], sig_ser: &[u8]) -> Result<bool> {
        let msg = Message::from_slice(msg_array)
            .chain_err(|| ErrorKind::InvalidSize((32, msg_array.len())))?;
        let sig = Signature::from_der(sig_ser).chain_err(|| ErrorKind::InvalidSignatureFormat)?;
        let pubkey = PublicKey::from_slice(pubkey).chain_err(|| ErrorKind::InvalidPubkey)?;
        match self.curve.verify(&msg, &sig, &pubkey) {
            Ok(()) => Ok(true),
            Err(Error::InvalidSignature) => Ok(false),
            err => {
                err.chain_err(|| ErrorKind::InvalidSignatureFormat)?;
                unreachable!()
            }
        }
    }

    fn derive_pubkey(&self, secret_key: &[u8]) -> Result<Pubkey> {
        let sk = SecretKey::from_slice(secret_key)
            .chain_err(|| ErrorKind::InvalidSize((32, secret_key.len())))?;
        Ok(Pubkey::new(
            PublicKey::from_secret_key(&self.curve, &sk).serialize(),
        ))
    }

    fn normalize_sig(&self, sig_ser: &[u8]) -> Result<Vec<u8>> {
        let mut sig =
            Signature::from_der_lax(sig_ser).chain_err(|| ErrorKind::InvalidSignatureFormat)?;
        sig.normalize_s();
        Ok(sig.serialize_der().to_vec())
    }
}
