use crate::address::CashAddrError;
use error_chain::error_chain;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BitcoinCodeError {
    DeserializeAnyNotSupported,
    InvalidBoolEncoding(u8),
    LeftoverBytes,
    DataTypeNotSupported(&'static str),
    MethodNotSupported(&'static str),
    SequenceMustHaveLength,
}

impl BitcoinCodeError {
    pub fn into_err<T>(self) -> Result<T> {
        Err(ErrorKind::BitcoinCodeDeserialize(self).into())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScriptSerializeError {
    PushTooLarge,
    InvalidInteger,
    UnknownOpcode,
}

impl ScriptSerializeError {
    pub fn into_err<T>(self) -> Result<T> {
        Err(ErrorKind::ScriptSerialize(self).into())
    }
}

error_chain! {
    links {
        Json(
            crate::serialize_json::error::Error,
            crate::serialize_json::error::ErrorKind
        );
    }

    foreign_links {
        FromHex(hex::FromHexError);
        Io(std::io::Error);
        Utf8(std::str::Utf8Error);
        SerdeJson(serde_json::Error);
    }

    errors {
        InvalidSize(expected: usize, actual: usize) {
            description("invalid size")
            display("invalid size, expected {}, got {}", expected, actual)
        }

        InvalidCashAddr(err: CashAddrError) {}

        BitcoinCodeDeserialize(err: BitcoinCodeError) {}

        ScriptSerialize(err: ScriptSerializeError) {}

        InsufficientInputAmount(amount: u64) {}

        InvalidSignatureFormat {}

        InvalidPubkey {}

        InputAlreadySigned(input_idx: usize) {}

        InvalidAddressType {}
    }
}

impl From<bitcoin_cash_base::FromSliceError> for Error {
    fn from(error: bitcoin_cash_base::FromSliceError) -> Self {
        ErrorKind::InvalidSize(error.expected, error.actual).into()
    }
}
