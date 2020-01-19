use crate::{Op, ByteArray};

#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub enum DataType {
    Generic,
    Integer,
    Boolean,
    ByteArray(Option<usize>),
}

pub type Integer = i32;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StackItemData<'a> {
    Integer(Integer),
    Boolean(bool),
    ByteArray(ByteArray<'a>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BitcoinInteger(pub Integer);
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BitcoinBoolean(pub bool);
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BitcoinByteArray(pub Vec<u8>);

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
        BitcoinByteArray(self.to_vec())
    }
    fn to_pushop(&self) -> Op {
        Op::PushByteArray(self.to_vec().into())
    }
    fn to_data_type(&self) -> DataType {
        DataType::ByteArray(None)
    }
}
impl<'a> BitcoinDataType for ByteArray<'a> {
    type Type = BitcoinByteArray;
    fn to_data(&self) -> Self::Type {
        BitcoinByteArray(self.data.clone().into_owned())
    }
    fn to_pushop(&self) -> Op {
        Op::PushByteArray(self.to_owned_array())
    }
    fn to_data_type(&self) -> DataType {
        DataType::ByteArray(Some(self.data.len()))
    }
}

impl Into<StackItemData<'static>> for Op {
    fn into(self) -> StackItemData<'static> {
        match self {
            Op::Code(_) => unimplemented!(),
            Op::PushBoolean(boolean) => StackItemData::Boolean(boolean),
            Op::PushInteger(int) => StackItemData::Integer(int),
            Op::PushByteArray(array) => StackItemData::ByteArray(array.to_owned_array()),
        }
    }
}
