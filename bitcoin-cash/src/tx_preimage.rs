use crate::{
    BitcoinByteArray, BitcoinCode, BitcoinDataType, ByteArray, DataType, Hashed, Op, Script,
    Sha256d, ToPreimages, TxOutpoint,
};
use serde::{Deserialize, Serialize};
use bitflags::bitflags;

bitflags! {
    #[derive(Deserialize, Serialize, Default)]
    pub struct SigHashFlags: u32 {
        const ALL          = 0x01;
        const NONE         = 0x02;
        const SINGLE       = 0x03;
        const FORKID       = 0x40;
        const ANYONECANPAY = 0x80;
        const MASK         = 0x1f;
        const DEFAULT      = Self::ALL.bits | Self::FORKID.bits;
    }
}

#[bitcoin_code(crate = "crate")]
#[derive(Clone, Debug, Default, BitcoinCode)]
pub struct TxPreimage {
    pub version: i32,
    pub hash_prevouts: Sha256d,
    pub hash_sequence: Sha256d,
    pub outpoint: TxOutpoint,
    pub script_code: Script,
    pub value: u64,
    pub sequence: u32,
    pub hash_outputs: Sha256d,
    pub lock_time: u32,
    pub sig_hash_type: u32,
}

impl SigHashFlags {
    pub fn from_u8(flags: u8) -> Self {
        let mut sig_hash_flags = Self::DEFAULT;
        sig_hash_flags.bits = flags as u32;
        sig_hash_flags
    }
}

impl TxPreimage {
    pub fn build_preimages(tx: &impl ToPreimages) -> Vec<Vec<TxPreimage>> {
        let hash_all_prevouts = {
            let mut outpoints_serialized = ByteArray::from_slice_unnamed(&[]);
            for input_idx in 0..tx.num_inputs() {
                outpoints_serialized = outpoints_serialized.concat(
                    tx.input_outpoint_at(input_idx)
                        .ser()
                        .named(format!("outpoint_{}", input_idx)),
                );
            }
            Sha256d::digest(outpoints_serialized.named("prevouts")).named("hashPrevouts")
        };
        let hash_all_sequences = {
            let mut sequences_serialized = ByteArray::from_slice_unnamed(&[]);
            for input_idx in 0..tx.num_inputs() {
                sequences_serialized = sequences_serialized.concat(
                    tx.input_sequence_at(input_idx)
                        .ser()
                        .named(format!("sequence_{}", input_idx)),
                );
            }
            Sha256d::digest(sequences_serialized.named("sequences")).named("hashSequence")
        };

        let hash_all_outputs = {
            let mut outputs_serialized = ByteArray::from_slice_unnamed(&[]);
            for output_idx in 0..tx.num_outputs() {
                let byte_array = tx
                    .output_at(output_idx)
                    .ser()
                    .named(format!("output_{}", output_idx));
                outputs_serialized = outputs_serialized.concat(byte_array);
            }
            Sha256d::digest(outputs_serialized.named("outputs")).named("hashOutputs")
        };
        let mut inputs_preimages = Vec::with_capacity(tx.num_inputs());
        for input_idx in 0..tx.num_inputs() {
            let sig_hash_flags = tx.input_sig_hash_flags_at(input_idx);
            let mut preimages = Vec::with_capacity(sig_hash_flags.len());
            for &sig_hash_flags in sig_hash_flags {
                let hash_prevouts = if !sig_hash_flags.contains(SigHashFlags::ANYONECANPAY) {
                    hash_all_prevouts.clone()
                } else {
                    Sha256d::new([0; 32]).named("hashPrevouts")
                };
                let masked_flags = sig_hash_flags & SigHashFlags::MASK;
                let hash_sequence = if !sig_hash_flags.contains(SigHashFlags::ANYONECANPAY)
                    && masked_flags != SigHashFlags::SINGLE
                    && masked_flags != SigHashFlags::NONE
                {
                    hash_all_sequences.clone()
                } else {
                    Sha256d::new([0; 32]).named("hashSequence")
                };
                let hash_outputs =
                    if masked_flags != SigHashFlags::SINGLE && masked_flags != SigHashFlags::NONE {
                        hash_all_outputs.clone()
                    } else if masked_flags == SigHashFlags::SINGLE && input_idx < tx.num_outputs() {
                        Sha256d::digest(tx.output_at(input_idx).ser())
                    } else {
                        Sha256d::new([0; 32]).named("hashOutputs")
                    };
                preimages.push(TxPreimage {
                    version: tx.version(),
                    hash_prevouts,
                    hash_sequence,
                    outpoint: tx.input_outpoint_at(input_idx).clone(),
                    script_code: tx.input_lock_script_at(input_idx).to_script_code_first(),
                    value: tx.input_value_at(input_idx),
                    sequence: tx.input_sequence_at(input_idx),
                    hash_outputs,
                    lock_time: tx.lock_time(),
                    sig_hash_type: sig_hash_flags.bits(),
                });
            }
            inputs_preimages.push(preimages);
        }
        inputs_preimages
    }

    pub fn empty_with_script(script_code: &Script) -> TxPreimage {
        TxPreimage {
            version: 0,
            hash_prevouts: Sha256d::new([0; 32]).named("hashPrevouts"),
            hash_sequence: Sha256d::new([0; 32]).named("hashSequence"),
            outpoint: TxOutpoint {
                tx_hash: Sha256d::new([0; 32]),
                vout: 0,
            },
            script_code: script_code.to_script_code_first(),
            value: 0,
            sequence: 0,
            hash_outputs: Sha256d::new([0; 32]).named("hashOutputs"),
            lock_time: 0,
            sig_hash_type: SigHashFlags::ALL.bits(),
        }
    }
}

impl BitcoinDataType for TxPreimage {
    type Type = BitcoinByteArray;
    fn to_data(&self) -> Self::Type {
        BitcoinByteArray(self.ser())
    }
    fn to_pushop(&self) -> Op {
        self.ser().into()
    }
    fn to_data_type(&self) -> DataType {
        DataType::ByteArray(None)
    }
}
