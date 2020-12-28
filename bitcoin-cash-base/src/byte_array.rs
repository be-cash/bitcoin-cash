use crate::{InnerInteger, Integer};
use byteorder::{LittleEndian, WriteBytesExt};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Function {
    Plain,
    Sha1,
    Ripemd160,
    Sha256,
    Hash256,
    Hash160,
    Num2Bin,
    EcdsaSign,
    SchnorrSign,
    ToDataSig,
    UnexpectedSplit,
    Reverse,
}

#[derive(Error, Debug)]
pub enum ByteArrayError {
    #[error("Index {split_idx} is out of bounds for array with length {len}")]
    InvalidSplit { split_idx: usize, len: usize },
    #[error("Invalid slice, expected {expected} but got {actual}.")]
    InvalidSlice { expected: usize, actual: usize },
    #[error("int={int} not valid for n_bytes={n_bytes}")]
    FromIntegerError { int: Integer, n_bytes: Integer },
    #[error("Leftover bytes: {bytes}")]
    LeftoverBytes { bytes: ByteArray },
}

#[derive(Clone, Debug)]
pub struct ByteArray {
    data: Arc<[u8]>,
    #[cfg(not(feature = "simple-bytearray"))]
    name: Option<Arc<Cow<'static, str>>>,
    #[cfg(not(feature = "simple-bytearray"))]
    function: Function,
    #[cfg(not(feature = "simple-bytearray"))]
    preimage: Option<Arc<[ByteArray]>>,
}

#[derive(Clone, Debug)]
pub struct FixedByteArray<T, H> {
    phantom: PhantomData<(T, H)>,
    byte_array: ByteArray,
}

pub trait HumanReadableHex {
    fn is_big_endian() -> bool;
}

#[derive(Clone, Debug)]
pub struct HumanReadableHexBigEndian;
#[derive(Clone, Debug)]
pub struct HumanReadableHexLittleEndian;

pub type FixedByteArrayBE<T> = FixedByteArray<T, HumanReadableHexBigEndian>;
pub type FixedByteArrayLE<T> = FixedByteArray<T, HumanReadableHexLittleEndian>;

impl HumanReadableHex for HumanReadableHexBigEndian {
    fn is_big_endian() -> bool { true }
}

impl HumanReadableHex for HumanReadableHexLittleEndian {
    fn is_big_endian() -> bool { false }
}

impl Function {
    pub fn should_keep_intact(self) -> bool {
        use Function::*;
        match self {
            Plain | Num2Bin | EcdsaSign | SchnorrSign | ToDataSig => false,
            _ => true,
        }
    }
}

impl ByteArray {
    pub fn debug_enabled() -> bool {
        true
    }

    #[cfg(not(feature = "simple-bytearray"))]
    pub fn new(name: impl Into<Cow<'static, str>>, data: impl Into<Arc<[u8]>>) -> Self {
        ByteArray {
            data: data.into(),
            name: Some(Arc::new(name.into())),
            function: Function::Plain,
            preimage: None,
        }
    }

    #[cfg(feature = "simple-bytearray")]
    pub fn new(_name: impl Into<Cow<'static, str>>, data: impl Into<Arc<[u8]>>) -> Self {
        ByteArray { data: data.into() }
    }

    #[cfg(not(feature = "simple-bytearray"))]
    pub fn new_unnamed(data: impl Into<Arc<[u8]>>) -> Self {
        ByteArray {
            data: data.into(),
            name: None,
            function: Function::Plain,
            preimage: None,
        }
    }

    #[cfg(feature = "simple-bytearray")]
    pub fn new_unnamed(data: impl Into<Arc<[u8]>>) -> Self {
        ByteArray { data: data.into() }
    }

    #[cfg(not(feature = "simple-bytearray"))]
    pub fn from_preimage(
        data: Arc<[u8]>,
        name: Option<Arc<Cow<'static, str>>>,
        function: Function,
        preimage: Option<Arc<[ByteArray]>>,
    ) -> Self {
        ByteArray {
            data,
            name,
            function,
            preimage,
        }
    }

