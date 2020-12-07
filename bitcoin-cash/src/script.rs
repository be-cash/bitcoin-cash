use crate::error::{self, ScriptSerializeError};
use crate::{encoding_utils::encode_int, BitcoinCode, ByteArray, Op, Opcode, Ops, TaggedOp};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::borrow::Cow;
use std::io::Read;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
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

    pub fn from_ops(ops: impl IntoIterator<Item = Op>) -> Self {
        Script {
            ops: ops
                .into_iter()
                .map(TaggedOp::from_op)
                .collect::<Vec<_>>()
                .into(),
        }
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
) -> error::Result<PushPrefixTail> {
    use Opcode::*;
    match bytes.len() {
        0 if is_minimal_push => {
            vec.push(OP_0 as u8);
            return Ok(PushPrefixTail::NoTail);
        }
        0 if !is_minimal_push => {
            vec.push(OP_PUSHDATA1 as u8);
            vec.push(0);
        }
        1 if is_minimal_push => {
            let value = bytes[0];
            if value > 0 && value <= 16 {
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
            vec.write_u16::<LittleEndian>(len as u16)?;
        }
        len @ 0x10000..=0xffff_ffff => {
            vec.push(OP_PUSHDATA4 as u8);
            vec.write_u32::<LittleEndian>(len as u32)?;
        }
        _ => return ScriptSerializeError::PushTooLarge.into_err(),
    }
    Ok(PushPrefixTail::PushedData)
}

fn serialize_push_bytes(bytes: ByteArray, is_minimal_push: bool) -> error::Result<ByteArray> {
    use PushPrefixTail::*;
    let mut vec = Vec::new();
    match serialize_push_prefix(&mut vec, &bytes, is_minimal_push)? {
        NoTail => Ok(vec.into()),
        PushedData => Ok(ByteArray::new_unnamed(vec).concat(bytes)),
    }
}

pub fn serialize_op(op: &Op) -> error::Result<ByteArray> {
    use Opcode::*;
    Ok(match *op {
        Op::Code(opcode) => {
            let name: &str = opcode.into();
            ByteArray::new(name, vec![opcode as u8])
        }
        Op::Invalid(opcode) => ByteArray::new("Invalid Opcode", vec![opcode as u8]),
        Op::PushBoolean(boolean) => ByteArray::new(
            if boolean { "OP_TRUE" } else { "OP_FALSE" },
            vec![if boolean { OP_1 as u8 } else { OP_0 as u8 }],
        ),
        Op::PushInteger(int) => {
            let int = int.value();
            match int {
                -1 => serialize_op(&Op::Code(OP_1NEGATE))?,
                0 => serialize_op(&Op::Code(OP_0))?,
                1..=16 => serialize_op(&Op::Code(
                    num::FromPrimitive::from_u8(OP_1 as u8 + int as u8 - 1).unwrap(),
                ))?,
                -0x8000_0000 => unreachable!("Invalid integer constructed"),
                _ => serialize_push_bytes(encode_int(int).into(), true)?,
            }
        }
        Op::PushByteArray {
            ref array,
            is_minimal,
        } => serialize_push_bytes(array.clone(), is_minimal)?,
    })
}

pub fn serialize_ops<'a>(ops: impl IntoIterator<Item = &'a Op>) -> error::Result<ByteArray> {
    Ok(ByteArray::from_parts(
        ops.into_iter()
            .map(serialize_op)
            .collect::<Result<Vec<_>, _>>()?,
    ))
}

pub fn deserialize_ops(bytes: &[u8]) -> error::Result<Vec<Op>> {
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
                let op = match num::FromPrimitive::from_u8(opcode) {
                    Some(opcode) => Op::Code(opcode),
                    None => Op::Invalid(opcode),
                };
                ops.push(op);
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

pub fn deserialize_ops_byte_array(byte_array: ByteArray) -> error::Result<Vec<Op>> {
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

impl Script {
    pub fn ser_ops(&self) -> ByteArray {
        serialize_ops(self.ops.iter().map(|op| &op.op)).expect("Serialize failed")
    }
}

impl BitcoinCode for Script {
    fn ser(&self) -> ByteArray {
        self.ser_ops().ser()
    }

    fn deser(data: ByteArray) -> error::Result<(Self, ByteArray)> {
        let (script_code, rest) = ByteArray::deser(data)?;
        let ops = deserialize_ops_byte_array(script_code)?;
        Ok((Self::from_ops(ops), rest))
    }
}

#[cfg(test)]
mod tests {}
