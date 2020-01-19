use crate::{
    error::{ErrorKind, Result, ResultExt},
    ops::Function,
    ByteArray,
};
use secp256k1::{All, Message, Secp256k1, SecretKey};

pub struct Crypto {
    curve: Secp256k1<All>,
}

impl Crypto {
    pub fn new() -> Self {
        Crypto {
            curve: Secp256k1::new(),
        }
    }

    pub fn sign(
        &self,
        secret_key: &[u8],
        msg_array: ByteArray<'static>,
    ) -> Result<ByteArray<'static>> {
        let sk = SecretKey::from_slice(secret_key)
            .chain_err(|| ErrorKind::InvalidSize((32, secret_key.len())))?;
        let msg = Message::from_slice(&msg_array.data)
            .chain_err(|| ErrorKind::InvalidSize((32, msg_array.data.len())))?;
        let sig = self.curve.sign(&msg, &sk).serialize_der().to_vec();
        Ok(msg_array.apply_function(sig.into(), Function::EcdsaSign))
    }
}