    #[cfg(feature = "simple-bytearray")]
    pub fn from_preimage(
        data: Arc<[u8]>,
        _name: Option<Arc<Cow<'static, str>>>,
        _function: Function,
        _preimage: Option<Arc<[ByteArray]>>,
    ) -> Self {
        ByteArray { data }
    }

    #[cfg(not(feature = "simple-bytearray"))]
    pub fn from_slice(name: impl Into<Cow<'static, str>>, slice: &[u8]) -> Self {
        ByteArray {
            data: slice.into(),
            name: Some(Arc::new(name.into())),
            function: Function::Plain,
            preimage: None,
        }
    }

    #[cfg(feature = "simple-bytearray")]
    pub fn from_slice(_name: impl Into<Cow<'static, str>>, slice: &[u8]) -> Self {
        ByteArray { data: slice.into() }
    }

    #[cfg(not(feature = "simple-bytearray"))]
    pub fn from_slice_unnamed(slice: &[u8]) -> Self {
        ByteArray {
            data: slice.into(),
            name: None,
            function: Function::Plain,
            preimage: None,
        }
    }

    #[cfg(feature = "simple-bytearray")]
    pub fn from_slice_unnamed(slice: &[u8]) -> Self {
        ByteArray { data: slice.into() }
    }

    pub fn from_parts(parts: impl IntoIterator<Item = ByteArray>) -> ByteArray {
        let mut byte_array: Option<ByteArray> = None;
        for part in parts {
            byte_array = Some(match byte_array {
                Some(byte_array) => byte_array.concat(part),
                None => part,
            })
        }
        byte_array.unwrap_or_else(|| ByteArray::new_unnamed(Vec::new()))
    }

    #[cfg(not(feature = "simple-bytearray"))]
    pub fn function(&self) -> Function {
        self.function
    }

    #[cfg(feature = "simple-bytearray")]
    pub fn function(&self) -> Function {
        Function::Plain
    }

    #[cfg(not(feature = "simple-bytearray"))]
    pub fn preimage(&self) -> Option<&[ByteArray]> {
        self.preimage.as_ref().map(|preimage| preimage.as_ref())
    }

    #[cfg(feature = "simple-bytearray")]
    pub fn preimage(&self) -> Option<&[ByteArray]> {
        None
    }

    #[cfg(not(feature = "simple-bytearray"))]
    pub fn preimage_arc(&self) -> Option<&Arc<[ByteArray]>> {
        self.preimage.as_ref()
    }

    #[cfg(feature = "simple-bytearray")]
    pub fn preimage_arc(&self) -> Option<&Arc<[ByteArray]>> {
        None
    }

