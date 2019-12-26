#[macro_use]
extern crate derive_error_chain;

pub mod address;
pub mod deserialize;
mod encoding_utils;
pub mod error;
mod hash;
pub mod serialize;

pub use address::{Address, AddressType, Prefix};
pub use deserialize::decode_bitcoin_code;
pub use hash::*;
pub use serialize::encode_bitcoin_code;
