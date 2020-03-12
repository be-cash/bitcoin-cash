use crate::{Script, Sha256d, ByteArray, encode_bitcoin_code, error::Result};
use serde_derive::{Deserialize, Serialize};

pub const DEFAULT_SEQUENCE: u32 = 0xffff_ffff;
pub const MAX_SIGNATURE_SIZE: usize = 73; // explained https://bitcoin.stackexchange.com/a/77192

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct TxOutpoint {
    pub tx_hash: Sha256d,
    pub vout: u32,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct TxInput {
    pub prev_out: TxOutpoint,
    pub script: Script,
    pub sequence: u32,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct TxOutput {
    pub value: u64,
    pub script: Script,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct UnhashedTx {
    pub version: i32,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub lock_time: u32,
}

impl TxInput {

}

impl TxOutput {
    pub fn serialize(&self) -> Result<ByteArray> {
        let value = ByteArray::new("value", encode_bitcoin_code(&self.value)?);
        Ok(value.concat(self.script.serialize()?.named("script")))
    }
}