    #[cfg(not(feature = "simple-bytearray"))]
    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|name| (*(*name).as_ref()).as_ref())
    }

    #[cfg(feature = "simple-bytearray")]
    pub fn name(&self) -> Option<&str> {
        None
    }

    #[cfg(not(feature = "simple-bytearray"))]
    pub fn name_arc(&self) -> Option<&Arc<Cow<'static, str>>> {
        self.name.as_ref()
    }

    #[cfg(feature = "simple-bytearray")]
    pub fn name_arc(&self) -> Option<&Arc<Cow<'static, str>>> {
        None
    }

    pub fn data(&self) -> &Arc<[u8]> {
        &self.data
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    pub fn hex(&self) -> String {
        hex::encode(&self)
    }

    #[cfg(not(feature = "simple-bytearray"))]
    pub fn concat_named_option(
        self,
        other: impl Into<ByteArray>,
        name: Option<Arc<Cow<'static, str>>>,
    ) -> ByteArray {
        let other = other.into();
        let mut new_data = Vec::with_capacity(self.data.len() + other.data.len());
        new_data.extend_from_slice(&self.data);
        new_data.extend_from_slice(&other.data);
        if self.function == Function::Plain && other.function == Function::Plain {
            let mut new_preimage = if self.preimage.is_some() {
                self.preimage.unwrap().to_vec()
            } else {
                vec![self]
            };
            new_preimage.append(&mut if other.preimage.is_some() {
                other.preimage.unwrap().to_vec()
            } else {
                vec![other]
            });
            return ByteArray {
                data: new_data.into(),
                name,
                function: Function::Plain,
                preimage: Some(new_preimage.into()),
            };
        }
        ByteArray {
            data: new_data.into(),
            name,
            function: Function::Plain,
            preimage: Some(vec![self, other].into()),
        }
    }

    #[cfg(feature = "simple-bytearray")]
    pub fn concat_named_option(
        self,
        other: impl Into<ByteArray>,
        _name: Option<Arc<Cow<'static, str>>>,
    ) -> ByteArray {
        let other = other.into();
        let mut new_data = Vec::with_capacity(self.data.len() + other.data.len());
        new_data.extend_from_slice(&self.data);
        new_data.extend_from_slice(&other.data);
        ByteArray {
            data: new_data.into(),
        }
    }

    pub fn concat(self, other: impl Into<ByteArray>) -> ByteArray {
        self.concat_named_option(other, None)
    }

    pub fn concat_named(
        self,
        name: impl Into<Arc<Cow<'static, str>>>,
        other: impl Into<ByteArray>,
    ) -> ByteArray {
        self.concat_named_option(other, Some(name.into()))
    }

    #[cfg(not(feature = "simple-bytearray"))]
    pub fn split(self, at: usize) -> Result<(ByteArray, ByteArray), ByteArrayError> {
        if self.data.len() < at {
            return Err(ByteArrayError::InvalidSplit {
                split_idx: at,
                len: self.data.len(),
            });
        }
        let mut data = self.data.to_vec();
        let other = data.split_off(at);
        let mut function = Function::Plain;
        let (left_preimage, right_preimage) = match self.preimage {
            Some(preimage) if !self.function.should_keep_intact() => {
                let mut left_preimage = Vec::new();
                let mut right_preimage = Vec::new();
                let mut is_left = true;
                let mut len = 0;
                for part in preimage.iter().cloned() {
                    let part_len = part.data.len();
                    if len == at {
                        is_left = false;
                    }
                    if len + part_len > at && is_left {
                        let part_function = part.function;
                        let (mut sub_left, mut sub_right) = part.split(at - len)?;
                        if part_function.should_keep_intact() {
                            sub_left.function = Function::UnexpectedSplit;
                            sub_right.function = Function::UnexpectedSplit;
                        }
                        left_preimage.push(sub_left);
                        right_preimage.push(sub_right);
                        is_left = false;
                    } else if is_left {
                        left_preimage.push(part);
                    } else {
                        right_preimage.push(part);
                    }
                    len += part_len;
                }
                (Some(left_preimage), Some(right_preimage))
            }
            Some(_) => {
                function = Function::UnexpectedSplit;
                (None, None)
            }
            None => (None, None),
        };
        Ok((
            ByteArray {
                data: data.into(),
                name: None,
                function,
                preimage: left_preimage.map(Into::into),
            },
            ByteArray {
                data: other.into(),
                name: None,
                function,
                preimage: right_preimage.map(Into::into),
            },
        ))
    }

    #[cfg(feature = "simple-bytearray")]
    pub fn split(self, at: usize) -> Result<(ByteArray, ByteArray), ByteArrayError> {
        if self.data.len() < at {
            return Err(format!(
                "Index {} is out of bounds for array with length {}, {}.",
                at,
                self.data.len(),
                hex::encode(&self.data)
            ));
        }
        let mut data = self.data.to_vec();
        let other = data.split_off(at);
        Ok((
            ByteArray { data: data.into() },
            ByteArray { data: other.into() },
        ))
    }

    #[cfg(not(feature = "simple-bytearray"))]
    pub fn apply_function(self, data: impl Into<Arc<[u8]>>, function: Function) -> ByteArray {
        ByteArray {
            data: data.into(),
            name: None,
            function,
            preimage: Some(vec![self].into()),
        }
    }

    #[cfg(feature = "simple-bytearray")]
    pub fn apply_function(self, data: impl Into<Arc<[u8]>>, _function: Function) -> ByteArray {
        ByteArray { data: data.into() }
    }

    #[cfg(not(feature = "simple-bytearray"))]
    pub fn named(self, name: impl Into<Cow<'static, str>>) -> ByteArray {
        ByteArray {
            name: Some(Arc::new(name.into())),
            ..self
        }
    }

    #[cfg(feature = "simple-bytearray")]
    pub fn named(self, _name: impl Into<Cow<'static, str>>) -> ByteArray {
        ByteArray { ..self }
    }

    #[cfg(not(feature = "simple-bytearray"))]
    pub fn named_option(self, name: Option<Arc<Cow<'static, str>>>) -> ByteArray {
        ByteArray { name, ..self }
    }

    #[allow(clippy::comparison_chain)]
    pub fn from_int(int: Integer, n_bytes: Integer) -> Result<Self, ByteArrayError> {
        let inner_int = int.value();
        let inner_n_bytes = n_bytes.value();
        if inner_n_bytes <= 0 {
            return Err(ByteArrayError::FromIntegerError { int, n_bytes });
        }
        let max_bits = (inner_n_bytes * 8 - 1).min(31) as u128;
        let max_num: u128 = 1 << max_bits;
        let max_num = (max_num - 1) as InnerInteger;
        let min_num = -max_num;
        if inner_int < min_num || inner_int > max_num {
            return Err(ByteArrayError::FromIntegerError { int, n_bytes });
        }
        let mut bytes = Vec::new();
        bytes.write_i32::<LittleEndian>(inner_int.abs()).unwrap();
        let n_bytes = inner_n_bytes as usize;
        if bytes.len() < n_bytes {
            bytes.append(&mut vec![0; n_bytes - bytes.len()]);
        } else if bytes.len() > n_bytes {
            bytes.drain(n_bytes..);
        }
        if inner_int < 0 {
            let len = bytes.len();
            bytes[len - 1] |= 0x80;
        }
        Ok(ByteArray::from_preimage(
            bytes.into(),
            None,
            Function::Num2Bin,
            None,
        ))
    }
}

