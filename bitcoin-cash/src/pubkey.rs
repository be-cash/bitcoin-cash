use crate::ByteArray;

pub struct Pubkey([u8; 33]);

impl Pubkey {
    pub fn from_slice(slice: &[u8]) -> Self {
        let mut pubkey = [0; 33];
        pubkey.copy_from_slice(slice);
        Pubkey(pubkey)
    }

    pub fn new(pubkey: [u8; 33]) -> Self {
        Pubkey(pubkey)
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    pub fn as_byte_array(&self) -> ByteArray {
        ByteArray::from_slice_unnamed(&self.0)
    }
}

impl Clone for Pubkey {
    fn clone(&self) -> Self {
        let mut pubkey = [0; 33];
        pubkey.copy_from_slice(&self.0);
        Pubkey(pubkey)
    }
}

impl std::fmt::Debug for Pubkey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "Pubkey({})", hex::encode(&self.0[..]))
    }
}
