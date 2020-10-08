use crate::{
    ByteArray, Function, Hashed, Op, Opcode, Ops, Script, Sha256d, TaggedOp, TxInput, TxOutpoint,
    TxOutput, UnhashedTx,
};
use bimap::BiMap;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Default)]
struct JsonData {
    data_b64: Vec<String>,
    byte_arrays: Vec<JsonByteArray>,
    strings: Vec<Cow<'static, str>>,

    #[serde(skip)]
    string_indices: HashMap<Arc<Cow<'static, str>>, usize>,

    #[serde(skip)]
    data_indices: BiMap<Arc<[u8]>, usize>,
}

struct JsonByteArray {
    data_idx: usize,
    function: Function,
    name_idx: Option<usize>,
    preimage_indices: Option<Vec<usize>>,
}

type JsonByteArrayTuple = (usize, Function, Option<usize>, Option<Vec<usize>>);
type JsonByteArrayTupleRef<'a> = (
    &'a usize,
    &'a Function,
    &'a Option<usize>,
    &'a Option<Vec<usize>>,
);

#[derive(Serialize, Deserialize)]
enum JsonOp {
    Code(u8),
    Invalid(u8),
    PushByteArray { array_idx: usize, is_minimal: bool },
    PushBoolean(bool),
    PushInteger(i32),
}

#[derive(Serialize, Deserialize)]
struct JsonTaggedOp {
    op: JsonOp,
    src_file: usize,
    src_line: u32,
    src_column: u32,
    src_code: Vec<(u32, usize)>,
    pushed_names: Option<Vec<Option<usize>>>,
    alt_pushed_names: Option<Vec<Option<usize>>>,
}

#[derive(Serialize, Deserialize)]
struct JsonInput {
    prev_out_hash: usize,
    prev_out_vout: u32,
    script: Vec<JsonTaggedOp>,
    sequence: u32,
    lock_script: Option<Vec<JsonTaggedOp>>,
    value: Option<u64>,
    is_p2sh: Option<bool>,
}

#[derive(Deserialize, Serialize)]
struct JsonOutput {
    value: u64,
    script: Vec<JsonTaggedOp>,
}

#[derive(Serialize, Deserialize)]
struct JsonTx {
    data: JsonData,
    version: i32,
    inputs: Vec<JsonInput>,
    outputs: Vec<JsonOutput>,
    lock_time: u32,
}

pub mod error {
    use error_chain::error_chain;
    error_chain! {
        foreign_links {
            DecodeError(base64::DecodeError);
        }
        errors {
            InvalidDataIdx(idx: usize) {}
            InvalidStringIdx(idx: usize) {}
            InvalidHash {}
        }
    }
}

use error::ResultExt;

pub fn tx_to_json(tx: &UnhashedTx) -> Result<String, serde_json::Error> {
    let json_tx = JsonTx::from_tx(tx);
    serde_json::to_string(&json_tx)
}

pub fn json_to_tx(s: &str) -> Result<UnhashedTx, crate::error::Error> {
    let mut json_tx: JsonTx = serde_json::from_str(s)?;
    Ok(json_tx.make_tx()?)
}

impl JsonTx {
    fn from_tx(tx: &UnhashedTx) -> Self {
        let mut json_tx = JsonTx {
            data: JsonData::default(),
            version: tx.version,
            inputs: vec![],
            outputs: vec![],
            lock_time: tx.lock_time,
        };
        for input in tx.inputs.iter() {
            let json_input = JsonInput {
                prev_out_hash: json_tx
                    .data
                    .insert_byte_array(input.prev_out.tx_hash.as_byte_array()),
                prev_out_vout: input.prev_out.vout,
                script: json_tx.make_ops(input.script.ops().iter()),
                sequence: input.sequence,
                lock_script: input
                    .lock_script
                    .as_ref()
                    .map(|script| json_tx.make_ops(script.ops().iter())),
                value: input.value,
                is_p2sh: input.is_p2sh,
            };
            json_tx.inputs.push(json_input);
        }
        for output in tx.outputs.iter() {
            let json_output = JsonOutput {
                value: output.value,
                script: json_tx.make_ops(output.script.ops().iter()),
            };
            json_tx.outputs.push(json_output);
        }
        json_tx
    }

