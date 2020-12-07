use crate::{BitcoinCode, ByteArray};

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
