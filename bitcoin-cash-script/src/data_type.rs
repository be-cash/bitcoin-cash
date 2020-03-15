use crate::{ByteArray, Op};

#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub enum DataType {
    Generic,
    Integer,
    Boolean,
    ByteArray(Option<usize>),
}

pub type Integer = i32;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StackItemData {
    Integer(Integer),
    Boolean(bool),
    ByteArray(ByteArray),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BitcoinInteger(pub Integer);
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BitcoinBoolean(pub bool);
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BitcoinByteArray(pub ByteArray);

pub trait BitcoinDataType {
    type Type;
    fn to_data(&self) -> Self::Type;
    fn to_pushop(&self) -> Op;
    fn to_data_type(&self) -> DataType;
}

impl BitcoinDataType for Integer {
    type Type = BitcoinInteger;
    fn to_data(&self) -> Self::Type {
        BitcoinInteger(*self)
    }
    fn to_pushop(&self) -> Op {
        Op::PushInteger(*self)
    }
    fn to_data_type(&self) -> DataType {
        DataType::Integer
    }
}
impl BitcoinDataType for bool {
    type Type = BitcoinBoolean;
    fn to_data(&self) -> Self::Type {
        BitcoinBoolean(*self)
    }
    fn to_pushop(&self) -> Op {
        Op::PushBoolean(*self)
    }
    fn to_data_type(&self) -> DataType {
        DataType::Boolean
    }
}
impl BitcoinDataType for [u8] {
    type Type = BitcoinByteArray;
    fn to_data(&self) -> Self::Type {
        BitcoinByteArray(self.into())
    }
    fn to_pushop(&self) -> Op {
        Op::PushByteArray {
            array: self.to_vec().into(),
            is_minimal: true,
        }
    }
    fn to_data_type(&self) -> DataType {
        DataType::ByteArray(None)
    }
}
impl BitcoinDataType for ByteArray {
    type Type = BitcoinByteArray;
    fn to_data(&self) -> Self::Type {
        BitcoinByteArray(self.clone())
    }
    fn to_pushop(&self) -> Op {
        Op::PushByteArray {
            array: self.clone(),
            is_minimal: true,
        }
    }
    fn to_data_type(&self) -> DataType {
        DataType::ByteArray(Some(self.len()))
    }
}

impl From<Op> for StackItemData {
    fn from(op: Op) -> StackItemData {
        match op {
            Op::Code(_) => unimplemented!(),
            Op::Invalid(_) => unimplemented!(),
            Op::PushBoolean(boolean) => StackItemData::Boolean(boolean),
            Op::PushInteger(int) => StackItemData::Integer(int),
            Op::PushByteArray { array, .. } => StackItemData::ByteArray(array),
        }
    }
}
