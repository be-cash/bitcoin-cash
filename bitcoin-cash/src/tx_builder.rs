use crate::error::Result;
use crate::{
    error::Error, BitcoinCode, Ops, Script, SigHashFlags, TaggedOp, TaggedScript, TxInput,
    TxOutpoint, TxOutput, TxPreimage, UnhashedTx,
};
use std::any::Any;
use std::collections::HashMap;
use std::marker::PhantomData;
use serde::{Deserialize, Serialize};

pub const DEFAULT_FEE_PER_KB: u64 = 1000;
pub const DUST_AMOUNT: u64 = 546;

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UnsignedTxInput {
    pub prev_out: TxOutpoint,
    pub sequence: u32,
    pub value: u64,
}

struct TxBuilderInput<'b> {
    input: UnsignedTxInput,
    func_script: Box<
        dyn Fn(&[TxPreimage], Option<usize>, Option<Box<dyn Any>>, &Script, &[TxOutput]) -> Script
            + 'b
            + Sync
            + Send,
    >,
    sig_hash_flags: Vec<SigHashFlags>,
    lock_script: Option<Script>,
    is_p2sh: Option<bool>,
    signed_input: Option<TxInput>,
}

#[derive(PartialEq, Debug, Clone)]
enum TxBuilderOutput {
    KnownValue(TxOutput),
    Leftover {
        lower_bound: u64,
        upper_bound: u64,
        precedence: i32,
        script: Script,
    },
}

#[derive(Default)]
pub struct TxBuilder<'b> {
    version: i32,
    inputs: Vec<TxBuilderInput<'b>>,
    outputs: Vec<TxBuilderOutput>,
    lock_time: u32,
    fee_per_kb: u64,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct InputReference<T> {
    phantom: PhantomData<T>,
    input_idx: usize,
}

impl<T: Clone> Copy for InputReference<T> {}

#[derive(Default)]
pub struct UnsignedTx<'b> {
    builder: TxBuilder<'b>,
    outputs: Vec<TxOutput>,
    tx_preimages: Vec<Vec<TxPreimage>>,
    inputs: Vec<Option<TxInput>>,
    estimated_size: usize,
}

struct TxBuilderPreimages<'b> {
    builder: &'b TxBuilder<'b>,
    outputs: &'b [TxOutput],
}

pub trait ToPreimages {
    fn version(&self) -> i32;
    fn num_inputs(&self) -> usize;
    fn input_outpoint_at(&self, input_idx: usize) -> &TxOutpoint;
    fn input_sequence_at(&self, input_idx: usize) -> u32;
    fn input_sig_hash_flags_at(&self, input_idx: usize) -> &[SigHashFlags];
    fn input_value_at(&self, input_idx: usize) -> u64;
    fn input_lock_script_at(&self, input_idx: usize) -> Script;
    fn num_outputs(&self) -> usize;
    fn output_at(&self, output_idx: usize) -> &TxOutput;
    fn lock_time(&self) -> u32;
}

pub trait SignatoryKind {
    type SigHashFlags;
    type TxPreimages: ?Sized;

    fn sig_hash_flags_vec(sig_hash_flags: Self::SigHashFlags) -> Vec<SigHashFlags>;
    fn make_tx_preimages(tx_preimages: &[TxPreimage]) -> &Self::TxPreimages;
}

pub struct SignatoryKindOne;
pub struct SignatoryKindMultiple;

pub trait Signatory {
    type Script: Ops;
    type Signatures: 'static;
    type Kind: SignatoryKind;
    fn sig_hash_flags(&self) -> <Self::Kind as SignatoryKind>::SigHashFlags;
    fn placeholder_signatures(&self) -> Self::Signatures;
    fn build_script(
        &self,
        tx_preimages: &<Self::Kind as SignatoryKind>::TxPreimages,
        estimated_size: Option<usize>,
        sigs: Self::Signatures,
        lock_script: &Script,
        tx_outputs: &[TxOutput],
    ) -> Self::Script;
    fn is_p2sh(&self) -> bool {
        true
    }
}

impl<'b> TxBuilder<'b> {
    pub fn new_simple() -> Self {
        TxBuilder {
            version: 1,
            inputs: Vec::new(),
            outputs: Vec::new(),
            lock_time: 0,
            fee_per_kb: DEFAULT_FEE_PER_KB,
        }
    }

