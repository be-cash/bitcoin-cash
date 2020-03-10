#[macro_use]
extern crate derive_error_chain;

mod address;
mod deserializer;
mod ecc;
pub mod encoding_utils;
pub mod error;
mod hash;
mod ops;
mod pubkey;
mod script;
mod scripts;
mod serializer;
mod tagged_op;
mod tx;
mod tx_builder;
mod tx_preimage;

pub use address::{Address, AddressType, Prefix};
pub use deserializer::*;
pub use ecc::*;
pub use hash::*;
pub use ops::*;
pub use pubkey::*;
pub use script::*;
pub use scripts::*;
pub use serializer::*;
pub use tagged_op::*;
pub use tx::*;
pub use tx_builder::*;
pub use tx_preimage::*;

pub use bitcoin_cash_script::*;
pub use bitcoin_cash_script_macro::script;
