use crate::{
    encode_bitcoin_code, encoding_utils, error::Result, ByteArray, Hashed, Script, Sha256d,
    SigHashFlags, ToPreimages, TxPreimage,
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

    #[serde(skip)]
    pub is_p2sh: Option<bool>,
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

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub struct Tx {
    #[serde(skip)]
    unhashed_tx: UnhashedTx,

    #[serde(skip)]
    hash: Sha256d,

    raw: ByteArray,
}

impl TxInput {
    pub fn serialize(&self) -> Result<ByteArray> {
        let vout = ByteArray::new("vout", encode_bitcoin_code(&self.prev_out.vout)?);
        let sequence = ByteArray::new("sequence", encode_bitcoin_code(&self.sequence)?);
        let script = self.script.serialize()?.named("script");
        let mut script_len_ser = Vec::new();
        encoding_utils::write_var_int(&mut script_len_ser, script.len() as u64)?;
        let script_len = ByteArray::new("script_len", script_len_ser);
        Ok(self
            .prev_out
            .tx_hash
            .clone()
            .concat(vout)
            .concat(script_len)
            .concat(script)
            .concat(sequence))
    }
}

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

    pub fn serialize(&self) -> Result<ByteArray> {
        let version = ByteArray::new("version", encode_bitcoin_code(&self.version)?);
        let lock_time = ByteArray::new("lock_time", encode_bitcoin_code(&self.version)?);

        let mut inputs_len_ser = Vec::new();
        encoding_utils::write_var_int(&mut inputs_len_ser, self.inputs.len() as u64)?;
        let inputs_len = ByteArray::new("inputs_len", inputs_len_ser);

        let mut outputs_len_ser = Vec::new();
        encoding_utils::write_var_int(&mut outputs_len_ser, self.outputs.len() as u64)?;
        let outputs_len = ByteArray::new("outputs_len", outputs_len_ser);

        let mut byte_array = version.concat(inputs_len);
        for input in self.inputs.iter() {
            byte_array = byte_array.concat(input.serialize()?);
        }

        byte_array = byte_array.concat(outputs_len);
        for output in self.outputs.iter() {
            byte_array = byte_array.concat(output.serialize()?);
        }

        Ok(byte_array.concat(lock_time))
    }

    pub fn hashed(self) -> Tx {
        let raw = self.serialize().expect("Couldn't encode UnhashedTx");
        let hash = Sha256d::digest(raw.clone());
        Tx {
            unhashed_tx: self,
            raw,
            hash,
        }
    }
}

impl Tx {
    pub fn hash(&self) -> &Sha256d {
        &self.hash
    }

    pub fn raw(&self) -> &ByteArray {
        &self.raw
    }

    pub fn version(&self) -> i32 {
        self.unhashed_tx.version
    }

    pub fn inputs(&self) -> &[TxInput] {
        &self.unhashed_tx.inputs
    }

    pub fn outputs(&self) -> &[TxOutput] {
        &self.unhashed_tx.outputs
    }

    pub fn lock_time(&self) -> u32 {
        self.unhashed_tx.lock_time
    }

    pub fn preimages(&self, sig_hash_flags: &[SigHashFlags]) -> Vec<Vec<TxPreimage>> {
        self.unhashed_tx.preimages(sig_hash_flags)
    }
}

impl Default for UnhashedTx {
    fn default() -> Self {
        UnhashedTx {
            version: 1,
            inputs: vec![],
            outputs: vec![],
            lock_time: 0,
        }
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
