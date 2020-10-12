use crate::{
    BitcoinCode, ByteArray, Hashed, Script, Sha256d, SigHashFlags, ToPreimages, TxPreimage,
};

pub const DEFAULT_SEQUENCE: u32 = 0xffff_ffff;
// Mark Lundeberg: "71 bytes for the DER, but then +1 for the hashtype,
// so 72 bytes for the full tx signature."
pub const MAX_SIGNATURE_SIZE: usize = 72;

#[bitcoin_code(crate = "crate")]
#[derive(BitcoinCode, PartialEq, Debug, Clone, Default)]
pub struct TxOutpoint {
    pub tx_hash: Sha256d,
    pub vout: u32,
}

#[bitcoin_code(crate = "crate")]
#[derive(BitcoinCode, PartialEq, Debug, Clone)]
pub struct TxInput {
    pub prev_out: TxOutpoint,
    pub script: Script,
    pub sequence: u32,

    #[bitcoin_code(skip)]
    pub lock_script: Option<Script>,

    #[bitcoin_code(skip)]
    pub value: Option<u64>,

    #[bitcoin_code(skip)]
    pub is_p2sh: Option<bool>,
}

#[bitcoin_code(crate = "crate")]
#[derive(BitcoinCode, PartialEq, Debug, Clone)]
pub struct TxOutput {
    pub value: u64,
    pub script: Script,
}

#[bitcoin_code(crate = "crate")]
#[derive(BitcoinCode, PartialEq, Debug, Clone)]
pub struct UnhashedTx {
    pub version: i32,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub lock_time: u32,
}

#[bitcoin_code(crate = "crate")]
#[derive(BitcoinCode, PartialEq, Debug, Clone)]
pub struct Tx {
    #[bitcoin_code(skip)]
    unhashed_tx: UnhashedTx,

    #[bitcoin_code(skip)]
    hash: Sha256d,

    raw: ByteArray,
}

impl UnhashedTx {
    pub fn preimages(&self, sig_hash_flags: &[SigHashFlags]) -> Vec<Vec<TxPreimage>> {
        TxPreimage::build_preimages(&SigTxPreimage {
            tx: self,
            sig_hash_flags,
        })
    }

    pub fn hashed(self) -> Tx {
        let raw = self.ser();
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

    pub fn unhashed_tx(&self) -> &UnhashedTx {
        &self.unhashed_tx
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
