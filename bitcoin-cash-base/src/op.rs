use crate::{ByteArray, Integer, Opcode};

#[derive(Clone)]
pub enum Op {
    Code(Opcode),
    Invalid(u8),
    PushByteArray { array: ByteArray, is_minimal: bool },
    PushBoolean(bool),
    PushInteger(Integer),
}

impl std::fmt::Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Op::Code(code) => write!(f, "{:?}", code),
            Op::Invalid(code) => write!(f, "{:02x}", code),
            Op::PushByteArray { array, .. } => write!(f, "0x{:?}", hex::encode(&array)),
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
