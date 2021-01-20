use crate::{BitcoinCode, ByteArray, error::Result};

pub fn encode_bitcoin_code_all<'a, T: 'a>(
    values: impl IntoIterator<Item = &'a T>,
) -> ByteArray
where
    T: BitcoinCode,
{
    ByteArray::from_parts(
        values.into_iter().map(|value| value.ser()),
    )
}

pub fn decode_bitcoin_code_all<T: BitcoinCode>(mut byte_array: ByteArray) -> Result<Vec<T>> {
    let mut items = Vec::new();
    loop {
        let (item, rest) = T::deser_rest(byte_array)?;
        byte_array = rest;
        items.push(item);
        if byte_array.is_empty() {
            return Ok(items);
        }
    }
}
