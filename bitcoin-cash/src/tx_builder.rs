use crate::error::Result;
use crate::{
    encode_bitcoin_code, error::ErrorKind, Ops, Script, SigHashFlags, TaggedOp, TaggedScript,
    TxInput, TxOutpoint, TxOutput, TxPreimage, UnhashedTx,
};
use std::any::Any;
use std::collections::HashMap;
use std::marker::PhantomData;

#[derive(PartialEq, Debug, Clone)]
pub struct UnsignedTxInput {
    pub prev_out: TxOutpoint,
    pub sequence: u32,
    pub value: u64,
}

struct TxBuilderInput<'b> {
    input: UnsignedTxInput,
    func_script: Box<
        dyn Fn(&[TxPreimage], &TxBuilder, Option<Box<dyn Any>>, &Script, &[TxOutput]) -> Script
            + 'b
            + Sync
            + Send,
    >,
    sig_hash_flags: Vec<SigHashFlags>,
    lock_script: Script,
    is_p2sh: bool,
}

#[derive(PartialEq, Debug, Clone)]
enum TxBuilderOutput {
    KnownValue(TxOutput),
    Leftover {
        fee_per_kb: u64,
        lower_bound: u64,
        upper_bound: u64,
        precedence: i32,
        script: Script,
    },
}

pub struct TxBuilder<'b> {
    version: i32,
    inputs: Vec<TxBuilderInput<'b>>,
    outputs: Vec<TxBuilderOutput>,
    lock_time: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InputReference<T> {
    phantom: PhantomData<T>,
    input_idx: usize,
}

pub struct UnsignedTx<'b> {
    builder: TxBuilder<'b>,
    outputs: Vec<TxOutput>,
    tx_preimages: Vec<Vec<TxPreimage>>,
    inputs: Vec<Option<TxInput>>,
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