    fn make_tx(&mut self) -> Result<UnhashedTx, error::Error> {
        let mut tx = UnhashedTx {
            version: self.version,
            inputs: Vec::with_capacity(self.inputs.len()),
            outputs: Vec::with_capacity(self.outputs.len()),
            lock_time: self.lock_time,
        };
        let JsonTx {
            data,
            inputs,
            outputs,
            ..
        } = self;
        for input in inputs.iter() {
            let input = TxInput {
                prev_out: TxOutpoint {
                    tx_hash: Sha256d::from_byte_array(data.get_byte_array(input.prev_out_hash)?)
                        .chain_err(|| error::ErrorKind::InvalidHash)?,
                    vout: input.prev_out_vout,
                },
                sequence: input.sequence,
                script: Script::new(
                    input
                        .script
                        .iter()
                        .map(|op| op.to_tagged_op(data))
                        .collect::<Result<Vec<_>, _>>()?,
                ),
                lock_script: input
                    .lock_script
                    .as_ref()
                    .map(|lock_script| {
                        lock_script
                            .iter()
                            .map(|op| op.to_tagged_op(data))
                            .collect::<Result<Vec<_>, _>>()
                            .map(Script::new)
                    })
                    .map_or(Ok(None), |name| name.map(Some))?,
                value: input.value,
                is_p2sh: input.is_p2sh,
            };
            tx.inputs.push(input);
        }
        for output in outputs.iter() {
            let output = TxOutput {
                script: Script::new(
                    output
                        .script
                        .iter()
                        .map(|op| op.to_tagged_op(data))
                        .collect::<Result<Vec<_>, _>>()?,
                ),
                value: output.value,
            };
            tx.outputs.push(output);
        }
        Ok(tx)
    }

    fn make_ops<'a>(&mut self, ops: impl Iterator<Item = &'a TaggedOp>) -> Vec<JsonTaggedOp> {
        ops.map(|op| JsonTaggedOp::from_tagged_op(&mut self.data, op))
            .collect()
    }
}

impl JsonTaggedOp {
    fn from_tagged_op(data: &mut JsonData, tagged_op: &TaggedOp) -> Self {
        let op = match tagged_op.op {
            Op::Code(code) => JsonOp::Code(code as u8),
            Op::Invalid(code) => JsonOp::Invalid(code),
            Op::PushBoolean(boolean) => JsonOp::PushBoolean(boolean),
            Op::PushInteger(integer) => JsonOp::PushInteger(integer),
            Op::PushByteArray {
                ref array,
                is_minimal,
            } => JsonOp::PushByteArray {
                array_idx: data.insert_byte_array(array),
                is_minimal,
            },
        };
        JsonTaggedOp {
            op,
            src_file: data.insert_string(&tagged_op.src_file),
            src_line: tagged_op.src_line,
            src_column: tagged_op.src_column,
            src_code: tagged_op
                .src_code
                .iter()
                .map(|&(width, ref code)| (width, data.insert_string(code)))
                .collect(),
            pushed_names: tagged_op.pushed_names.as_ref().map(|pushed_names| {
                pushed_names
                    .iter()
                    .map(|name| name.as_ref().map(|n| data.insert_string(n)))
                    .collect()
            }),
            alt_pushed_names: tagged_op.alt_pushed_names.as_ref().map(|pushed_names| {
                pushed_names
                    .iter()
                    .map(|name| name.as_ref().map(|n| data.insert_string(n)))
                    .collect()
            }),
        }
    }

    fn to_tagged_op(&self, data: &mut JsonData) -> Result<TaggedOp, error::Error> {
        let op = match self.op {
            JsonOp::Code(code) => {
                let opcode: Option<Opcode> = num::FromPrimitive::from_u8(code);
                opcode.map(Op::Code).unwrap_or(Op::Invalid(code))
            }
            JsonOp::Invalid(code) => Op::Invalid(code),
            JsonOp::PushBoolean(boolean) => Op::PushBoolean(boolean),
            JsonOp::PushInteger(int) => Op::PushInteger(int),
            JsonOp::PushByteArray {
                array_idx,
                is_minimal,
            } => Op::PushByteArray {
                array: data.get_byte_array(array_idx)?,
                is_minimal,
            },
        };
        Ok(TaggedOp {
            op,
            src_file: data.get_string(self.src_file)?.clone(),
            src_line: self.src_line,
            src_column: self.src_column,
            src_code: self
                .src_code
                .iter()
                .map(|&(width, string_idx)| data.get_string(string_idx).map(|s| (width, s.clone())))
                .collect::<Result<Vec<_>, _>>()?,
            pushed_names: self
                .pushed_names
                .as_ref()
                .map(|pushed_names| {
                    pushed_names
                        .iter()
                        .map(|name| {
                            name.map(|string_idx| data.get_string(string_idx).map(Clone::clone))
                                .map_or(Ok(None), |name| name.map(Some))
                        })
                        .collect::<Result<Vec<_>, _>>()
                })
                .map_or(Ok(None), |name| name.map(Some))?,
            alt_pushed_names: self
                .alt_pushed_names
                .as_ref()
                .map(|pushed_names| {
                    pushed_names
                        .iter()
                        .map(|name| {
                            name.map(|string_idx| data.get_string(string_idx).map(Clone::clone))
                                .map_or(Ok(None), |name| name.map(Some))
                        })
                        .collect::<Result<Vec<_>, _>>()
                })
                .map_or(Ok(None), |name| name.map(Some))?,
        })
    }
}

