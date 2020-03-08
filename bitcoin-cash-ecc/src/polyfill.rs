#[cfg(feature = "rust_ecc")]
pub type SelectedECC = crate::rust_ecc::RustECC;

#[cfg(feature = "c_ecc")]
pub type SelectedECC = crate::c_ecc::CECC;

pub fn init_ecc() -> SelectedECC {
    SelectedECC::default()
}