impl<T, H> FixedByteArray<T, H> {
    pub fn as_byte_array(&self) -> &ByteArray {
        &self.byte_array
    }

    pub fn into_byte_array(self) -> ByteArray {
        self.byte_array
    }

    pub fn named(self, name: impl Into<Cow<'static, str>>) -> FixedByteArray<T, H> {
        FixedByteArray {
            phantom: PhantomData,
            byte_array: self.byte_array.named(name),
        }
    }
}

impl<T, H> FixedByteArray<T, H>
where
    T: AsRef<[u8]>,
{
    pub fn new(name: impl Into<Cow<'static, str>>, data: T) -> Self {
        FixedByteArray {
            phantom: PhantomData,
            byte_array: ByteArray::from_preimage(
                data.as_ref().into(),
                Some(Arc::new(name.into())),
                Function::Plain,
                None,
            ),
        }
    }

    pub fn new_unnamed(data: T) -> Self {
        FixedByteArray {
            phantom: PhantomData,
            byte_array: ByteArray::from_preimage(data.as_ref().into(), None, Function::Plain, None),
        }
    }
}

impl<T, H> FixedByteArray<T, H>
where
    T: Default + AsRef<[u8]>,
{
    pub fn from_slice(
        name: impl Into<Cow<'static, str>>,
        slice: &[u8],
    ) -> Result<Self, ByteArrayError> {
        let array = T::default();
        if array.as_ref().len() != slice.len() {
            return Err(ByteArrayError::InvalidSlice {
                expected: array.as_ref().len(),
                actual: slice.len(),
            });
        }
        Ok(FixedByteArray {
            phantom: PhantomData,
            byte_array: ByteArray::from_preimage(
                slice.into(),
                Some(Arc::new(name.into())),
                Function::Plain,
                None,
            ),
        })
    }

    pub fn from_slice_unnamed(slice: &[u8]) -> Result<Self, ByteArrayError> {
        let array = T::default();
        if array.as_ref().len() != slice.len() {
            return Err(ByteArrayError::InvalidSlice {
                expected: array.as_ref().len(),
                actual: slice.len(),
            });
        }
        Ok(FixedByteArray {
            phantom: PhantomData,
            byte_array: ByteArray::from_preimage(slice.into(), None, Function::Plain, None),
        })
    }

    pub fn from_byte_array(byte_array: ByteArray) -> Result<Self, ByteArrayError> {
        let array = T::default();
        if array.as_ref().len() != byte_array.len() {
            return Err(ByteArrayError::InvalidSlice {
                expected: array.as_ref().len(),
                actual: byte_array.len(),
            });
        }
        Ok(FixedByteArray {
            phantom: PhantomData,
            byte_array,
        })
    }
}

