use crate::error::{Result, ScriptSerializeError};
use crate::{encoding_utils::encode_int, ByteArray, Op, Opcode, Ops, TaggedOp};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use serde::{de, ser, Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Cow;
use std::io::Read;
use std::sync::Arc;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Script {
    ops: Arc<[TaggedOp]>,
}

impl Ops for Script {
    fn ops(&self) -> Cow<[TaggedOp]> {
        self.ops.as_ref().into()
    }
}

impl Script {
    pub fn new(ops: impl Into<Arc<[TaggedOp]>>) -> Self {
        Script { ops: ops.into() }
    }

    pub fn serialize(&self) -> Result<ByteArray> {
        serialize_ops(self.ops.iter().map(|op| &op.op))
    }
}

impl Script {
    pub fn ops_arc(&self) -> &Arc<[TaggedOp]> {
        &self.ops
    }

    pub fn to_script_code(&self, n_codesep: Option<usize>) -> Script {
        let idx = if let Some(n_codesep) = n_codesep {
            let mut n_codeseps_found = 0;
            let mut codesep_idx = None;
            for (idx, op) in self.ops.iter().enumerate() {
                match op.op {
                    Op::Code(Opcode::OP_CODESEPARATOR) => {
                        if n_codesep == n_codeseps_found {
                            codesep_idx = Some(idx);
                            break;
                        }
                        n_codeseps_found += 1;
                    }
                    _ => continue,
                }
            }
            codesep_idx.expect("Couldn't find OP_CODESEPARATOR")
        } else {
            0
        };
        Script::new(self.ops[idx..].to_vec())
    }

    pub fn to_script_code_first(&self) -> Script {
        if let Some(code_separator_idx) = self
            .ops
            .iter()
            .position(|op| op.op == Op::Code(Opcode::OP_CODESEPARATOR))
        {
            Script::new(self.ops[code_separator_idx + 1..].to_vec())
        } else {
            self.clone()
        }
    }
}

impl Default for Script {
    fn default() -> Self {
        Script { ops: Arc::new([]) }
    }
}

#[derive(Clone, Copy)]
enum PushPrefixTail {
    NoTail,
    PushedData,
}

fn serialize_push_prefix(
    vec: &mut Vec<u8>,
    bytes: &[u8],
    is_minimal_push: bool,
) -> Result<PushPrefixTail> {
    use Opcode::*;
    match bytes.len() {
        0 if is_minimal_push => {
            vec.push(OP_0 as u8);
            return Ok(PushPrefixTail::NoTail);
        }
        1 if is_minimal_push => {
            let value = bytes[0];
            if value <= 16 {
                vec.push(OP_1 as u8 - 1 + value);
                return Ok(PushPrefixTail::NoTail);
            } else if value == 0x81 {
                vec.push(OP_1NEGATE as u8);
                return Ok(PushPrefixTail::NoTail);
            } else {
                vec.push(1);
            }
        }
        len @ 0x00..=0x4b => vec.push(len as u8),
        len @ 0x4c..=0xff => {
            vec.push(OP_PUSHDATA1 as u8);
            vec.push(len as u8);
        }
        len @ 0x100..=0xffff => {
            vec.push(OP_PUSHDATA2 as u8);
            vec.write_u16::<LittleEndian>(len as u16).unwrap();
        }
        len @ 0x10000..=0xffff_ffff => {
            vec.push(OP_PUSHDATA4 as u8);
            vec.write_u32::<LittleEndian>(len as u32).unwrap();
        }
        _ => return ScriptSerializeError::PushTooLarge.into_err(),
    }
    Ok(PushPrefixTail::PushedData)
}

fn serialize_push_bytes(bytes: ByteArray, is_minimal_push: bool) -> Result<ByteArray> {
    use PushPrefixTail::*;
    let mut vec = Vec::new();
    match serialize_push_prefix(&mut vec, &bytes, is_minimal_push)? {
        NoTail => Ok(vec.into()),
        PushedData => Ok(ByteArray::new_unnamed(vec).concat(bytes)),
    }
}

pub fn serialize_op(op: &Op) -> Result<ByteArray> {
    use Opcode::*;
    match *op {
        Op::Code(opcode) => Ok([opcode as u8].into()),
        Op::Invalid(opcode) => Ok([opcode as u8].into()),
        Op::PushBoolean(boolean) => Ok([if boolean { OP_1 as u8 } else { OP_0 as u8 }].into()),
        Op::PushInteger(int) => Ok([match int {
            -1 => OP_1NEGATE as u8,
            0 => OP_0 as u8,
            1..=16 => OP_1 as u8 + int as u8 - 1,
            -0x8000_0000 => return ScriptSerializeError::InvalidInteger.into_err(),
            _ => {
                return serialize_push_bytes(encode_int(int).into(), false);
            }
        }]
        .into()),
        Op::PushByteArray {
            ref array,
            is_minimal,
        } => serialize_push_bytes(array.clone(), is_minimal),
    }
}

pub fn serialize_ops<'a>(ops: impl IntoIterator<Item = &'a Op>) -> Result<ByteArray> {
    let mut byte_array: ByteArray = [].into();
    for op in ops {
        byte_array = byte_array.concat(serialize_op(op)?);
    }
    Ok(byte_array)
}

