use crate::error::Result;
use crate::{
    encode_bitcoin_code, error::ErrorKind, p2sh_script, Address, AddressType, ByteArray, Hash160,
    Hashed, InputScript, Op, Ops, Prefix, Script, SigHashFlags, TaggedScript, TaggedScriptOps,
    TxInput, TxOutpoint, TxOutput, TxPreimage, UnhashedTx, MAX_SIGNATURE_SIZE,
};
use std::collections::HashMap;

#[derive(PartialEq, Debug)]
pub struct EmptyTxInput {
    pub prev_out: TxOutpoint,
    pub sequence: u32,
    pub value: u64,
}

struct TxBuilderInput<'b> {
    input: EmptyTxInput,
    func_script: Box<
        dyn Fn(
                &[TxPreimage],
                &TxBuilder,
                Vec<ByteArray<'static>>,
                &Script,
                &[TxOutput],
            ) -> Script<'static>
            + 'b,
    >,
    sig_hash_flags: Vec<SigHashFlags>,
    lock_script: TaggedScriptOps,
}

#[derive(PartialEq, Debug, Clone)]
enum TxBuilderOutput<'b> {
    KnownValue(TxOutput<'b>),
    Leftover {
        fee_per_kb: u64,
        lower_bound: u64,
        upper_bound: u64,
        precedence: i32,
        script: Script<'b>,
    },
}

pub struct TxBuilder<'b> {
    version: i32,
    inputs: Vec<TxBuilderInput<'b>>,
    outputs: Vec<TxBuilderOutput<'b>>,
    output_redeem_scripts: Vec<Option<Script<'b>>>,
    lock_time: u32,
}

pub struct UnsignedTx<'b> {
    pub builder: TxBuilder<'b>,
    pub outputs: Vec<TxOutput<'b>>,
    pub output_redeem_scripts: Vec<Option<Script<'b>>>,
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
    fn output_redeem_script_at(&self, output_idx: usize) -> &Option<Script>;
    fn lock_time(&self) -> u32;
}

