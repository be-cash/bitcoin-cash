use crate::address::CashAddrError;

#[derive(Debug, ErrorChain, PartialEq)]
pub enum ErrorKind {
    Msg(String),

    #[error_chain(foreign)]
    Fmt(hex::FromHexError),

    #[error_chain(custom)]
    #[error_chain(description = |_| "invalid hash size")]
    #[error_chain(display = |t: &(_, _)| write!(f, "invalid hash size, expected {}, got {}", t.0, t.1))]
    InvalidHashSize((usize, usize)),

    #[error_chain(custom)]
    InvalidCashAddr(CashAddrError),
}
