use crate::{error::Result, ByteArray, Pubkey};

pub trait ECC: Default {
    fn sign(&self, secret_key: &[u8], msg_array: ByteArray<'static>) -> Result<ByteArray<'static>>;

    fn verify(&self, pubkey: &[u8], msg_array: &[u8], sig: &[u8]) -> Result<bool>;

    fn derive_pubkey(&self, secret_key: &[u8]) -> Result<Pubkey>;

    fn normalize_sig(&self, sig: &[u8]) -> Result<Vec<u8>>;
}
