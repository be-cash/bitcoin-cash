use crate::{
    encode_bitcoin_code, ops::Function, ops::Ops, serialize_ops, ByteArray, Hashed, Script,
    Sha256d, ToPreimages, TxOutpoint,
};
use bitflags::bitflags;
use serde_derive::{Deserialize, Serialize};

bitflags! {
    #[derive(Deserialize, Serialize)]
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

#[derive(Clone, Debug)]
pub struct TxPreimage<'a> {
    version: i32,
    hash_prevouts: ByteArray<'a>,
    hash_sequence: ByteArray<'a>,
    outpoint: TxOutpoint,
    script_code: ByteArray<'a>,
    value: u64,
    sequence: u32,
    hash_outputs: ByteArray<'a>,
    lock_time: u32,
    sighash_flags: SigHashFlags,
}

impl<'a> TxPreimage<'a> {
    pub fn build_preimages(tx: &impl ToPreimages) -> Vec<Vec<TxPreimage<'a>>> {
        let hash_all_prevouts = {
            let mut outpoints_serialized = ByteArray::from_slice(&[]);
            for input_idx in 0..tx.num_inputs() {
                outpoints_serialized = outpoints_serialized.concat(
                    encode_bitcoin_code(tx.input_outpoint_at(input_idx))
                        .expect("Cannot encode outpoint")
                        .into(),
                );
            }
            Sha256d::digest_byte_array(outpoints_serialized)
        };
        let hash_all_sequences = {
            let mut sequences_serialized = ByteArray::from_slice(&[]);
            for input_idx in 0..tx.num_inputs() {
                sequences_serialized = sequences_serialized.concat(
                    encode_bitcoin_code(&tx.input_sequence_at(input_idx))
                        .expect("Cannot encode sequence")
                        .into(),
                );
            }
            Sha256d::digest_byte_array(sequences_serialized)
        };
        let hash_all_outputs = {
            let mut outputs_serialized = ByteArray::from_slice(&[]);
            for output_idx in 0..tx.num_outputs() {
                let mut byte_array = ByteArray::new(
                    encode_bitcoin_code(tx.output_at(output_idx))
                        .expect("Cannot encode output")
                        .into(),
                );
                if let Some(redeem_script) = tx.output_redeem_script_at(output_idx) {
                    byte_array.preimage = Some(
                        vec![serialize_ops(&redeem_script.ops())
                            .expect("Cannot encode redeem script")
                            .into()]
                        .into(),
                    );
                    byte_array.function = Function::Hash160;
                }
                outputs_serialized = outputs_serialized.concat(byte_array);
            }
            Sha256d::digest_byte_array(outputs_serialized)
        };
        let mut inputs_preimages = Vec::with_capacity(tx.num_inputs());
        for input_idx in 0..tx.num_inputs() {
            let sig_hash_flags = tx.input_sig_hash_flags_at(input_idx);
            let mut preimages = Vec::with_capacity(sig_hash_flags.len());
            for &sighash_flags in sig_hash_flags {
                let hash_prevouts = if !sighash_flags.contains(SigHashFlags::ANYONECANPAY) {
                    hash_all_prevouts.clone()
                } else {
                    ByteArray::from_slice(&[0; 32])
                };
                let masked_flags = sighash_flags & SigHashFlags::MASK;
                let hash_sequence = if !sighash_flags.contains(SigHashFlags::ANYONECANPAY)
                    && masked_flags != SigHashFlags::SINGLE
                    && masked_flags != SigHashFlags::NONE
                {
                    hash_all_sequences.clone()
                } else {
                    ByteArray::from_slice(&[0; 32])
                };
                let hash_outputs =
                    if masked_flags != SigHashFlags::SINGLE && masked_flags != SigHashFlags::NONE {
                        hash_all_outputs.clone()
                    } else if masked_flags == SigHashFlags::SINGLE && input_idx < tx.num_outputs() {
                        Sha256d::digest_byte_array(
                            encode_bitcoin_code(tx.output_at(input_idx))
                                .expect("Cannot encode output")
                                .into(),
                        )
                    } else {
                        ByteArray::from_slice(&[0; 32])
                    };
                preimages.push(TxPreimage {
                    version: tx.version(),
                    hash_prevouts,
                    hash_sequence,
                    outpoint: tx.input_outpoint_at(input_idx).clone(),
                    script_code: encode_bitcoin_code(
                        &tx.input_lock_script_at(input_idx).to_script_code_first(),
                    )
                    .unwrap()
                    .into(),
                    value: tx.input_value_at(input_idx),
                    sequence: tx.input_sequence_at(input_idx),
                    hash_outputs,
                    lock_time: tx.lock_time(),
                    sighash_flags,
                });
            }
            inputs_preimages.push(preimages);
        }
        inputs_preimages
    }

    pub fn size_with_script(script_code: &Script) -> usize {
        struct TxPreimageWithoutScript {
            _version: i32,
            _hash_prevouts: Sha256d,
            _hash_sequence: Sha256d,
            _outpoint: TxOutpoint,
            _value: u64,
            _sequence: u32,
            _hash_outputs: Sha256d,
            _lock_time: u32,
            _sighash_flags: u32,
        }
        #[derive(Serialize)]
        struct TxPreimageOnlyScript<'a> {
            script_code: &'a Script<'a>,
        }
        let script_size = encode_bitcoin_code(&TxPreimageOnlyScript { script_code })
            .expect("Couldn't encode script")
            .len();
        let rest_size = std::mem::size_of::<TxPreimageWithoutScript>();
        script_size + rest_size
    }

    pub fn empty_with_script(script_code: &Script) -> TxPreimage<'static> {
        TxPreimage {
            version: 0,
            hash_prevouts: ByteArray::from_slice(&[0; 32]),
            hash_sequence: ByteArray::from_slice(&[0; 32]),
            outpoint: TxOutpoint {
                tx_hash: Sha256d::new([0; 32]),
                vout: 0,
            },
            script_code: encode_bitcoin_code(script_code).unwrap().into(),
            value: 0,
            sequence: 0,
            hash_outputs: ByteArray::from_slice(&[0; 32]),
            lock_time: 0,
            sighash_flags: SigHashFlags::ALL,
        }
    }

    pub fn to_owned_array(&self) -> ByteArray<'static> {
        ByteArray::new(self.version.to_le_bytes().to_vec().into())
            .concat(self.hash_prevouts.to_owned_array())
            .concat(self.hash_sequence.to_owned_array())
            .concat(encode_bitcoin_code(&self.outpoint).unwrap().into())
            .concat(self.script_code.to_owned_array())
            .concat(self.value.to_le_bytes().to_vec().into())
            .concat(self.sequence.to_le_bytes().to_vec().into())
            .concat(self.hash_outputs.to_owned_array())
            .concat(self.lock_time.to_le_bytes().to_vec().into())
            .concat(self.sighash_flags.bits().to_le_bytes().to_vec().into())
    }
}
