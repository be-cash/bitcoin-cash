use crate::error::{BitcoinCodeError, Error, Result};
use crate::{encoding_utils::encode_var_int, ByteArray};
use lazy_static::lazy_static;
use std::sync::Mutex;

pub trait SerializeExt {
    fn ser(&self) -> ByteArray;
}

struct Serializer;

struct StaticCompound<'a> {
    ser: &'a mut Serializer,
    parts: Vec<ByteArray>,
    name: Option<&'static str>,
}

struct SeqCompound<'a> {
    ser: &'a mut Serializer,
    parts: Vec<ByteArray>,
    bytes: Vec<u8>,
    expected_len: Option<usize>,
}

struct StructCompound<'a> {
    ser: &'a mut Serializer,
    fields: Vec<ByteArray>,
    name: &'static str,
}

struct NotImplementedCompound;

lazy_static! {
    pub static ref NEXT_BYTE_ARRAY: Mutex<Option<ByteArray>> = Mutex::new(None);
}

enum Data {
    BA(ByteArray),
    Byte(u8),
}

impl From<ByteArray> for Data {
    fn from(byte_array: ByteArray) -> Self {
        Data::BA(byte_array)
    }
}

impl From<Data> for ByteArray {
    fn from(data: Data) -> Self {
        match data {
            Data::BA(data) => data,
            Data::Byte(byte) => vec![byte].into(),
        }
    }
}

impl serde::ser::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Error::Msg(msg.to_string())
    }
}

use Data::BA;

impl<'a> serde::ser::Serializer for &'a mut Serializer {
    type Ok = Data;
    type Error = Error;
    type SerializeSeq = SeqCompound<'a>;
    type SerializeTuple = StaticCompound<'a>;
    type SerializeTupleStruct = StaticCompound<'a>;
    type SerializeTupleVariant = NotImplementedCompound;
    type SerializeMap = NotImplementedCompound;
    type SerializeStruct = StructCompound<'a>;
    type SerializeStructVariant = NotImplementedCompound;