    pub fn new(version: i32, lock_time: u32) -> Self {
        TxBuilder {
            version,
            inputs: Vec::new(),
            outputs: Vec::new(),
            lock_time,
            fee_per_kb: DEFAULT_FEE_PER_KB,
        }
    }

    pub fn new_with_fee(version: i32, lock_time: u32, fee_per_kb: u64) -> Self {
        TxBuilder {
            version,
            inputs: Vec::new(),
            outputs: Vec::new(),
            lock_time,
            fee_per_kb,
        }
    }

    pub fn from_tx(tx: UnhashedTx) -> TxBuilder<'static> {
        TxBuilder {
            version: tx.version,
            inputs: tx.inputs.into_iter().map(|input| {
                let script = input.script.clone();
                let func = move |_tx_preimages: &[TxPreimage],
                        _estimated_size: Option<usize>,
                        _sigs: Option<Box<dyn Any>>,
                        _lock_script: &Script,
                        _tx_outputs: &[TxOutput]| {
                    script.clone()
                };
                TxBuilderInput {
                    input: UnsignedTxInput {
                        prev_out: input.prev_out.clone(),
                        sequence: input.sequence,
                        value: input.value.unwrap_or(0),
                    },
                    is_p2sh: input.is_p2sh,
                    lock_script: input.lock_script.clone(),
                    sig_hash_flags: input.clone().sig_hash_flags.unwrap_or(vec![]),
                    func_script: Box::new(func),
                    signed_input: Some(input),
                }
            }).collect(),
            outputs: tx.outputs.into_iter().map(TxBuilderOutput::KnownValue).collect(),
            lock_time: tx.lock_time,
            fee_per_kb: DEFAULT_FEE_PER_KB,
        }
    }

    pub fn set_fee_per_kb(&mut self, fee_per_kb: u64) {
        self.fee_per_kb = fee_per_kb;
    }

    pub fn add_input<S: Signatory + 'b + Sync + Send>(
        &mut self,
        input: impl Into<UnsignedTxInput>,
        lock_script: TaggedScript<S::Script>,
        input_signatory: S,
    ) -> InputReference<S> {
        let sig_hash_flags = input_signatory.sig_hash_flags();
        let sig_hash_flags = <S::Kind as SignatoryKind>::sig_hash_flags_vec(sig_hash_flags);
        let is_p2sh = input_signatory.is_p2sh();
        let func = move |tx_preimages: &[TxPreimage],
                         estimated_size: Option<usize>,
                         sigs: Option<Box<dyn Any>>,
                         lock_script: &Script,
                         tx_outputs: &[TxOutput]| {
            let sigs = match sigs {
                Some(sigs) => *sigs.downcast::<S::Signatures>().expect("Incompatible sigs"),
                None => input_signatory.placeholder_signatures(),
            };
            let tx_preimages = <S::Kind as SignatoryKind>::make_tx_preimages(tx_preimages);
            let mut ops: Vec<_> = input_signatory
                .build_script(tx_preimages, estimated_size, sigs, lock_script, tx_outputs)
                .ops()
                .into();
            if input_signatory.is_p2sh() {
                ops.push(TaggedOp::from_op(lock_script.ser_ops().into()));
            }
            Script::new(ops)
        };
        let input_idx = self.inputs.len();
        self.inputs.push(TxBuilderInput {
            input: input.into(),
            func_script: Box::new(func),
            sig_hash_flags,
            lock_script: Some(lock_script.into()),
            is_p2sh: Some(is_p2sh),
            signed_input: None,
        });
        InputReference {
            phantom: PhantomData,
            input_idx,
        }
    }

    pub fn add_output(&mut self, output: impl Into<TxOutput>) {
        self.outputs
            .push(TxBuilderOutput::KnownValue(output.into()));
    }

    pub fn add_outputs(&mut self, outputs: impl IntoIterator<Item = impl Into<TxOutput>>) {
        for output in outputs {
            self.add_output(output);
        }
    }

    pub fn add_leftover_output(&mut self, script: Script) {
        self.outputs.push(TxBuilderOutput::Leftover {
            lower_bound: DUST_AMOUNT,
            upper_bound: std::u64::MAX,
            script,
            precedence: 0,
        });
    }

    pub fn add_leftover_output_bounded(
        &mut self,
        lower_bound: u64,
        upper_bound: u64,
        precedence: i32,
        script: Script,
    ) {
        self.outputs.push(TxBuilderOutput::Leftover {
            lower_bound,
            upper_bound,
            script,
            precedence,
        });
    }

    pub fn version(&self) -> i32 {
        self.version
    }

    pub fn lock_time(&self) -> u32 {
        self.lock_time
    }

    fn estimate_size(&self, outputs: Vec<TxOutput>) -> usize {
        let mut inputs = Vec::with_capacity(self.inputs.len());
        for input in &self.inputs {
            let n_sigs = input.sig_hash_flags.len();
            let (preimages, lock_script) = match &input.lock_script {
                Some(lock_script) => {
                    let lock_script = Script::new(lock_script.ops().to_vec());
                    (vec![TxPreimage::empty_with_script(&lock_script); n_sigs], lock_script)
                }
                None => (vec![], Script::new(vec![])),
            };
            inputs.push(TxInput {
                prev_out: input.input.prev_out.clone(),
                script: (input.func_script)(&preimages, None, None, &lock_script, &outputs),
                sequence: input.input.sequence,
                lock_script: None,
                value: None,
                is_p2sh: None,
                sig_hash_flags: None,
            });
        }
        let tx = UnhashedTx {
            version: self.version,
            inputs,
            outputs,
            lock_time: self.lock_time,
        };
        tx.ser().len()
    }

    fn make_outputs(&self, leftover_amounts: &HashMap<usize, u64>) -> Vec<TxOutput> {
        let mut outputs = Vec::new();
        for (idx, output) in self.outputs.iter().enumerate() {
            match *output {
                TxBuilderOutput::KnownValue(ref output) => outputs.push(output.clone()),
                TxBuilderOutput::Leftover { ref script, .. } => outputs.push(TxOutput {
                    value: match leftover_amounts.get(&idx) {
                        Some(&value) => value,
                        None => continue,
                    },
                    script: script.clone(),
                }),
            }
        }
        outputs
    }

    pub fn build_without_estimation(self) -> Result<UnsignedTx<'b>> {
        let outputs = self.outputs.iter().map(|output| match output {
            TxBuilderOutput::KnownValue(output) => Ok(output.clone()),
            TxBuilderOutput::Leftover { .. } => Err(Error::Msg("Found a leftover output".to_string())),
        }).collect::<Result<Vec<_>>>()?;
        let estimated_size = self.estimate_size(outputs.clone());
        let mut inputs = vec![None; self.inputs.len()];
        for (idx, input) in self.inputs.iter().enumerate() {
            if let Some(signed_input) = &input.signed_input {
                if !signed_input.script.ops().is_empty() {
                    inputs[idx] = Some(signed_input.clone());
                }
            }
        }
        let tx_preimages = TxPreimage::build_preimages(&TxBuilderPreimages {
            builder: &self,
            outputs: &outputs,
        });
        return Ok(UnsignedTx::new(outputs, self, tx_preimages, estimated_size, inputs));
    }

    pub fn build(self) -> Result<UnsignedTx<'b>> {
        let known_output_amount = self.known_output_sum();
        let total_input_amount = self.input_sum();
        if known_output_amount > total_input_amount {
            return Err(Error::InsufficientInputAmount {
                amount: known_output_amount - total_input_amount,
            });
        }
        let mut total_leftover = total_input_amount - known_output_amount;
        let mut leftover_amounts = HashMap::new();
        let mut leftover_precedence = self
            .outputs
            .iter()
            .enumerate()
            .filter_map(|(idx, output)| match output {
                TxBuilderOutput::Leftover { precedence, .. } => Some((idx, precedence)),
                _ => None,
            })
            .collect::<Vec<_>>();
        leftover_precedence.sort_by(|(_, a), (_, b)| a.cmp(b));
        let mut estimated_size = None;
        for (idx, _) in leftover_precedence {
            if let TxBuilderOutput::Leftover {
                lower_bound,
                upper_bound,
                ..
            } = self.outputs[idx]
            {
                if total_leftover <= lower_bound {
                    continue;
                }
                let max_leftover = total_leftover.min(upper_bound);
                leftover_amounts.insert(idx, max_leftover);
                let new_size = self.estimate_size(self.make_outputs(&leftover_amounts));
                let fee = new_size as u64 * self.fee_per_kb / 1000;
                if fee <= total_leftover {
                    let leftover = (total_leftover - fee).min(upper_bound);
                    if leftover <= lower_bound {
                        leftover_amounts.remove(&idx);
                        continue;
                    }
                    leftover_amounts.insert(idx, leftover);
                    total_leftover -= leftover;
                    estimated_size = Some(new_size);
                } else {
                    leftover_amounts.remove(&idx);
                }
            }
        }
        let estimated_size = match estimated_size {
            Some(estimated_size) => estimated_size,
            None => self.estimate_size(self.make_outputs(&leftover_amounts)),
        };
        let outputs = self.make_outputs(&leftover_amounts);
        let tx_preimages = TxPreimage::build_preimages(&TxBuilderPreimages {
            builder: &self,
            outputs: &outputs,
        });
        let mut inputs = vec![None; self.inputs.len()];
        for (idx, input) in self.inputs.iter().enumerate() {
            if let Some(signed_input) = &input.signed_input {
                if !signed_input.script.ops().is_empty() {
                    inputs[idx] = Some(signed_input.clone());
                }
            }
        }
        Ok(UnsignedTx::new(outputs, self, tx_preimages, estimated_size, inputs))
    }

    pub fn known_output_sum(&self) -> u64 {
        self
            .outputs
            .iter()
            .map(|output| output.get_value())
            .sum::<u64>()
    }

    pub fn input_sum(&self) -> u64 {
        self
            .inputs
            .iter()
            .map(|input| input.input.value)
            .sum::<u64>()
    }
}