pub trait Signatory {
    type Script: Ops;
    type Signatures: 'static;
    fn sig_hash_flags(&self) -> Vec<SigHashFlags>;
    fn placeholder_signatures(&self) -> Self::Signatures;
    fn build_script(
        &self,
        tx_preimages: &[TxPreimage],
        unsigned_tx: &TxBuilder,
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
        }
    }

    pub fn new(version: i32, lock_time: u32) -> Self {
        TxBuilder {
            version,
            inputs: Vec::new(),
            outputs: Vec::new(),
            lock_time,
        }
    }

    pub fn add_input<S: Signatory + 'b + Sync + Send>(
        &mut self,
        input: impl Into<UnsignedTxInput>,
        lock_script: TaggedScript<S::Script>,
        input_script_builder: S,
    ) -> InputReference<S> {
        let sig_hash_flags = input_script_builder.sig_hash_flags();
        let is_p2sh = input_script_builder.is_p2sh();
        let func = move |tx_preimage: &[TxPreimage],
                         unsigned_tx: &TxBuilder,
                         sigs: Option<Box<dyn Any>>,
                         lock_script: &Script,
                         tx_outputs: &[TxOutput]| {
            let sigs = match sigs {
                Some(sigs) => *sigs.downcast::<S::Signatures>().expect("Incompatible sigs"),
                None => input_script_builder.placeholder_signatures(),
            };
            let mut ops: Vec<_> = input_script_builder
                .build_script(tx_preimage, unsigned_tx, sigs, lock_script, tx_outputs)
                .ops()
                .into();
            if input_script_builder.is_p2sh() {
                ops.push(TaggedOp::from_op(
                    lock_script
                        .serialize()
                        .expect("Cannot encode lock_script")
                        .into(),
                ));
            }
            Script::new(ops)
        };
        let input_idx = self.inputs.len();
        self.inputs.push(TxBuilderInput {
            input: input.into(),
            func_script: Box::new(func),
            sig_hash_flags,
            lock_script: lock_script.into(),
            is_p2sh,
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

    pub fn add_leftover_output(
        &mut self,
        fee_per_kb: u64,
        lower_bound: u64,
        upper_bound: u64,
        precedence: i32,
        script: Script,
    ) {
        self.outputs.push(TxBuilderOutput::Leftover {
            fee_per_kb,
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

    pub fn estimate_size(&self, outputs: Vec<TxOutput>) -> usize {
        let mut inputs = Vec::with_capacity(self.inputs.len());
        for input in &self.inputs {
            let n_sigs = input.sig_hash_flags.len();
            let lock_script = Script::new(input.lock_script.ops().to_vec());
            let preimages = vec![TxPreimage::empty_with_script(&lock_script); n_sigs];
            inputs.push(TxInput {
                prev_out: input.input.prev_out.clone(),
                script: (input.func_script)(&preimages, self, None, &lock_script, &outputs),
                sequence: input.input.sequence,
                lock_script: None,
                value: None,
                is_p2sh: None,
            });
        }
        let tx = UnhashedTx {
            version: self.version,
            inputs,
            outputs,
            lock_time: self.lock_time,
        };
        encode_bitcoin_code(&tx)
            .expect("Failed to encode tx for size estimation")
            .len()
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

    pub fn build(self) -> Result<UnsignedTx<'b>> {
        let known_output_amount = self
            .outputs
            .iter()
            .map(|output| output.get_value())
            .sum::<u64>();
        let total_input_amount = self
            .inputs
            .iter()
            .map(|input| input.input.value)
            .sum::<u64>();
        if known_output_amount > total_input_amount {
            return Err(ErrorKind::InsufficientInputAmount(
                known_output_amount - total_input_amount,
            )
            .into());
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
        for (idx, _) in leftover_precedence {
            if let TxBuilderOutput::Leftover {
                fee_per_kb,
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
                let new_size = self.estimate_size(self.make_outputs(&leftover_amounts)) as u64;
                let fee = new_size * fee_per_kb / 1000;
                if fee <= total_leftover {
                    let leftover = (total_leftover - fee).min(upper_bound);
                    if leftover <= lower_bound {
                        leftover_amounts.remove(&idx);
                        continue;
                    }
                    leftover_amounts.insert(idx, leftover);
                    total_leftover -= leftover;
                } else {
                    leftover_amounts.remove(&idx);
                }
            }
        }
        let outputs = self.make_outputs(&leftover_amounts);
        let tx_preimages = TxPreimage::build_preimages(&TxBuilderPreimages {
            builder: &self,
            outputs: &outputs,
        });
        Ok(UnsignedTx::new(outputs, self, tx_preimages))
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
        Script::new(self.builder.inputs[input_idx].lock_script.ops().to_vec())
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
    pub fn new(
        outputs: Vec<TxOutput>,
        builder: TxBuilder<'b>,
        tx_preimages: Vec<Vec<TxPreimage>>,
    ) -> Self {
        UnsignedTx {
            inputs: vec![None; builder.inputs.len()],
            builder,
            outputs,
            tx_preimages,
        }
    }

    pub fn sign_input_with<S: Signatory>(
        &mut self,
        input_token: InputReference<S>,
        make_sigs: impl FnOnce(&[TxPreimage]) -> S::Signatures,
    ) {
        let sigs = make_sigs(&self.tx_preimages[input_token.input_idx]);
        self.sign_input(input_token, sigs)
    }

    pub fn sign_input<S: Signatory>(
        &mut self,
        input_token: InputReference<S>,
        sigs: S::Signatures,
    ) {
        let input_ref = &mut self.inputs[input_token.input_idx];
        if input_ref.is_some() {
            panic!("Input already signed");
        }
        let builder_input = &self.builder.inputs[input_token.input_idx];
        let preimage = &self.tx_preimages[input_token.input_idx];
        *input_ref = Some(TxInput {
            prev_out: builder_input.input.prev_out.clone(),
            script: (builder_input.func_script)(
                preimage,
                &self.builder,
                Some(Box::new(sigs)),
                &Script::new(builder_input.lock_script.ops().to_vec()),
                &self.outputs,
            ),
            sequence: builder_input.input.sequence,
            lock_script: Some(builder_input.lock_script.clone()),
            value: Some(builder_input.input.value),
            is_p2sh: Some(builder_input.is_p2sh),
        });
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
