use crate::deserialize::BitcoinCodeError;
use crate::encoding_utils::write_var_int;
use crate::error::{Error, ErrorKind, Result};
use byteorder::{LittleEndian, WriteBytesExt};
use std::io;

struct Serializer<W> {
    writer: W,
}

struct Compound<'a, W: 'a> {
    ser: &'a mut Serializer<W>,
}

impl serde::ser::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        ErrorKind::Msg(msg.to_string()).into()
    }
}

impl<'a, W: io::Write> serde::ser::Serializer for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Compound<'a, W>;
    type SerializeTuple = Compound<'a, W>;
    type SerializeTupleStruct = Compound<'a, W>;
    type SerializeTupleVariant = Compound<'a, W>;
    type SerializeMap = Compound<'a, W>;
    type SerializeStruct = Compound<'a, W>;
    type SerializeStructVariant = Compound<'a, W>;

    fn serialize_unit(self) -> Result<()> {
        Ok(())
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<()> {
        Ok(())
    }

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.writer
            .write_u8(if v { 1 } else { 0 })
            .map_err(Into::into)
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.writer.write_u8(v).map_err(Into::into)
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.writer.write_u16::<LittleEndian>(v).map_err(Into::into)
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.writer.write_u32::<LittleEndian>(v).map_err(Into::into)
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.writer.write_u64::<LittleEndian>(v).map_err(Into::into)
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.writer.write_i8(v).map_err(Into::into)
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.writer.write_i16::<LittleEndian>(v).map_err(Into::into)
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.writer.write_i32::<LittleEndian>(v).map_err(Into::into)
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.writer.write_i64::<LittleEndian>(v).map_err(Into::into)
    }

    fn serialize_u128(self, _v: u128) -> Result<()> {
        BitcoinCodeError::DataTypeNotSupported("u128").into_err()
    }

    fn serialize_i128(self, _v: i128) -> Result<()> {
        BitcoinCodeError::DataTypeNotSupported("i128").into_err()
    }

    fn serialize_f32(self, _v: f32) -> Result<()> {
        BitcoinCodeError::DataTypeNotSupported("f32").into_err()
    }

    fn serialize_f64(self, _v: f64) -> Result<()> {
        BitcoinCodeError::DataTypeNotSupported("f64").into_err()
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        write_var_int(&mut self.writer, v.len() as u64)?;
        self.writer.write_all(v.as_bytes())?;
        Ok(())
    }

    fn serialize_char(self, _c: char) -> Result<()> {
        BitcoinCodeError::DataTypeNotSupported("char").into_err()
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        write_var_int(&mut self.writer, v.len() as u64)?;
        self.writer.write_all(v)?;
        Ok(())
    }

    fn serialize_none(self) -> Result<()> {
        BitcoinCodeError::MethodNotSupported("serialize_none").into_err()
    }

    fn serialize_some<T: ?Sized>(self, _v: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        BitcoinCodeError::MethodNotSupported("serialize_some").into_err()
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        let len = match len {
            Some(len) => len,
            None => return BitcoinCodeError::SequenceMustHaveLength.into_err(),
        };
        write_var_int(&mut self.writer, len as u64)?;
        Ok(Compound { ser: self })
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(Compound { ser: self })
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Ok(Compound { ser: self })
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

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        let len = match len {
            Some(len) => len,
            None => return BitcoinCodeError::SequenceMustHaveLength.into_err(),
        };
        write_var_int(&mut self.writer, len as u64)?;
        Ok(Compound { ser: self })
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(Compound { ser: self })
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

    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, value: &T) -> Result<()>
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
    ) -> Result<()>
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
    ) -> Result<()> {
        BitcoinCodeError::MethodNotSupported("serialize_unit_variant").into_err()
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

impl<'a, W: io::Write> serde::ser::SerializeSeq for Compound<'a, W> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: serde::ser::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W: io::Write> serde::ser::SerializeTuple for Compound<'a, W> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: serde::ser::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W: io::Write> serde::ser::SerializeTupleStruct for Compound<'a, W> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: serde::ser::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W: io::Write> serde::ser::SerializeTupleVariant for Compound<'a, W> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: serde::ser::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W: io::Write> serde::ser::SerializeMap for Compound<'a, W> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_key<K: ?Sized>(&mut self, value: &K) -> Result<()>
    where
        K: serde::ser::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn serialize_value<V: ?Sized>(&mut self, value: &V) -> Result<()>
    where
        V: serde::ser::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W: io::Write> serde::ser::SerializeStruct for Compound<'a, W> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, value: &T) -> Result<()>
    where
        T: serde::ser::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W: io::Write> serde::ser::SerializeStructVariant for Compound<'a, W> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, value: &T) -> Result<()>
    where
        T: serde::ser::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

pub fn encode_bitcoin_code<T>(value: &T) -> Result<Vec<u8>>
where
    T: serde::ser::Serialize,
{
    let mut vec = Vec::new();
    let mut serializer = Serializer { writer: &mut vec };
    value.serialize(&mut serializer)?;
    Ok(vec)
}
