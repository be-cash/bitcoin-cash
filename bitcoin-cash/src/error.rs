use crate::{address::CashAddrError, ByteArrayError, IntegerError, JsonError};

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

    #[error("Integer error: {0}")]
    IntegerError(#[from] IntegerError),

    #[error("Byte array error: {0}")]
    ByteArrayError(#[from] ByteArrayError),

    #[error("{0}")]
    Msg(String),
}

impl From<String> for Error {
    fn from(msg: String) -> Self {
        Error::Msg(msg)
    }
}
