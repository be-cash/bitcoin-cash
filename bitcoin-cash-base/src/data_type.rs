use crate::{ByteArray, InnerInteger, Integer, IntegerResult, Op};

#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub enum DataType {
    Generic,
    Integer,
    Boolean,
    ByteArray(Option<usize>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum StackItemData {
    Integer(Integer),
    Boolean(bool),
    ByteArray(ByteArray),
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
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
impl BitcoinDataType for IntegerResult {
    type Type = BitcoinInteger;
    fn to_data(&self) -> Self::Type {
        BitcoinInteger(self.integer().expect("Invalid integer"))
    }
    fn to_pushop(&self) -> Op {
        Op::PushInteger(self.integer().expect("Invalid integer"))
    }
    fn to_data_type(&self) -> DataType {
        DataType::Integer
    }
}
impl BitcoinDataType for InnerInteger {
    type Type = BitcoinInteger;
    fn to_data(&self) -> Self::Type {
        BitcoinInteger(Integer::new(*self).expect("Invalid integer"))
    }
    fn to_pushop(&self) -> Op {
        Op::PushInteger(Integer::new(*self).expect("Invalid integer"))
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
        Op::from_array(self)
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
        Op::from_array(self.clone())
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