impl TxBuilderOutput {
    fn get_value(&self) -> u64 {
        match self {
            TxBuilderOutput::Leftover { .. } => 0,
            TxBuilderOutput::KnownValue(output) => output.value,
        }
    }
}

impl ToPreimages for TxBuilderPreimages<'_> {
    fn version(&self) -> i32 {
        self.builder.version()
    }
    fn num_inputs(&self) -> usize {
        self.builder.inputs.len()
    }
    fn input_outpoint_at(&self, input_idx: usize) -> &TxOutpoint {
        &self.builder.inputs[input_idx].input.prev_out
    }
    fn input_sequence_at(&self, input_idx: usize) -> u32 {
        self.builder.inputs[input_idx].input.sequence
    }
    fn input_sig_hash_flags_at(&self, input_idx: usize) -> &[SigHashFlags] {
        &self.builder.inputs[input_idx].sig_hash_flags
    }
    fn input_value_at(&self, input_idx: usize) -> u64 {
        self.builder.inputs[input_idx].input.value
    }
    fn input_lock_script_at(&self, input_idx: usize) -> Script {
        match &self.builder.inputs[input_idx].lock_script {
            Some(lock_script) => Script::new(lock_script.ops().to_vec()),
            None => Script::new(vec![]),
        }
    }

    fn num_outputs(&self) -> usize {
        self.outputs.len()
    }
    fn output_at(&self, output_idx: usize) -> &TxOutput {
        &self.outputs[output_idx]
    }
    fn lock_time(&self) -> u32 {
        self.builder.lock_time
    }
}

