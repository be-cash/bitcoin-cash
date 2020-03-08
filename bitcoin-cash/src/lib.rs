#[macro_use]
extern crate derive_error_chain;

pub mod address;
pub mod deserializer;
mod ecc;
mod encoding_utils;
pub mod error;
mod hash;
mod pubkey;
mod script;
mod scripts;
pub mod serializer;
mod tx;
mod tx_builder;
mod tx_preimage;

pub use address::{Address, AddressType, Prefix};
pub use deserializer::decode_bitcoin_code;
pub use ecc::*;
pub use hash::*;
pub use pubkey::*;
pub use script::*;
pub use scripts::*;
pub use serializer::*;
pub use tx::*;
pub use tx_builder::*;
pub use tx_preimage::*;

pub use bitcoin_cash_script as ops;
pub use bitcoin_cash_script::ByteArray;
pub use bitcoin_cash_script_macro::script;