impl JsonData {
    fn insert_byte_array(&mut self, byte_array: &ByteArray) -> usize {
        let preimage_indices = byte_array.preimage().map(|preimage| {
            preimage
                .iter()
                .map(|preimage| self.insert_byte_array(preimage))
                .collect::<Vec<_>>()
        });
        let name_idx = byte_array
            .name_arc()
            .map(|name| self.insert_string_arc(name));
        let data_idx = self.ensure_data(byte_array.data());
        let byte_array_idx = self.byte_arrays.len();
        self.byte_arrays.push(JsonByteArray {
            data_idx,
            function: byte_array.function(),
            name_idx,
            preimage_indices,
        });
        byte_array_idx
    }

    fn ensure_data(&mut self, data: &Arc<[u8]>) -> usize {
        if let Some(&data_idx) = self.data_indices.get_by_left(data) {
            data_idx
        } else {
            let new_idx = self.data_b64.len();
            self.data_b64.push(base64::encode(data));
            self.data_indices.insert(Arc::clone(data), new_idx);
            new_idx
        }
    }

    fn get_byte_array(&mut self, byte_array_idx: usize) -> Result<ByteArray, error::Error> {
        use error::ErrorKind::*;
        let preimage = self.byte_arrays[byte_array_idx]
            .preimage_indices
            .clone()
            .map(|preimage_indices| {
                preimage_indices
                    .iter()
                    .map(|&idx| self.get_byte_array(idx))
                    .collect::<Result<Vec<_>, _>>()
            })
            .map_or(Ok(None), |preimage| preimage.map(Some))?;
        let json = &self.byte_arrays[byte_array_idx];
        let data = if let Some(data) = self.data_indices.get_by_right(&json.data_idx) {
            Arc::clone(data)
        } else {
            let data_b64 = self
                .data_b64
                .get(json.data_idx)
                .ok_or(InvalidDataIdx(json.data_idx))?;
            let data = base64::decode(data_b64)?.into();
            self.data_indices.insert(Arc::clone(&data), json.data_idx);
            data
        };
        let name = json
            .name_idx
            .map(|name_idx| self.get_string(name_idx).map(Clone::clone).map(Arc::new))
            .map_or(Ok(None), |name| name.map(Some))?;
        Ok(ByteArray::from_preimage(
            data,
            name,
            json.function,
            preimage.map(Into::into),
        ))
    }

    #[allow(clippy::ptr_arg)]
    fn insert_string(&mut self, cow: &Cow<'static, str>) -> usize {
        if let Some(&string_idx) = self.string_indices.get(cow) {
            string_idx
        } else {
            let string_idx = self.strings.len();
            self.strings.push(cow.clone());
            self.string_indices
                .insert(Arc::new(cow.clone()), string_idx);
            string_idx
        }
    }

    fn insert_string_arc(&mut self, arc: &Arc<Cow<'static, str>>) -> usize {
        if let Some(&string_idx) = self.string_indices.get(arc) {
            string_idx
        } else {
            let string_idx = self.strings.len();
            self.strings.push((**arc).clone());
            self.string_indices.insert(Arc::clone(arc), string_idx);
            string_idx
        }
    }

    fn get_string(&self, string_idx: usize) -> Result<&Cow<'static, str>, error::Error> {
        self.strings
            .get(string_idx)
            .ok_or_else(|| error::ErrorKind::InvalidStringIdx(string_idx).into())
    }
}

impl JsonByteArray {
    fn from_tuple(t: JsonByteArrayTuple) -> Self {
        JsonByteArray {
            data_idx: t.0,
            function: t.1,
            name_idx: t.2,
            preimage_indices: t.3,
        }
    }

    fn as_tuple(&self) -> JsonByteArrayTupleRef {
        (
            &self.data_idx,
            &self.function,
            &self.name_idx,
            &self.preimage_indices,
        )
    }
}

impl<'de> serde::Deserialize<'de> for JsonByteArray {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(JsonByteArray::from_tuple(
            <JsonByteArrayTuple as serde::Deserialize<'de>>::deserialize(deserializer)?,
        ))
    }
}

impl serde::Serialize for JsonByteArray {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.as_tuple().serialize(serializer)
    }
}
