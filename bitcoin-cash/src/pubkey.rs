use crate::{BitcoinByteArray, BitcoinDataType, ByteArray, DataType, Op};

#[derive(Clone, Copy)]
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

impl std::fmt::Debug for Pubkey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "Pubkey({})", hex::encode(&self.0[..]))
    }
}

impl Default for Pubkey {
    fn default() -> Self {
        Pubkey([0; 33])
    }
}

impl BitcoinDataType for Pubkey {
    type Type = BitcoinByteArray;
    fn to_data(&self) -> Self::Type {
        BitcoinByteArray(self.as_byte_array())
    }
    fn to_pushop(&self) -> Op {
        self.as_byte_array().into()
    }
    fn to_data_type(&self) -> DataType {
        DataType::ByteArray(Some(self.0.len()))
    }
}
