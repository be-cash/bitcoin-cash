use secp256k1::{Message, SecretKey};
use crate::{
    ByteArray,
    error::{Result, ResultExt, ErrorKind},
    ops::Function,
};

pub struct Crypto;

impl Crypto {
    pub fn new() -> Self {
        Crypto
    }

    pub fn sign(&self, secret_key: &[u8], msg_array: ByteArray<'static>) -> Result<ByteArray<'static>> {
        let sk = SecretKey::parse_slice(secret_key)
            .chain_err(|| ErrorKind::InvalidSize((32, secret_key.len())))?;
        let msg = Message::parse_slice(&msg_array.data)
            .chain_err(|| ErrorKind::InvalidSize((32, msg_array.data.len())))?;
        let sig = secp256k1::sign(&msg, &sk).0.serialize_der().as_ref().to_vec();
        Ok(msg_array.apply_function(sig.into(), Function::EcdsaSign))
    }
}