impl<'b> UnsignedTx<'b> {
    fn new(
        outputs: Vec<TxOutput>,
        builder: TxBuilder<'b>,
        tx_preimages: Vec<Vec<TxPreimage>>,
        estimated_size: usize,
        inputs: Vec<Option<TxInput>>,
    ) -> Self {
        UnsignedTx {
            inputs,
            builder,
            outputs,
            tx_preimages,
            estimated_size,
        }
    }

    pub fn sign_input<S: Signatory>(
        &mut self,
        input_ref: InputReference<S>,
        sigs: S::Signatures,
    ) -> Result<()> {
        println!("Signing input {}", input_ref.input_idx());
        self.sign_input_dyn(input_ref.input_idx, Box::new(sigs))
    }

    pub fn sign_input_dyn(&mut self, input_idx: usize, sigs: Box<dyn Any>) -> Result<()> {
        let input = &mut self.inputs[input_idx];
        if input.is_some() {
            return Err(Error::InputAlreadySigned { input_idx });
        }
        let builder_input = &self.builder.inputs[input_idx];
        let preimage = &self.tx_preimages[input_idx];
        *input = Some(TxInput {
            prev_out: builder_input.input.prev_out.clone(),
            script: (builder_input.func_script)(
                preimage,
                Some(self.estimated_size),
                Some(sigs),
                builder_input.lock_script.as_ref().unwrap_or(&Script::new(vec![])),
                &self.outputs,
            ),
            sequence: builder_input.input.sequence,
            lock_script: builder_input.lock_script.clone(),
            value: Some(builder_input.input.value),
            is_p2sh: builder_input.is_p2sh,
            sig_hash_flags: Some(builder_input.sig_hash_flags.clone()),
        });
        Ok(())
    }

