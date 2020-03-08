use crate::address::CashAddrError;

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

#[derive(Debug, ErrorChain)]
pub enum ErrorKind {
    Msg(String),

    #[error_chain(foreign)]
    FromHex(hex::FromHexError),

    #[error_chain(foreign)]
    Io(std::io::Error),

    #[error_chain(foreign)]
    Utf8(std::str::Utf8Error),

    #[error_chain(custom)]
    #[error_chain(description = |_| "invalid size")]
    #[error_chain(display = |t: &(_, _)| write!(f, "invalid size, expected {}, got {}", t.0, t.1))]
    InvalidSize((usize, usize)),

    #[error_chain(custom)]
    InvalidCashAddr(CashAddrError),

    #[error_chain(custom)]
    BitcoinCodeDeserialize(BitcoinCodeError),

    #[error_chain(custom)]
    ScriptSerialize(ScriptSerializeError),

    #[error_chain(custom)]
    InsufficientInputAmount(u64),

    #[error_chain(custom)]
    InvalidSignatureFormat,

    #[error_chain(custom)]
    InvalidPubkey,
}
