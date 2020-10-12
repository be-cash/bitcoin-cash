use crate::{
    encoding_utils::{encode_var_int, read_var_int},
    error::Result,
    ByteArray, FixedByteArray,
};

pub trait BitcoinCode: Sized {
    fn ser(&self) -> ByteArray;
    fn deser(data: ByteArray) -> Result<(Self, ByteArray)>;
}

fn read_size(data: ByteArray) -> Result<(usize, ByteArray)> {
    let mut cursor = std::io::Cursor::new(data.as_slice());
    let len = read_var_int(&mut cursor)? as usize;
    let position = cursor.position() as usize;
    let (_, rest) = data.split(position)?;
    Ok((len, rest))
}

impl BitcoinCode for ByteArray {
    fn ser(&self) -> ByteArray {
        ByteArray::new("size", encode_var_int(self.len() as u64)).concat(self.clone())
    }

    fn deser(data: ByteArray) -> Result<(Self, ByteArray)> {
        let (len, rest) = read_size(data)?;
        let (byte_array, rest) = rest.split(len)?;
        Ok((byte_array, rest))
    }
}

impl<T> BitcoinCode for FixedByteArray<T>
where
    T: Default + AsRef<[u8]>,
{
    fn ser(&self) -> ByteArray {
        self.as_byte_array().clone()
    }

    fn deser(data: ByteArray) -> Result<(Self, ByteArray)> {
        let array = T::default();
        let split_idx = array.as_ref().len();
        let (left, right) = data.split(split_idx)?;
        let fixed_byte_array = Self::from_byte_array(left)?;
        Ok((fixed_byte_array, right))
    }
}

impl<T: BitcoinCode> BitcoinCode for Vec<T> {
    fn ser(&self) -> ByteArray {
        let parts = self.iter().map(|item| item.ser());
        let data = ByteArray::from_parts(parts);
        ByteArray::new("size", encode_var_int(self.len() as u64)).concat(data)
    }

    fn deser(data: ByteArray) -> Result<(Self, ByteArray)> {
        let (len, mut byte_array) = read_size(data)?;
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            let (item, rest) = T::deser(byte_array)?;
            vec.push(item);
            byte_array = rest;
        }
        Ok((vec, byte_array))
    }
}

impl BitcoinCode for bool {
    fn ser(&self) -> ByteArray {
        [*self as u8].into()
    }

    fn deser(data: ByteArray) -> Result<(Self, ByteArray)> {
        let (left, right) = data.split(1)?;
        Ok((left[0] != 0, right))
    }
}

macro_rules! array_impls {
    ($($T:ident)+) => {
        $(
            impl BitcoinCode for $T {
                fn ser(&self) -> ByteArray {
                    self.to_le_bytes().into()
                }

                fn deser(data: ByteArray) -> Result<(Self, ByteArray)> {
                    let split_idx = std::mem::size_of::<$T>();
                    let (left, right) = data.split(split_idx)?;
                    let mut array = [0; std::mem::size_of::<$T>()];
                    array.copy_from_slice(&left);
                    let value = $T::from_le_bytes(array);
                    Ok((value, right))
                }
            }
        )+
    }
}

array_impls! {
    u8 i8 u16 i16 u32 i32 u64 i64 u128 i128
}
