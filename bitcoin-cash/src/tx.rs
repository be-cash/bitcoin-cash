use crate::Script;
use crate::Sha256d;
use serde_derive::{Deserialize, Serialize};

pub const DEFAULT_SEQUENCE: u32 = 0xffff_ffff;
pub const MAX_SIGNATURE_SIZE: usize = 73; // explained https://bitcoin.stackexchange.com/a/77192

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct TxOutpoint {
    pub tx_hash: Sha256d,
    pub vout: u32,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct TxInput<'a> {
    pub prev_out: TxOutpoint,
    pub script: Script<'a>,
    pub sequence: u32,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct TxOutput<'a> {
    pub value: u64,
    pub script: Script<'a>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct UnhashedTx<'a> {
    pub version: i32,
    pub inputs: Vec<TxInput<'a>>,
    pub outputs: Vec<TxOutput<'a>>,
    pub lock_time: u32,
}

impl TxOutput<'_> {
    pub fn to_owned_output(&self) -> TxOutput<'static> {
        TxOutput {
            value: self.value,
            script: self.script.to_owned_script(),
        }
    }
}

impl TxInput<'_> {
    pub fn to_owned_input(&self) -> TxInput<'static> {
        TxInput {
            prev_out: self.prev_out.clone(),
            script: self.script.to_owned_script(),
            sequence: self.sequence,
        }
    }
}

impl UnhashedTx<'_> {
    pub fn to_owned_tx(&self) -> UnhashedTx<'static> {
        UnhashedTx {
            version: self.version,
            inputs: self.inputs.iter().map(TxInput::to_owned_input).collect(),
            outputs: self.outputs.iter().map(TxOutput::to_owned_output).collect(),
            lock_time: self.lock_time,
        }
    }
}