impl Default for ByteArray {
    fn default() -> Self {
        ByteArray::from_slice_unnamed(&[])
    }
}

impl From<Vec<u8>> for ByteArray {
    fn from(vec: Vec<u8>) -> Self {
        ByteArray::new_unnamed(vec)
    }
}

impl From<&[u8]> for ByteArray {
    fn from(slice: &[u8]) -> Self {
        ByteArray::from_slice_unnamed(slice)
    }
}

macro_rules! array_impls {
    ($($N:literal)+) => {
        $(
            impl From<[u8; $N]> for ByteArray {
                fn from(array: [u8; $N]) -> Self {
                    ByteArray::from_slice_unnamed(array.as_ref())
                }
            }
            impl From<&[u8; $N]> for ByteArray {
                fn from(array: &[u8; $N]) -> Self {
                    ByteArray::from_slice_unnamed(array.as_ref())
                }
            }
        )+
    }
}

array_impls! {
     0  1  2  3  4  5  6  7  8  9
    10 11 12 13 14 15 16 17 18 19
    20 21 22 23 24 25 26 27 28 29
    30 31 32
}

impl Hash for ByteArray {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.data.hash(state)
    }
}

impl std::fmt::Display for ByteArray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self))
    }
}

impl Deref for ByteArray {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl AsRef<[u8]> for ByteArray {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

impl<I: std::slice::SliceIndex<[u8]>> std::ops::Index<I> for ByteArray {
    type Output = I::Output;
    fn index(&self, index: I) -> &Self::Output {
        &self.data[index]
    }
}

impl PartialEq for ByteArray {
    fn eq(&self, other: &ByteArray) -> bool {
        self.data == other.data
    }
}

impl Eq for ByteArray {}

impl<'de> serde::Deserialize<'de> for ByteArray {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(ByteArray::from_slice_unnamed(
            <&[u8] as serde::Deserialize<'de>>::deserialize(deserializer)?,
        ))
    }
}

impl serde::Serialize for ByteArray {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.data.serialize(serializer)
    }
}

impl<T: Default + AsRef<[u8]>, H> Default for FixedByteArray<T, H> {
    fn default() -> Self {
        FixedByteArray {
            phantom: PhantomData,
            byte_array: ByteArray::new_unnamed(T::default().as_ref()),
        }
    }
}

impl<T, H> Hash for FixedByteArray<T, H> {
    fn hash<HS: std::hash::Hasher>(&self, state: &mut HS) {
        self.byte_array.hash(state)
    }
}

impl<T, H> Deref for FixedByteArray<T, H> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.byte_array.data
    }
}

impl<T, H> AsRef<[u8]> for FixedByteArray<T, H> {
    fn as_ref(&self) -> &[u8] {
        &self.byte_array.data
    }
}

impl<T, H, I: std::slice::SliceIndex<[u8]>> std::ops::Index<I> for FixedByteArray<T, H> {
    type Output = I::Output;
    fn index(&self, index: I) -> &Self::Output {
        &self.byte_array.data[index]
    }
}

impl<T, H> PartialEq for FixedByteArray<T, H> {
    fn eq(&self, other: &FixedByteArray<T, H>) -> bool {
        self.byte_array.data == other.byte_array.data
    }
}

