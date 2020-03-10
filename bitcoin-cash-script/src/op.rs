use crate::{ByteArray, Opcode};

#[derive(Clone, Eq, PartialEq)]
pub enum Op {
    Code(Opcode),
    PushByteArray(ByteArray<'static>),
    PushBoolean(bool),
    PushInteger(i32),
}

impl std::fmt::Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Op::Code(code) => write!(f, "{:?}", code),
            Op::PushByteArray(array) => write!(f, "0x{:?}", hex::encode(&array.data)),
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
            Op::PushByteArray(array) => write!(
                f,
                "PushByteArray(hex!({:?}).to_vec())",
                hex::encode(&array.data)
            ),
            Op::PushBoolean(boolean) => write!(f, "PushBoolean({:?})", boolean),
            Op::PushInteger(int) => write!(f, "PushInteger({})", int),
        }
    }
}