    fn serialize_unit(self) -> Result<Data> {
        Ok(BA(vec![].into()))
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Data> {
        Ok(ByteArray::new(name, vec![]).into())
    }

    fn serialize_bool(self, v: bool) -> Result<Data> {
        Ok(BA(vec![if v { 1 } else { 0 }].into()))
    }

    fn serialize_u8(self, v: u8) -> Result<Data> {
        Ok(Data::Byte(v))
    }

    fn serialize_u16(self, v: u16) -> Result<Data> {
        Ok(BA(v.to_le_bytes().into()))
    }

    fn serialize_u32(self, v: u32) -> Result<Data> {
        Ok(BA(v.to_le_bytes().into()))
    }

    fn serialize_u64(self, v: u64) -> Result<Data> {
        Ok(BA(v.to_le_bytes().into()))
    }

    fn serialize_i8(self, v: i8) -> Result<Data> {
        Ok(BA(v.to_le_bytes().into()))
    }

    fn serialize_i16(self, v: i16) -> Result<Data> {
        Ok(BA(v.to_le_bytes().into()))
    }

    fn serialize_i32(self, v: i32) -> Result<Data> {
        Ok(BA(v.to_le_bytes().into()))
    }

    fn serialize_i64(self, v: i64) -> Result<Data> {
        Ok(BA(v.to_le_bytes().into()))
    }

    fn serialize_u128(self, _v: u128) -> Result<Data> {
        BitcoinCodeError::DataTypeNotSupported("u128").into_err()
    }

    fn serialize_i128(self, _v: i128) -> Result<Data> {
        BitcoinCodeError::DataTypeNotSupported("i128").into_err()
    }

    fn serialize_f32(self, _v: f32) -> Result<Data> {
        BitcoinCodeError::DataTypeNotSupported("f32").into_err()
    }

    fn serialize_f64(self, _v: f64) -> Result<Data> {
        BitcoinCodeError::DataTypeNotSupported("f64").into_err()
    }

    fn serialize_str(self, v: &str) -> Result<Data> {
        let bytes = v.as_bytes();
        Ok(BA(ByteArray::new(
            "size",
            encode_var_int(bytes.len() as u64),
        )
        .concat(bytes)))
    }

    fn serialize_char(self, _c: char) -> Result<Data> {
        BitcoinCodeError::DataTypeNotSupported("char").into_err()
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Data> {
        Ok(BA(
            ByteArray::new("size", encode_var_int(v.len() as u64)).concat(v)
        ))
    }

    fn serialize_none(self) -> Result<Data> {
        BitcoinCodeError::MethodNotSupported("serialize_none").into_err()
    }

    fn serialize_some<T: ?Sized>(self, _v: &T) -> Result<Data>
    where
        T: serde::Serialize,
    {
        BitcoinCodeError::MethodNotSupported("serialize_some").into_err()
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(SeqCompound {
            ser: self,
            parts: Vec::new(),
            bytes: Vec::new(),
            expected_len: len,
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        Ok(StaticCompound {
            ser: self,
            parts: Vec::with_capacity(len),
            name: None,
        })
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Ok(StaticCompound {
            ser: self,
            parts: Vec::with_capacity(len),
            name: Some(name),
        })
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        BitcoinCodeError::MethodNotSupported("serialize_tuple_variant").into_err()
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        BitcoinCodeError::MethodNotSupported("serialize_map").into_err()
    }

    fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        Ok(StructCompound {
            ser: self,
            fields: Vec::with_capacity(len),
            name,
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        BitcoinCodeError::MethodNotSupported("serialize_struct_variant").into_err()
    }

    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, value: &T) -> Result<Data>
    where
        T: serde::ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Data>
    where
        T: serde::ser::Serialize,
    {
        BitcoinCodeError::MethodNotSupported("serialize_newtype_variant").into_err()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Data> {
        BitcoinCodeError::MethodNotSupported("serialize_unit_variant").into_err()
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

impl<'a> serde::ser::SerializeSeq for SeqCompound<'a> {
    type Ok = Data;
    type Error = Error;

    #[inline]
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: serde::ser::Serialize,
    {
        match value.serialize(&mut *self.ser)? {
            Data::BA(byte_array) => {
                if self.parts.is_empty() {
                    if let Some(len) = self.expected_len {
                        self.parts = Vec::with_capacity(len);
                    }
                }
                self.parts.push(byte_array);
            }
            Data::Byte(byte) => {
                if self.bytes.is_empty() {
                    if let Some(len) = self.expected_len {
                        self.bytes = Vec::with_capacity(len);
                    }
                }
                self.bytes.push(byte);
            }
        }
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok> {
        if !self.bytes.is_empty() {
            assert!(self.parts.is_empty());
            let byte_array = ByteArray::new("size", encode_var_int(self.bytes.len() as u64));
            return Ok(BA(byte_array.concat(self.bytes)));
        }
        assert!(self.bytes.is_empty());
        let mut byte_array = ByteArray::new("size", encode_var_int(self.parts.len() as u64));
        for element in self.parts {
            byte_array = byte_array.concat(element);
        }
        Ok(BA(byte_array))
    }
}

impl<'a> serde::ser::SerializeTuple for StaticCompound<'a> {
    type Ok = Data;
    type Error = Error;

    #[inline]
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: serde::ser::Serialize,
    {
        self.parts.push(value.serialize(&mut *self.ser)?.into());
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok> {
        let mut byte_array = ByteArray::from_parts(self.parts);
        if let Some(name) = self.name {
            byte_array = byte_array.named(name);
        }
        Ok(BA(byte_array))
    }
}

impl<'a> serde::ser::SerializeTupleStruct for StaticCompound<'a> {
    type Ok = Data;
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: serde::ser::Serialize,
    {
        self.parts.push(value.serialize(&mut *self.ser)?.into());
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok> {
        let mut byte_array = ByteArray::from_parts(self.parts);
        if let Some(name) = self.name {
            byte_array = byte_array.named(name);
        }
        Ok(BA(byte_array))
    }
}

impl serde::ser::SerializeTupleVariant for NotImplementedCompound {
    type Ok = Data;
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, _value: &T) -> Result<()>
    where
        T: serde::ser::Serialize,
    {
        unreachable!("SerializeTupleVariant not implemented")
    }

    #[inline]
    fn end(self) -> Result<Self::Ok> {
        unreachable!("SerializeTupleVariant not implemented")
    }
}

impl serde::ser::SerializeMap for NotImplementedCompound {
    type Ok = Data;
    type Error = Error;

    #[inline]
    fn serialize_key<K: ?Sized>(&mut self, _value: &K) -> Result<()>
    where
        K: serde::ser::Serialize,
    {
        unreachable!("SerializeMap not implemented")
    }

    #[inline]
    fn serialize_value<V: ?Sized>(&mut self, _value: &V) -> Result<()>
    where
        V: serde::ser::Serialize,
    {
        unreachable!("SerializeMap not implemented")
    }

    #[inline]
    fn end(self) -> Result<Self::Ok> {
        unreachable!("SerializeMap not implemented")
    }
}

impl<'a> serde::ser::SerializeStruct for StructCompound<'a> {
    type Ok = Data;
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: serde::ser::Serialize,
    {
        let byte_array: ByteArray = value.serialize(&mut *self.ser)?.into();
        let byte_array = match NEXT_BYTE_ARRAY.lock().unwrap().take() {
            Some(byte_array) => byte_array,
            None => byte_array.named(key),
        };
        self.fields.push(byte_array);
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok> {
        Ok(BA(ByteArray::from_parts(self.fields).named(self.name)))
    }
}

impl<'a> serde::ser::SerializeStructVariant for NotImplementedCompound {
    type Ok = Data;
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, _value: &T) -> Result<()>
    where
        T: serde::ser::Serialize,
    {
        unreachable!("SerializeStructVariant not implemented")
    }

    #[inline]
    fn end(self) -> Result<Self::Ok> {
        unreachable!("SerializeStructVariant not implemented")
    }
}

impl<T: serde::ser::Serialize> SerializeExt for T {
    fn ser(&self) -> ByteArray {
        self.serialize(&mut Serializer)
            .expect("Unsupported serialization.")
            .into()
    }
}

pub fn encode_bitcoin_code_all<'a, T: 'a>(
    values: impl IntoIterator<Item = &'a T>,
) -> Result<ByteArray>
where
    T: serde::ser::Serialize,
{
    Ok(ByteArray::from_parts(
        values.into_iter().map(|value| value.ser()),
    ))
}