impl<T, H> Eq for FixedByteArray<T, H> {}

impl<'de, T, H> serde::Deserialize<'de> for FixedByteArray<T, H>
where
    T: serde::Deserialize<'de> + AsRef<[u8]> + AsMut<[u8]> + Default,
    H: HumanReadableHex
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let hex = String::deserialize(deserializer)?;
            let mut data = hex::decode(&hex).map_err(serde::de::Error::custom)?;
            if !H::is_big_endian() {
                data.reverse();
            }
            let mut array = T::default();
            array.as_mut().copy_from_slice(&data);
            Ok(FixedByteArray::new_unnamed(array))
        } else {
            Ok(FixedByteArray::new_unnamed(T::deserialize(deserializer)?))
        }
    }
}

impl<T, H> serde::Serialize for FixedByteArray<T, H>
where
    T: serde::Serialize + Default + AsMut<[u8]>,
    H: HumanReadableHex,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut array = T::default();
        array.as_mut().copy_from_slice(self.byte_array.as_ref());
        if serializer.is_human_readable() {
            if !H::is_big_endian() {
                array.as_mut().reverse();
            }
            let hex = hex::encode(array.as_mut());
            hex.serialize(serializer)
        } else {
            array.serialize(serializer)
        }
    }
}

#[cfg(not(feature = "simple-bytearray"))]
#[cfg(test)]
mod tests {
    use super::{ByteArray, Function};
    use sha2::Digest;

    #[test]
    fn test_cat() {
        let a = ByteArray::from_slice_unnamed(b"A");
        let b = ByteArray::from_slice_unnamed(b"B");
        let c = ByteArray::from_slice_unnamed(b"C");
        let d = ByteArray::from_slice_unnamed(b"D");
        let ab = a.concat(b);
        {
            assert_eq!(ab.data.as_ref(), b"AB");
            let preimage = ab.preimage.as_ref().expect("No preimage");
            assert_eq!(preimage[0].data.as_ref(), b"A");
            assert_eq!(preimage[0].preimage, None);
            assert_eq!(preimage[1].data.as_ref(), b"B");
            assert_eq!(preimage[1].preimage, None);
        }
        let abcd = ab.concat(c.concat(d));
        {
            assert_eq!(abcd.data.as_ref(), b"ABCD");
            let preimage = abcd.preimage.as_ref().expect("No preimage");
            assert_eq!(preimage.len(), 4);
            assert_eq!(preimage[0].data.as_ref(), b"A");
            assert_eq!(preimage[0].preimage, None);
            assert_eq!(preimage[1].data.as_ref(), b"B");
            assert_eq!(preimage[1].preimage, None);
            assert_eq!(preimage[2].data.as_ref(), b"C");
            assert_eq!(preimage[2].preimage, None);
            assert_eq!(preimage[3].data.as_ref(), b"D");
            assert_eq!(preimage[3].preimage, None);
        }
    }

    #[test]
    fn test_hash() {
        let a = ByteArray::from_slice_unnamed(b"A");
        let b = ByteArray::from_slice_unnamed(b"B");
        let c = ByteArray::from_slice_unnamed(b"C");
        let cat = a.concat(b).concat(c);
        let hash = sha2::Sha256::digest(&cat.data);
        let hashed = cat.apply_function(hash.as_ref(), Function::Sha256);
        let hash_preimage = hashed.preimage.as_ref().expect("No hash_preimage");
        assert_eq!(hashed.data.as_ref(), hash.as_slice());
        assert_eq!(hash_preimage.len(), 1);
        assert_eq!(hash_preimage[0].data.as_ref(), b"ABC");
        let preimage = hash_preimage[0].preimage.as_ref().expect("No preimage");
        assert_eq!(preimage[0].data.as_ref(), b"A");
        assert_eq!(preimage[0].preimage, None);
        assert_eq!(preimage[1].data.as_ref(), b"B");
        assert_eq!(preimage[1].preimage, None);
        assert_eq!(preimage[2].data.as_ref(), b"C");
        assert_eq!(preimage[2].preimage, None);
    }

