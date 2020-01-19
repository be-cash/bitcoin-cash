#[macro_use]
extern crate derive_error_chain;

pub mod address;
pub mod deserializer;
mod encoding_utils;
pub mod error;
mod hash;
pub mod serializer;
mod tx_builder;
mod tx;
mod script;
mod tx_preimage;
mod scripts;
mod pubkey;
mod crypto;

pub use address::{Address, AddressType, Prefix};
pub use deserializer::decode_bitcoin_code;
pub use hash::*;
pub use serializer::*;
pub use script::*;
pub use tx::*;
pub use tx_builder::*;
pub use tx_preimage::*;
pub use scripts::*;
pub use pubkey::*;
pub use crypto::*;

pub use bitcoin_cash_script as ops;
pub use bitcoin_cash_script_macro::script;
pub use bitcoin_cash_script::ByteArray;
