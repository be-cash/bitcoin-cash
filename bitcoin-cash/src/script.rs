use crate::error::{Result, ScriptSerializeError};
use crate::ops::{encoding_utils::encode_int, Op, OpcodeType, Ops};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use serde::{de, ser, Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Cow;
use std::io::Read;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Script<'a> {
    ops: Cow<'a, [Op]>,
    is_minimal_push: bool,
}

impl Ops for Script<'_> {
    fn ops(&self) -> Cow<[Op]> {
        self.ops.as_ref().into()
    }
}

impl<'a> Script<'a> {
    pub fn new(ops: Cow<'a, [Op]>, is_minimal_push: bool) -> Self {
        Script {
            ops,
            is_minimal_push,
        }
    }

    pub fn minimal(ops: Cow<'a, [Op]>) -> Self {
        Script {
            ops,
            is_minimal_push: true,
        }
    }

    pub fn serialize(&self) -> Result<Vec<u8>> {
        serialize_ops(&self.ops, self.is_minimal_push)
    }
}

impl<'a> Script<'a> {
    pub fn to_script_code(&self, n_codesep: Option<usize>) -> Script<'_> {
        let idx = if let Some(n_codesep) = n_codesep {
            let mut n_codeseps_found = 0;
            let mut codesep_idx = None;
            for (idx, op) in self.ops.iter().enumerate() {
                match op {
                    Op::Code(OpcodeType::OP_CODESEPARATOR) => {
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
        Script::new(self.ops[idx..].as_ref().into(), self.is_minimal_push)
    }

    pub fn to_script_code_first(&self) -> Script<'_> {
        if let Some(code_separator_idx) = self
            .ops
            .iter()
            .position(|op| op == &Op::Code(OpcodeType::OP_CODESEPARATOR))
        {
            Script::new(
                self.ops[code_separator_idx + 1..].as_ref().into(),
                self.is_minimal_push,
            )
        } else {
            self.clone()
        }
    }

    pub fn to_owned_script(&self) -> Script<'static> {
        Script {
            ops: self.ops.clone().into_owned().into(),
            is_minimal_push: self.is_minimal_push,
        }
    }
}

fn serialize_push_bytes(vec: &mut Vec<u8>, bytes: &[u8], is_minimal_push: bool) -> Result<()> {
    use OpcodeType::*;
    match bytes.len() {
        0 if is_minimal_push => {
            vec.push(Opcode::OP_0 as u8);
            return Ok(());
        }
        1 if is_minimal_push => {
            let value = bytes[0];
            if value <= 16 {
                vec.push(Opcode::OP_1 as u8 - 1 + value);
                return Ok(());
            }
            if value == 0x81 {
                vec.push(Opcode::OP_1NEGATE as u8);
                return Ok(());
            }
            vec.push(1);
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
    vec.extend(bytes);
    Ok(())
}

pub fn serialize_op(vec: &mut Vec<u8>, op: &Op, is_minimal_push: bool) -> Result<()> {
    use OpcodeType::*;
    match *op {
        Op::Code(opcode) => Ok(vec.push(opcode as u8)),
        Op::PushBoolean(boolean) => {
            vec.push(if boolean { OP_0 as u8 } else { OP_1 as u8 });
            Ok(())
        }
        Op::PushInteger(int) => {
            vec.push(match int {
                -1 => OP_1NEGATE as u8,
                0 => OP_0 as u8,
                1..=16 => OP_1 as u8 + int as u8 - 1,
                -0x8000_0000 => return ScriptSerializeError::InvalidInteger.into_err(),
                _ => return serialize_push_bytes(vec, &encode_int(int), false),
            });
            Ok(())
        }
        Op::PushByteArray(ref array) => serialize_push_bytes(vec, &array.data, is_minimal_push),
    }
}

pub fn serialize_ops(ops: &[Op], is_minimal_push: bool) -> Result<Vec<u8>> {
    let mut vec = Vec::new();
    for op in ops {
        serialize_op(&mut vec, op, is_minimal_push)?;
    }
    Ok(vec)
}

pub fn deserialize_ops(bytes: &[u8]) -> Result<Vec<Op>> {
    use OpcodeType::*;
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
        ops.push(Op::PushByteArray(vec.into()));
    }
    Ok(ops)
}

impl<'a> Serialize for Script<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        use ser::Error;
        serializer.serialize_bytes(
            &serialize_ops(&self.ops, self.is_minimal_push)
                .map_err(|err| S::Error::custom(err.to_string()))?,
        )
    }
}

struct ScriptVisitor;

impl<'de> de::Visitor<'de> for ScriptVisitor {
    type Value = Vec<Op>;
    fn expecting(
        &self,
        fmt: &mut std::fmt::Formatter<'_>,
    ) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "a byte array")
    }

    fn visit_bytes<E: de::Error>(self, v: &[u8]) -> std::result::Result<Self::Value, E> {
        deserialize_ops(v).map_err(|err| E::custom(err.to_string()))
    }
}

impl<'de, 'a> Deserialize<'de> for Script<'a> {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Script::new(
            deserializer.deserialize_bytes(ScriptVisitor)?.into(),
            true, // TODO: add options
        ))
    }
}

#[cfg(test)]
mod tests {}
