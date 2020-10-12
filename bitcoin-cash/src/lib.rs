#[macro_use]
extern crate thiserror;

mod address;
mod bitcoin_code;
mod deserializer;
mod ecc;
pub mod encoding_utils;
pub mod error;
mod hash;
mod ops;
mod pubkey;
mod script;
mod scripts;
mod serialize_json;
mod serializer;
mod tagged_op;
mod tx;
mod tx_builder;
mod tx_preimage;

pub use address::{Address, AddressType, Prefix};
pub use bitcoin_code::*;
pub use deserializer::*;
pub use ecc::*;
pub use hash::*;
pub use ops::*;
pub use pubkey::*;
pub use script::*;
pub use scripts::*;
pub use serialize_json::*;
pub use serializer::encode_bitcoin_code_all;
pub use tagged_op::*;
pub use tx::*;
pub use tx_builder::*;
pub use tx_preimage::*;

pub use bitcoin_cash_base::*;
pub use bitcoin_cash_code::*;
pub use bitcoin_cash_script_macro::script;