pub trait InputScriptBuilder {
    type Script: InputScript;
    fn sig_hash_flags(&self) -> Vec<SigHashFlags>;
    fn build_script<'a, 'b>(
        &self,
        tx_preimage: &[TxPreimage],
        unsigned_tx: &TxBuilder,
        sigs: Vec<ByteArray<'static>>,
        lock_script: &Script<'a>,
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
            output_redeem_scripts: Vec::new(),
            lock_time: 0,
        }
    }

    pub fn add_input<I: InputScript>(
        &mut self,
        input: impl Into<EmptyTxInput>,
        lock_script: TaggedScript<I>,
        input_script_builder: impl InputScriptBuilder<Script = I> + 'b,
    ) {
        let sig_hash_flags = input_script_builder.sig_hash_flags();
        let func = move |tx_preimage: &[TxPreimage],
                         unsigned_tx: &TxBuilder,
                         sigs: Vec<ByteArray<'static>>,
                         lock_script: &Script,
                         tx_outputs: &[TxOutput]| {
            let mut ops: Vec<_> = input_script_builder
                .build_script(tx_preimage, unsigned_tx, sigs, lock_script, tx_outputs)
                .ops()
                .into();
            if input_script_builder.is_p2sh() {
                ops.push(Op::PushByteArray(lock_script.serialize().unwrap().into()));
            }
            Script::minimal(ops.into())
        };
        self.inputs.push(TxBuilderInput {
            input: input.into(),
            func_script: Box::new(func),
            sig_hash_flags,
            lock_script: lock_script.into(),
        });
    }

    pub fn add_output(&mut self, output: impl Into<TxOutput<'b>>) {
        self.outputs
            .push(TxBuilderOutput::KnownValue(output.into()));
        self.output_redeem_scripts.push(None);
    }

    pub fn add_p2sh_output(&mut self, value: u64, redeem_script: Script<'b>) {
        let lock_script = Script::minimal(
            p2sh_script(&Address::from_hash(
                Prefix::default(),
                AddressType::P2SH,
                Hash160::digest(&redeem_script.serialize().unwrap()),
            ))
            .ops()
            .into_owned()
            .into(),
        );
        self.outputs.push(TxBuilderOutput::KnownValue(TxOutput {
            value,
            script: lock_script,
        }));
        self.output_redeem_scripts.push(Some(redeem_script));
    }

    pub fn add_outputs(&mut self, outputs: impl IntoIterator<Item = impl Into<TxOutput<'b>>>) {
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
        script: Script<'b>,
    ) {
        self.outputs.push(TxBuilderOutput::Leftover {
            fee_per_kb,
            lower_bound,
            upper_bound,
            script,
            precedence,
        });
        self.output_redeem_scripts.push(None);
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
            let lock_script = Script::minimal(input.lock_script.ops());
            let preimages = vec![TxPreimage::empty_with_script(&lock_script); n_sigs];
            let fake_sigs = vec![ByteArray::new_unnamed([0; MAX_SIGNATURE_SIZE].as_ref()); n_sigs];
            inputs.push(TxInput {
                prev_out: input.input.prev_out.clone(),
                script: (input.func_script)(&preimages, self, fake_sigs, &lock_script, &outputs),
                sequence: input.input.sequence,
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

    fn make_outputs(
        &self,
        leftover_amounts: &HashMap<usize, u64>,
    ) -> (Vec<TxOutput<'static>>, Vec<Option<Script<'static>>>) {
        let mut outputs = Vec::new();
        let mut output_redeem_scripts = Vec::new();
        for (idx, output) in self.outputs.iter().enumerate() {
            match output {
                TxBuilderOutput::KnownValue(output) => outputs.push(output.to_owned_output()),
                TxBuilderOutput::Leftover { script, .. } => outputs.push(TxOutput {
                    value: match leftover_amounts.get(&idx) {
                        Some(&value) => value,
                        None => continue,
                    },
                    script: script.to_owned_script(),
                }),
            }
            output_redeem_scripts.push(
                self.output_redeem_scripts[idx]
                    .as_ref()
                    .map(|script| script.to_owned_script()),
            );
        }
        (outputs, output_redeem_scripts)
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
        println!("total_leftover: {}", total_leftover);
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
                let new_size = self.estimate_size(self.make_outputs(&leftover_amounts).0) as u64;
                let fee = new_size * fee_per_kb / 1000;
                println!("fee for idx: {}", fee);
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
        let (outputs, output_redeem_scripts) = self.make_outputs(&leftover_amounts);
        Ok(UnsignedTx::new(outputs, output_redeem_scripts, self))
    }
}

impl<'b> TxBuilderOutput<'b> {
    fn get_value(&self) -> u64 {
        match self {
            TxBuilderOutput::Leftover { .. } => 0,
            TxBuilderOutput::KnownValue(output) => output.value,
        }
    }
}

impl ToPreimages for UnsignedTx<'_> {
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
        Script::minimal(self.builder.inputs[input_idx].lock_script.ops())
    }
    fn num_outputs(&self) -> usize {
        self.outputs.len()
    }
    fn output_at(&self, output_idx: usize) -> &TxOutput {
        &self.outputs[output_idx]
    }
    fn output_redeem_script_at(&self, output_idx: usize) -> &Option<Script> {
        &self.output_redeem_scripts[output_idx]
    }
    fn lock_time(&self) -> u32 {
        self.builder.lock_time
    }
}

impl<'b> UnsignedTx<'b> {
    pub fn new(
        outputs: Vec<TxOutput<'b>>,
        output_redeem_scripts: Vec<Option<Script<'b>>>,
        builder: TxBuilder<'b>,
    ) -> Self {
        UnsignedTx {
            builder,
            output_redeem_scripts,
            outputs,
        }
    }

    pub fn complete_tx(
        self,
        preimages: Vec<Vec<TxPreimage<'b>>>,
        sigs: Vec<Vec<ByteArray<'static>>>,
    ) -> UnhashedTx<'b> {
        let inputs = self
            .builder
            .inputs
            .iter()
            .zip(preimages)
            .zip(sigs)
            .map(|((input, preimage), sigs)| TxInput {
                prev_out: input.input.prev_out.clone(),
                script: (input.func_script)(
                    &preimage,
                    &self.builder,
                    sigs,
                    &Script::minimal(input.lock_script.ops()),
                    &self.outputs,
                ),
                sequence: input.input.sequence,
            })
            .collect();
        UnhashedTx {
            version: self.builder.version,
            inputs,
            outputs: self.outputs,
            lock_time: self.builder.lock_time,
        }
    }
}