    #[test]
    fn test_hash_nested() {
        let a = ByteArray::from_slice_unnamed(b"A");
        let b = ByteArray::from_slice_unnamed(b"B");
        let inner = a.concat(b);
        let inner_hash = sha2::Sha256::digest(&inner.data);
        let inner_hashed = inner.apply_function(inner_hash.as_ref(), Function::Sha256);
        let c = ByteArray::from_slice_unnamed(b"C");
        let d = ByteArray::from_slice_unnamed(b"D");
        let outer = c.concat(inner_hashed).concat(d);
        let outer_hash = sha2::Sha256::digest(&outer.data);
        let outer_hashed = outer.apply_function(outer_hash.as_ref(), Function::Sha256);
        assert_eq!(outer_hashed.data.as_ref(), outer_hash.as_slice());

        let outer_preimage = outer_hashed.preimage.as_ref().expect("No preimage");

        assert_eq!(outer_preimage.len(), 1);
        let outer_preimage0 = &outer_preimage[0];
        assert_eq!(
            outer_preimage0.data.as_ref(),
            [b"C", inner_hash.as_slice(), b"D"].concat().as_slice()
        );
        let outer_preimages = outer_preimage0.preimage.as_ref().expect("No preimage");
        assert_eq!(outer_preimages.len(), 3);
        assert_eq!(outer_preimages[0].preimage, None);
        assert_eq!(outer_preimages[1].data.as_ref(), inner_hash.as_slice());
        assert!(outer_preimages[1].preimage.is_some());
        assert_eq!(outer_preimages[2].data.as_ref(), b"D");
        assert_eq!(outer_preimages[2].preimage, None);

        let inner_hash_preimage = outer_preimages[1].preimage.as_ref().expect("No preimage");
        assert_eq!(inner_hash_preimage.len(), 1);
        assert_eq!(inner_hash_preimage[0].data.as_ref(), b"AB");
        let inner_preimage = inner_hash_preimage[0]
            .preimage
            .as_ref()
            .expect("No preimage");
        assert_eq!(inner_preimage[0].data.as_ref(), b"A");
        assert_eq!(inner_preimage[0].preimage, None);
        assert_eq!(inner_preimage[1].data.as_ref(), b"B");
        assert_eq!(inner_preimage[1].preimage, None);
    }

    #[test]
    fn test_split_a_b() {
        let a = ByteArray::from_slice_unnamed(b"A");
        let b = ByteArray::from_slice_unnamed(b"B");
        let cat = a.concat(b);
        let (left, right) = cat.split(1).unwrap();
        let left_preimage = left.preimage.as_ref().expect("No preimage");
        let right_preimage = right.preimage.as_ref().expect("No preimage");
        assert_eq!(left.function, Function::Plain);
        assert_eq!(left.data.as_ref(), b"A");
        assert_eq!(left_preimage.len(), 1);
        assert_eq!(left_preimage[0].data.as_ref(), b"A");
        assert_eq!(right.function, Function::Plain);
        assert_eq!(right.data.as_ref(), b"B");
        assert_eq!(right_preimage.len(), 1);
        assert_eq!(right_preimage[0].data.as_ref(), b"B");
    }