pub fn deserialize_ops(bytes: &[u8]) -> Result<Vec<Op>> {
    use Opcode::*;
    let mut i = 0;
    let mut ops = Vec::new();
    let mut cur = std::io::Cursor::new(bytes);
    while i < bytes.len() {
        let byte = cur.read_u8()?;
        i += 1;
        let push_len = match byte {
            0 => {
                ops.push(Op::Code(OP_0));
                continue;
            }
            push_len @ 0x01..=0x4b => push_len as usize,
            byte if byte == OP_PUSHDATA1 as u8 => {
                i += 1;
                cur.read_u8()? as usize
            }
            byte if byte == OP_PUSHDATA2 as u8 => {
                i += 2;
                cur.read_u16::<LittleEndian>()? as usize
            }
            byte if byte == OP_PUSHDATA4 as u8 => {
                i += 4;
                cur.read_u32::<LittleEndian>()? as usize
            }
            opcode => {
                let opcode = num::FromPrimitive::from_u8(opcode)
                    .ok_or(())
                    .or_else(|()| ScriptSerializeError::UnknownOpcode.into_err())?;
                ops.push(Op::Code(opcode));
                continue;
            }
        };
        let mut vec = vec![0; push_len];
        cur.read_exact(&mut vec)?;
        i += push_len;
        let mut prefix = Vec::new();
        serialize_push_prefix(&mut prefix, &vec, true)?;
        let mut op: Op = ByteArray::new_unnamed(vec).into();
        if prefix[0] == byte {
            if let Op::PushByteArray { is_minimal, .. } = &mut op {
                *is_minimal = false;
            }
        }
        ops.push(op);
    }
    Ok(ops)
}

pub fn deserialize_ops_byte_array(byte_array: ByteArray) -> Result<Vec<Op>> {
    use std::convert::TryInto;
    use Opcode::*;
    let mut ops = Vec::new();
    let mut byte_array = Some(byte_array);
    while byte_array.as_ref().map_or(false, |array| array.len() > 0) {
        let (head, remainder) = byte_array.take().unwrap().split(1)?;
        let byte = head[0];
        let (push_len, remainder) = match byte {
            0 => {
                ops.push(Op::Code(OP_0));
                byte_array = Some(remainder);
                continue;
            }
            push_len @ 0x01..=0x4b => (push_len as usize, remainder),
            byte if byte == OP_PUSHDATA1 as u8 => {
                let (push_len, remainder) = remainder.split(1)?;
                (push_len[0] as usize, remainder)
            }
            byte if byte == OP_PUSHDATA2 as u8 => {
                let (push_len, remainder) = remainder.split(2)?;
                (
                    u16::from_le_bytes(push_len.as_ref().try_into().unwrap()) as usize,
                    remainder,
                )
            }
            byte if byte == OP_PUSHDATA4 as u8 => {
                let (push_len, remainder) = remainder.split(4)?;
                (
                    u32::from_le_bytes(push_len.as_ref().try_into().unwrap()) as usize,
                    remainder,
                )
            }
            opcode => {
                ops.push(
                    num::FromPrimitive::from_u8(opcode)
                        .map(Op::Code)
                        .unwrap_or(Op::Invalid(opcode)),
                );
                byte_array = Some(remainder);
                continue;
            }
        };
        let (pushed, remainder) = remainder.split(push_len)?;
        let mut prefix = Vec::new();
        serialize_push_prefix(&mut prefix, &pushed, true)?;
        let mut op: Op = pushed.into();
        if prefix[0] == byte {
            if let Op::PushByteArray { is_minimal, .. } = &mut op {
                *is_minimal = false;
            }
        }
        ops.push(op);
        byte_array = Some(remainder);
    }
    Ok(ops)
}

impl Serialize for Script {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        use ser::Error;
        serializer.serialize_bytes(
            &serialize_ops(self.ops.iter().map(|op| &op.op))
                .map_err(|err| S::Error::custom(err.to_string()))?,
        )
    }
}

struct ScriptVisitor;

impl<'de> de::Visitor<'de> for ScriptVisitor {
    type Value = Vec<TaggedOp>;
    fn expecting(
        &self,
        fmt: &mut std::fmt::Formatter<'_>,
    ) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "a byte array")
    }

    fn visit_bytes<E: de::Error>(self, v: &[u8]) -> std::result::Result<Self::Value, E> {
        let ops = deserialize_ops(v).map_err(|err| E::custom(err.to_string()))?;
        Ok(ops.into_iter().map(TaggedOp::from_op).collect())
    }
}

impl<'de, 'a> Deserialize<'de> for Script {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Script::new(deserializer.deserialize_bytes(ScriptVisitor)?))
    }
}

#[cfg(test)]
mod tests {}