    pub fn sign_input_signatory<S: Signatory>(
        &mut self,
        input_ref: InputReference<S>,
        signatory: S,
        sigs: S::Signatures,
    ) -> Result<()> {
        let input_idx = input_ref.input_idx;
        let input = &mut self.inputs[input_idx];
        if input.is_some() {
            return Err(Error::InputAlreadySigned { input_idx });
        }
        let builder_input = &self.builder.inputs[input_idx];
        let preimages = &self.tx_preimages[input_idx];
        let tx_preimages = <S::Kind as SignatoryKind>::make_tx_preimages(preimages);
        let empty_script = Script::new(vec![]);
        let lock_script = builder_input.lock_script.as_ref().unwrap_or(&empty_script);
        let mut ops: Vec<_> = signatory
            .build_script(
                tx_preimages,
                Some(self.estimated_size),
                sigs,
                lock_script,
                &self.outputs,
            )
            .ops()
            .into();
        if signatory.is_p2sh() {
            ops.push(TaggedOp::from_op(lock_script.ser_ops().into()));
        }
        *input = Some(TxInput {
            prev_out: builder_input.input.prev_out.clone(),
            script: Script::new(ops),
            sequence: builder_input.input.sequence,
            lock_script: builder_input.lock_script.clone(),
            value: Some(builder_input.input.value),
            is_p2sh: builder_input.is_p2sh,
            sig_hash_flags: Some(builder_input.sig_hash_flags.clone()),
        });
        Ok(())
    }

    pub fn input_preimages<S: Signatory>(
        &self,
        input_token: InputReference<S>,
    ) -> &<S::Kind as SignatoryKind>::TxPreimages {
        <S::Kind as SignatoryKind>::make_tx_preimages(&self.tx_preimages[input_token.input_idx])
    }

    pub fn preimages(&self) -> &[Vec<TxPreimage>] {
        &self.tx_preimages
    }

    pub fn complete_tx(self) -> UnhashedTx {
        let inputs = self
            .inputs
            .into_iter()
            .enumerate()
            .map(|(idx, input)| input.unwrap_or_else(|| panic!("Input {} not signed", idx)))
            .collect();
        UnhashedTx {
            version: self.builder.version,
            inputs,
            outputs: self.outputs,
            lock_time: self.builder.lock_time,
        }
    }

    pub fn partial_tx(self) -> UnhashedTx {
        let mut inputs = Vec::new();
        for (input, builder_input) in self.inputs.into_iter().zip(self.builder.inputs) {
            match input {
                Some(input) => inputs.push(input),
                None => inputs.push(TxInput {
                    prev_out: builder_input.input.prev_out,
                    script: Script::new(vec![]),
                    sequence: builder_input.input.sequence,
                    is_p2sh: builder_input.is_p2sh,
                    lock_script: builder_input.lock_script,
                    value: Some(builder_input.input.value),
                    sig_hash_flags: Some(builder_input.sig_hash_flags),
                })
            }
        }
        UnhashedTx {
            version: self.builder.version,
            inputs,
            outputs: self.outputs,
            lock_time: self.builder.lock_time,
        }
    }

    pub fn into_tx_builder(self) -> TxBuilder<'b> {
        self.builder
    }

    pub fn estimated_size(&self) -> usize {
        self.estimated_size
    }
}

impl<T> InputReference<T> {
    pub fn new(input_idx: usize) -> Self {
        InputReference {
            phantom: PhantomData,
            input_idx,
        }
    }

    pub fn input_idx(&self) -> usize {
        self.input_idx
    }
}

impl SignatoryKind for SignatoryKindOne {
    type SigHashFlags = SigHashFlags;

    type TxPreimages = TxPreimage;

    fn sig_hash_flags_vec(sig_hash_flags: Self::SigHashFlags) -> Vec<SigHashFlags> {
        vec![sig_hash_flags]
    }

    fn make_tx_preimages(tx_preimages: &[TxPreimage]) -> &Self::TxPreimages {
        &tx_preimages[0]
    }
}

impl SignatoryKind for SignatoryKindMultiple {
    type SigHashFlags = Vec<SigHashFlags>;

    type TxPreimages = [TxPreimage];

    fn sig_hash_flags_vec(sig_hash_flags: Self::SigHashFlags) -> Vec<SigHashFlags> {
        sig_hash_flags
    }

    fn make_tx_preimages(tx_preimages: &[TxPreimage]) -> &Self::TxPreimages {
        tx_preimages
    }
}
