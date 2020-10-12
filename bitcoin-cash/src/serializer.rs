use crate::error;
use crate::{BitcoinCode, ByteArray};

pub fn encode_bitcoin_code_all<'a, T: 'a>(
    values: impl IntoIterator<Item = &'a T>,
) -> error::Result<ByteArray>
where
    T: BitcoinCode,
{
    Ok(ByteArray::from_parts(
        values.into_iter().map(|value| value.ser()),
    ))
}