    #[test]
    fn test_split_nested() {
        let a = ByteArray::from_slice_unnamed(b"A");
        let b = ByteArray::from_slice_unnamed(b"B");
        let inner = a.concat(b);
        let inner_hash = sha2::Sha256::digest(&inner.data);
        let inner_hashed = inner.apply_function(inner_hash.as_ref(), Function::Sha256);
        let c = ByteArray::from_slice_unnamed(b"C");
        let d = ByteArray::from_slice_unnamed(b"D");
        let outer = c.concat(inner_hashed.clone()).concat(d);

        // test 1, split neatly at 1
        {
            let (left, right) = outer.clone().split(1).unwrap();
            let left_preimage = left.preimage.as_ref().expect("No preimage");
            let right_preimage = right.preimage.as_ref().expect("No preimage");
            assert_eq!(left.function, Function::Plain);
            assert_eq!(left.data.as_ref(), b"C");
            assert_eq!(left_preimage.len(), 1);
            assert_eq!(left_preimage[0].data.as_ref(), b"C");
            assert_eq!(left_preimage[0].preimage, None);
            assert_eq!(right.function, Function::Plain);
            assert_eq!(
                right.data.as_ref(),
                [inner_hash.as_slice(), b"D"].concat().as_slice()
            );
            assert_eq!(right_preimage.len(), 2);
            assert_eq!(right_preimage[0].function, Function::Sha256);
            assert_eq!(right_preimage[0].data.as_ref(), inner_hash.as_slice());
            assert_eq!(right_preimage[1].function, Function::Plain);
            assert_eq!(right_preimage[1].data.as_ref(), b"D");
            let right_preimage2 = right_preimage[0].preimage.as_ref().expect("No preimage");
            assert_eq!(right_preimage2[0].function, Function::Plain);
            assert_eq!(right_preimage2[0].data.as_ref(), b"AB");
            let right_preimage3 = right_preimage2[0].preimage.as_ref().expect("No preimage");
            assert_eq!(right_preimage3[0].function, Function::Plain);
            assert_eq!(right_preimage3[0].data.as_ref(), b"A");
            assert_eq!(right_preimage3[0].preimage, None);
            assert_eq!(right_preimage3[1].function, Function::Plain);
            assert_eq!(right_preimage3[1].data.as_ref(), b"B");
            assert_eq!(right_preimage3[1].preimage, None);
        }

        // test 2, split in middle of hash
        {
            let (left, right) = outer.clone().split(3).unwrap();
            let left_preimage = left.preimage.as_ref().expect("No preimage");
            let right_preimage = right.preimage.as_ref().expect("No preimage");
            assert_eq!(left.function, Function::Plain);
            assert_eq!(
                left.data.as_ref(),
                [b"C", &inner_hash[..2]].concat().as_slice()
            );
            assert_eq!(left_preimage.len(), 2);
            assert_eq!(left_preimage[0].function, Function::Plain);
            assert_eq!(left_preimage[0].data.as_ref(), b"C");
            assert_eq!(left_preimage[0].preimage, None);
            assert_eq!(left_preimage[1].function, Function::UnexpectedSplit);
            assert_eq!(left_preimage[1].data.as_ref(), &inner_hash[..2]);
            assert_eq!(left_preimage[1].preimage, None);
            assert_eq!(right.function, Function::Plain);
            assert_eq!(
                right.data.as_ref(),
                [&inner_hash[2..], b"D"].concat().as_slice()
            );
            assert_eq!(right_preimage[0].data.as_ref(), &inner_hash[2..]);
            assert_eq!(right_preimage[0].preimage, None);
            assert_eq!(right_preimage[1].data.as_ref(), b"D");
            assert_eq!(right_preimage[1].preimage, None);
        }

        // test 3, split neatly after hash
        {
            let (left, right) = outer.clone().split(33).unwrap();
            let left_preimage = left.preimage.as_ref().expect("No preimage");
            let right_preimage = right.preimage.as_ref().expect("No preimage");
            assert_eq!(left.function, Function::Plain);
            assert_eq!(
                left.data.as_ref(),
                [b"C", inner_hash.as_slice()].concat().as_slice()
            );
            assert_eq!(left_preimage.len(), 2);
            assert_eq!(left_preimage[0].function, Function::Plain);
            assert_eq!(left_preimage[0].data.as_ref(), b"C");
            assert_eq!(left_preimage[0].preimage, None);
            assert_eq!(left_preimage[1].function, Function::Sha256);
            assert_eq!(left_preimage[1].data.as_ref(), inner_hash.as_slice());
            assert_eq!(&left_preimage[1], &inner_hashed);
            assert_eq!(right.function, Function::Plain);
            assert_eq!(right.data.as_ref(), b"D");
            assert_eq!(right_preimage[0].data.as_ref(), b"D");
            assert_eq!(right_preimage[0].preimage, None);
        }
    }
}
