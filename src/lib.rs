#[macro_use]
extern crate derive_error_chain;


mod hash;
pub mod error;
pub mod address;

pub use hash::*;
pub use address::{Address, AddressType, Prefix};
