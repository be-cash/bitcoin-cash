use crate::{
    encode_bitcoin_code, encoding_utils, error::Result, ByteArray, Script, Sha256d, SigHashFlags,
    ToPreimages, TxPreimage,
};
use serde_derive::{Deserialize, Serialize};

pub const DEFAULT_SEQUENCE: u32 = 0xffff_ffff;
pub const MAX_SIGNATURE_SIZE: usize = 73; // explained https://bitcoin.stackexchange.com/a/77192

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone, Default)]
pub struct TxOutpoint {
    pub tx_hash: Sha256d,
    pub vout: u32,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct TxInput {
    pub prev_out: TxOutpoint,
    pub script: Script,
    pub sequence: u32,

    #[serde(skip)]
    pub lock_script: Option<Script>,

    #[serde(skip)]
    pub value: Option<u64>,
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

impl TxInput {}

impl TxOutput {
    pub fn serialize(&self) -> Result<ByteArray> {
        let value = ByteArray::new("value", encode_bitcoin_code(&self.value)?);
        let script = self.script.serialize()?.named("script");
        let mut script_len_ser = Vec::new();
        encoding_utils::write_var_int(&mut script_len_ser, script.len() as u64)?;
        let script_len = ByteArray::new("script_len", script_len_ser);
        Ok(value.concat(script_len).concat(script))
    }
}

impl UnhashedTx {
    pub fn preimages(&self, sig_hash_flags: &[SigHashFlags]) -> Vec<Vec<TxPreimage>> {
        TxPreimage::build_preimages(&SigTxPreimage {
            tx: self,
            sig_hash_flags,
        })
    }
}

struct SigTxPreimage<'b> {
    tx: &'b UnhashedTx,
    sig_hash_flags: &'b [SigHashFlags],
}

impl ToPreimages for SigTxPreimage<'_> {
    fn version(&self) -> i32 {
        self.tx.version
    }
    fn num_inputs(&self) -> usize {
        self.tx.inputs.len()
    }
    fn input_outpoint_at(&self, input_idx: usize) -> &TxOutpoint {
        &self.tx.inputs[input_idx].prev_out
    }
    fn input_sequence_at(&self, input_idx: usize) -> u32 {
        self.tx.inputs[input_idx].sequence
    }
    fn input_sig_hash_flags_at(&self, _input_idx: usize) -> &[SigHashFlags] {
        &self.sig_hash_flags
    }
    fn input_value_at(&self, input_idx: usize) -> u64 {
        self.tx.inputs[input_idx]
            .value
            .expect("No known value for input")
    }
    fn input_lock_script_at(&self, input_idx: usize) -> Script {
        self.tx.inputs[input_idx]
            .lock_script
            .clone()
            .expect("No known lock_script for input")
    }
    fn num_outputs(&self) -> usize {
        self.tx.outputs.len()
    }
    fn output_at(&self, output_idx: usize) -> &TxOutput {
        &self.tx.outputs[output_idx]
    }
    fn lock_time(&self) -> u32 {
        self.tx.lock_time
    }
}
