use crate::{BitcoinByteArray, BitcoinDataType, ByteArray, DataType, Op};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Copy)]
pub struct Pubkey([u8; 33]);

impl Pubkey {
    pub fn from_slice(slice: &[u8]) -> Self {
        let mut pubkey = [0; 33];
        pubkey.copy_from_slice(slice);
        Pubkey(pubkey)
    }

    pub fn from_slice_checked(slice: &[u8]) -> Option<Self> {
        let mut pubkey = [0; 33];
        if slice.len() != pubkey.len() {
            return None;
        }
        pubkey.copy_from_slice(slice);
        Some(Pubkey(pubkey))
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
        write!(f, "Pubkey({})", hex::encode(&self.0))
    }
}

impl std::fmt::Display for Pubkey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
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

impl Serialize for Pubkey {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            self.to_string().serialize(serializer)
        } else {
            self.0.to_vec().serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for Pubkey {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let data;
        if deserializer.is_human_readable() {
            let hex = String::deserialize(deserializer)?;
            data = hex::decode(&hex).map_err(serde::de::Error::custom)?;
        } else {
            data = Vec::<u8>::deserialize(deserializer)?;
        }
        let mut pubkey = Pubkey::default();
        if data.len() != pubkey.0.len() {
            return Err(serde::de::Error::invalid_length(data.len(), &"Pubkey must have length 33"));
        }
        pubkey.0.copy_from_slice(&data);
        Ok(pubkey)
    }
}
