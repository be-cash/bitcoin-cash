use crate::address::CashAddrError;
use crate::deserialize::BitcoinCodeError;

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
    #[error_chain(description = |_| "invalid hash size")]
    #[error_chain(display = |t: &(_, _)| write!(f, "invalid hash size, expected {}, got {}", t.0, t.1))]
    InvalidHashSize((usize, usize)),

    #[error_chain(custom)]
    InvalidCashAddr(CashAddrError),

    #[error_chain(custom)]
    BitcoinCodeDeserialize(BitcoinCodeError),
}
