use crate::address::CashAddrError;
use crate::serialize_json::JsonError;

#[derive(Error, Clone, Copy, Debug, PartialEq)]
pub enum BitcoinCodeError {
    #[error("Deserialize any not supported")]
    DeserializeAnyNotSupported,

    #[error("Invalid bool encoding: {0}")]
    InvalidBoolEncoding(u8),

    #[error("Leftover bytes")]
    LeftoverBytes,

    #[error("Datatype {0} not supported")]
    DataTypeNotSupported(&'static str),

    #[error("Method {0} not supported")]
    MethodNotSupported(&'static str),
}

impl BitcoinCodeError {
    pub fn into_err<T>(self) -> Result<T> {
        Err(Error::BitcoinCodeDeserialize(self))
    }
}

#[derive(Error, Clone, Debug, PartialEq)]
pub enum ScriptSerializeError {
    #[error("Push too large")]
    PushTooLarge,
    #[error("Invalid integer")]
    InvalidInteger,
}

impl ScriptSerializeError {
    pub fn into_err<T>(self) -> Result<T> {
        Err(Error::ScriptSerialize(self))
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid CashAddr")]
    InvalidCashAddr(#[from] CashAddrError),

    #[error("Invalid hex: {0}")]
    FromHex(#[from] hex::FromHexError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Utf8 error: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("Serde JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("JSON error: {0}")]
    Json(#[from] JsonError),

    #[error("Invalid size: expected {expected}, got {actual}")]
    InvalidSize { expected: usize, actual: usize },

    #[error("Bitcoin code error: {0}")]
    BitcoinCodeDeserialize(#[from] BitcoinCodeError),

    #[error("Script serialize error: {0}")]
    ScriptSerialize(#[from] ScriptSerializeError),

    #[error("Script serialize error: {amount}")]
    InsufficientInputAmount { amount: u64 },

    #[error("Invalid signature format")]
    InvalidSignatureFormat,

    #[error("Invalid invalid pubkey")]
    InvalidPubkey,

    #[error("Input {input_idx} already spent")]
    InputAlreadySigned { input_idx: usize },

    #[error("Invalid address type")]
    InvalidAddressType,

    #[error("{0}")]
    Msg(String),
}

impl From<String> for Error {
    fn from(msg: String) -> Self {
        Error::Msg(msg)
    }
}

impl From<bitcoin_cash_base::FromSliceError> for Error {
    fn from(error: bitcoin_cash_base::FromSliceError) -> Self {
        Error::InvalidSize {
            expected: error.expected,
            actual: error.actual,
        }
    }
}
