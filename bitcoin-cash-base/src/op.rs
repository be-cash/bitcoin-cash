use crate::{ByteArray, Integer, Opcode, encoding_utils::vec_to_int};

#[derive(Clone)]
pub enum Op {
    Code(Opcode),
    Invalid(u8),
    PushByteArray { array: ByteArray, is_minimal: bool },
    PushBoolean(bool),
    PushInteger(Integer),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PatternOp {
    Code(Opcode),
    Invalid(u8),
    Array(Vec<u8>),
    Bool(bool),
    Int(Integer),
}

impl std::fmt::Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Op::Code(code) => write!(f, "{:?}", code),
            Op::Invalid(code) => write!(f, "{:02x}", code),
            Op::PushByteArray { array, .. } => write!(f, "0x{}", hex::encode(&array)),
            Op::PushBoolean(boolean) => {
                write!(f, "{}", if *boolean { "OP_TRUE" } else { "OP_FALSE" })
            }
            Op::PushInteger(int) => write!(f, "{}", int),
        }
    }
}

impl std::fmt::Debug for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Op::Code(code) => write!(f, "Code({:?})", code),
            Op::Invalid(code) => write!(f, "Invalid(0x{:02x})", code),
            Op::PushByteArray { array, is_minimal } => {
                write!(f, "PushByteArray{{array: hex!({:?}).to_vec(), is_minimal: {}}})", hex::encode(&array), is_minimal)
            }
            Op::PushBoolean(boolean) => write!(f, "PushBoolean({:?})", boolean),
            Op::PushInteger(int) => write!(f, "PushInteger({})", int),
        }
    }
}

impl Op {
    pub fn from_array(array: impl Into<ByteArray>) -> Op {
        Op::PushByteArray {
            array: array.into(),
            is_minimal: true,
        }
    }

    pub fn from_int(int: impl std::convert::TryInto<Integer>) -> Op {
        Op::PushInteger(int.try_into().map_err(|_| "invalid integer").unwrap())
    }

    pub fn to_byte_array(&self) -> Option<&ByteArray> {
        match &self {
            Op::PushByteArray { array, ..} => Some(array),
            _ => None
        }
    }
    
    pub fn to_integer(&self) -> Option<Integer> {
        match *self {
            Op::PushInteger(integer) => Some(integer),
            Op::Code(opcode) => {
                if opcode == Opcode::OP_0 {
                    Some(0u8.into())
                } else if opcode == Opcode::OP_1NEGATE {
                    Some((-1i8).into())
                } else {
                    let opcode_num = opcode as u8;
                    if opcode_num >= Opcode::OP_1 as u8 && opcode_num <= Opcode::OP_16 as u8 {
                        Some((opcode_num + 1 - Opcode::OP_1 as u8).into())
                    } else {
                        None
                    }
                }
            },
            Op::PushByteArray { ref array, ..} => Integer::new(vec_to_int(&array[..]).ok()?).ok(),
            Op::PushBoolean(boolean) => Some((boolean as u8).into()),
            _ => None,
        }
    }
}

impl PartialEq for Op {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Op::Code(code1), Op::Code(code2)) => code1 == code2,
            (Op::Invalid(code1), Op::Invalid(code2)) => code1 == code2,
            (Op::PushByteArray { array: array1, .. }, Op::PushByteArray { array: array2, ..}) => array1 == array2,
            (Op::PushBoolean(b1), Op::PushBoolean(b2)) => b1 == b2,
            (Op::PushInteger(int1), Op::PushInteger(int2)) => int1 == int2,
            _ => false,
        }
    }
}

impl From<ByteArray> for Op {
    fn from(array: ByteArray) -> Self {
        Op::from_array(array)
    }
}

impl From<&Op> for PatternOp {
    fn from(op: &Op) -> Self {
        match *op {
            Op::Code(opcode) => PatternOp::Code(opcode),
            Op::Invalid(code) => PatternOp::Invalid(code),
            Op::PushByteArray { ref array, .. } => PatternOp::Array(array.to_vec()),
            Op::PushBoolean(boolean) => PatternOp::Bool(boolean),
            Op::PushInteger(int) => PatternOp::Int(int),
        }
    }
}
