use crate::{
    error::Result, BitcoinByteArray, BitcoinDataType, ByteArray, DataType, FixedByteArray,
    Function, Op,
};

use serde_derive::{Deserialize, Serialize};
use sha1::Digest;
use std::borrow::Cow;
use std::fmt::{Debug, Display};
use std::sync::Arc;

pub trait Hashed: Display + Debug + Sized + Eq + PartialEq {
    fn as_slice(&self) -> &[u8];
    fn from_slice(hash: &[u8]) -> Result<Self>;
    fn from_byte_array(hash: ByteArray) -> Result<Self>;
    fn function() -> Function;
    fn digest_slice(msg: &[u8]) -> Arc<[u8]>;
    fn digest(msg: impl Into<ByteArray>) -> Self {
        let msg = msg.into();
        let hash = Self::digest_slice(&msg);
        Self::from_byte_array(msg.apply_function(hash, Self::function()))
            .expect("Self::digest_slice produced invalid slice")
    }
    fn named(self, name: impl Into<Cow<'static, str>>) -> Self;
    fn from_hex_le(s: &str) -> Result<Self> {
        Self::from_slice(&hex::decode(s)?.iter().cloned().rev().collect::<Vec<_>>())
    }
    fn from_hex_be(s: &str) -> Result<Self> {
        Self::from_slice(&hex::decode(s)?)
    }
    fn from_slice_le(hash_le: &[u8]) -> Result<Self> {
        Self::from_slice(&hash_le.iter().cloned().rev().collect::<Vec<_>>())
    }
    fn to_vec_le(&self) -> Vec<u8> {
        self.as_slice().iter().cloned().rev().collect()
    }
    fn to_hex_le(&self) -> String {
        hex::encode(self.to_vec_le())
    }
    fn to_hex_be(&self) -> String {
        hex::encode(self.as_slice())
    }
    fn as_byte_array(&self) -> &ByteArray;
    fn into_byte_array(self) -> ByteArray;
    fn concat(self, other: impl Into<ByteArray>) -> ByteArray {
        self.into_byte_array().concat(other)
    }
}

#[derive(Clone, Eq, PartialEq, Default, Hash, Deserialize, Serialize)]
pub struct Sha1(FixedByteArray<[u8; 20]>);
#[derive(Clone, Eq, PartialEq, Default, Hash, Deserialize, Serialize)]
pub struct Ripemd160(FixedByteArray<[u8; 20]>);
#[derive(Clone, Eq, PartialEq, Default, Hash, Deserialize, Serialize)]
pub struct Sha256(FixedByteArray<[u8; 32]>);
#[derive(Clone, Eq, PartialEq, Default, Hash, Deserialize, Serialize)]
pub struct Sha256d(FixedByteArray<[u8; 32]>);
#[derive(Clone, Eq, PartialEq, Default, Hash, Deserialize, Serialize)]
pub struct Hash160(FixedByteArray<[u8; 20]>);

impl Sha1 {
    pub fn new(hash: [u8; 20]) -> Self {
        Sha1(FixedByteArray::new_unnamed(hash))
    }
}
impl Ripemd160 {
    pub fn new(hash: [u8; 20]) -> Self {
        Ripemd160(FixedByteArray::new_unnamed(hash))
    }
}
impl Sha256 {
    pub fn new(hash: [u8; 32]) -> Self {
        Sha256(FixedByteArray::new_unnamed(hash))
    }
}
impl Sha256d {
    pub fn new(hash: [u8; 32]) -> Self {
        Sha256d(FixedByteArray::new_unnamed(hash))
    }
}
impl Hash160 {
    pub fn new(hash: [u8; 20]) -> Self {
        Hash160(FixedByteArray::new_unnamed(hash))
    }
}

impl Debug for Sha1 {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "Sha1({})", self.to_hex_le())
    }
}
impl Debug for Ripemd160 {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "Ripemd160({})", self.to_hex_le())
    }
}
impl Debug for Sha256 {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "Sha256({})", self.to_hex_le())
    }
}
impl Debug for Sha256d {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "Sha256d({})", self.to_hex_le())
    }
}
impl Debug for Hash160 {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "Hash160({})", self.to_hex_le())
    }
}

impl Display for Sha1 {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "{}", self.to_hex_le())
    }
}
impl Display for Ripemd160 {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "{}", self.to_hex_le())
    }
}
impl Display for Sha256 {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "{}", self.to_hex_le())
    }
}
impl Display for Sha256d {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "{}", self.to_hex_le())
    }
}
impl Display for Hash160 {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "{}", self.to_hex_le())
    }
}

impl Hashed for Sha1 {
    fn function() -> Function {
        Function::Sha1
    }
    fn digest_slice(msg: &[u8]) -> Arc<[u8]> {
        sha1::Sha1::digest(msg).as_slice().into()
    }
    fn as_slice(&self) -> &[u8] {
        self.0.as_ref()
    }
    fn from_slice(hash: &[u8]) -> Result<Self> {
        Ok(Sha1(FixedByteArray::from_slice_unnamed(hash)?))
    }
    fn from_byte_array(hash: ByteArray) -> Result<Self> {
        Ok(Sha1(FixedByteArray::from_byte_array(hash)?))
    }
    fn named(self, name: impl Into<Cow<'static, str>>) -> Self {
        Sha1(self.0.named(name))
    }
    fn as_byte_array(&self) -> &ByteArray {
        self.0.as_byte_array()
    }
    fn into_byte_array(self) -> ByteArray {
        self.0.into_byte_array()
    }
}

impl Hashed for Ripemd160 {
    fn function() -> Function {
        Function::Ripemd160
    }
    fn digest_slice(msg: &[u8]) -> Arc<[u8]> {
        ripemd160::Ripemd160::digest(msg).as_slice().into()
    }
    fn as_slice(&self) -> &[u8] {
        self.0.as_ref()
    }
    fn from_slice(hash: &[u8]) -> Result<Self> {
        Ok(Ripemd160(FixedByteArray::from_slice_unnamed(hash)?))
    }
    fn from_byte_array(hash: ByteArray) -> Result<Self> {
        Ok(Ripemd160(FixedByteArray::from_byte_array(hash)?))
    }
    fn named(self, name: impl Into<Cow<'static, str>>) -> Self {
        Ripemd160(self.0.named(name))
    }
    fn as_byte_array(&self) -> &ByteArray {
        self.0.as_byte_array()
    }
    fn into_byte_array(self) -> ByteArray {
        self.0.into_byte_array()
    }
}

impl Hashed for Sha256 {
    fn function() -> Function {
        Function::Sha256
    }
    fn digest_slice(msg: &[u8]) -> Arc<[u8]> {
        sha2::Sha256::digest(msg).as_slice().into()
    }
    fn as_slice(&self) -> &[u8] {
        self.0.as_ref()
    }
    fn from_slice(hash: &[u8]) -> Result<Self> {
        Ok(Sha256(FixedByteArray::from_slice_unnamed(hash)?))
    }
    fn from_byte_array(hash: ByteArray) -> Result<Self> {
        Ok(Sha256(FixedByteArray::from_byte_array(hash)?))
    }
    fn named(self, name: impl Into<Cow<'static, str>>) -> Self {
        Sha256(self.0.named(name))
    }
    fn as_byte_array(&self) -> &ByteArray {
        self.0.as_byte_array()
    }
    fn into_byte_array(self) -> ByteArray {
        self.0.into_byte_array()
    }
}

impl Hashed for Sha256d {
    fn function() -> Function {
        Function::Hash256
    }
    fn digest_slice(msg: &[u8]) -> Arc<[u8]> {
        sha2::Sha256::digest(sha2::Sha256::digest(msg).as_slice())
            .as_slice()
            .into()
    }
    fn as_slice(&self) -> &[u8] {
        self.0.as_ref()
    }
    fn from_slice(hash: &[u8]) -> Result<Self> {
        Ok(Sha256d(FixedByteArray::from_slice_unnamed(hash)?))
    }
    fn from_byte_array(hash: ByteArray) -> Result<Self> {
        Ok(Sha256d(FixedByteArray::from_byte_array(hash)?))
    }
    fn named(self, name: impl Into<Cow<'static, str>>) -> Self {
        Sha256d(self.0.named(name))
    }
    fn as_byte_array(&self) -> &ByteArray {
        self.0.as_byte_array()
    }
    fn into_byte_array(self) -> ByteArray {
        self.0.into_byte_array()
    }
}

impl Hashed for Hash160 {
    fn function() -> Function {
        Function::Hash160
    }
    fn digest_slice(msg: &[u8]) -> Arc<[u8]> {
        ripemd160::Ripemd160::digest(sha2::Sha256::digest(msg).as_slice())
            .as_slice()
            .into()
    }
    fn as_slice(&self) -> &[u8] {
        self.0.as_ref()
    }
    fn from_slice(hash: &[u8]) -> Result<Self> {
        Ok(Hash160(FixedByteArray::from_slice_unnamed(hash)?))
    }
    fn from_byte_array(hash: ByteArray) -> Result<Self> {
        Ok(Hash160(FixedByteArray::from_byte_array(hash)?))
    }
    fn named(self, name: impl Into<Cow<'static, str>>) -> Self {
        Hash160(self.0.named(name))
    }
    fn as_byte_array(&self) -> &ByteArray {
        self.0.as_byte_array()
    }
    fn into_byte_array(self) -> ByteArray {
        self.0.into_byte_array()
    }
}

impl BitcoinDataType for Sha1 {
    type Type = BitcoinByteArray;
    fn to_data(&self) -> Self::Type {
        BitcoinByteArray(self.as_byte_array().clone())
    }
    fn to_pushop(&self) -> Op {
        self.as_byte_array().clone().into()
    }
    fn to_data_type(&self) -> DataType {
        DataType::ByteArray(Some(self.0.len()))
    }
}

impl BitcoinDataType for Ripemd160 {
    type Type = BitcoinByteArray;
    fn to_data(&self) -> Self::Type {
        BitcoinByteArray(self.as_byte_array().clone())
    }
    fn to_pushop(&self) -> Op {
        self.as_byte_array().clone().into()
    }
    fn to_data_type(&self) -> DataType {
        DataType::ByteArray(Some(self.0.len()))
    }
}

impl BitcoinDataType for Sha256 {
    type Type = BitcoinByteArray;
    fn to_data(&self) -> Self::Type {
        BitcoinByteArray(self.as_byte_array().clone())
    }
    fn to_pushop(&self) -> Op {
        self.as_byte_array().clone().into()
    }
    fn to_data_type(&self) -> DataType {
        DataType::ByteArray(Some(self.0.len()))
    }
}

impl BitcoinDataType for Sha256d {
    type Type = BitcoinByteArray;
    fn to_data(&self) -> Self::Type {
        BitcoinByteArray(self.as_byte_array().clone())
    }
    fn to_pushop(&self) -> Op {
        self.as_byte_array().clone().into()
    }
    fn to_data_type(&self) -> DataType {
        DataType::ByteArray(Some(self.0.len()))
    }
}

impl BitcoinDataType for Hash160 {
    type Type = BitcoinByteArray;
    fn to_data(&self) -> Self::Type {
        BitcoinByteArray(self.as_byte_array().clone())
    }
    fn to_pushop(&self) -> Op {
        self.as_byte_array().clone().into()
    }
    fn to_data_type(&self) -> DataType {
        DataType::ByteArray(Some(self.0.len()))
    }
}

impl From<Sha1> for ByteArray {
    fn from(hash: Sha1) -> Self {
        hash.into_byte_array()
    }
}
impl From<Ripemd160> for ByteArray {
    fn from(hash: Ripemd160) -> Self {
        hash.into_byte_array()
    }
}
impl From<Sha256> for ByteArray {
    fn from(hash: Sha256) -> Self {
        hash.into_byte_array()
    }
}
impl From<Sha256d> for ByteArray {
    fn from(hash: Sha256d) -> Self {
        hash.into_byte_array()
    }
}
impl From<Hash160> for ByteArray {
    fn from(hash: Hash160) -> Self {
        hash.into_byte_array()
    }
}

#[cfg(test)]
mod tests {
    use super::{Hash160, Hashed, Result, Ripemd160, Sha1, Sha256, Sha256d};
    use crate::error::ErrorKind;
    use hex_literal::hex;

    #[test]
    fn test_sha1_slice() -> Result<()> {
        const EMPTY_SHA1: [u8; 20] = hex!("da39a3ee5e6b4b0d3255bfef95601890afd80709");
        assert_eq!(Sha1::digest(b"").as_slice(), EMPTY_SHA1);
        assert_eq!(Sha1::digest(b""), Sha1::from_slice(&EMPTY_SHA1)?);
        assert_eq!(
            ErrorKind::InvalidSize((20, 2)).to_string(),
            Sha1::from_slice(&[0, 0]).unwrap_err().kind().to_string(),
        );
        Ok(())
    }

    #[test]
    fn test_sha1_hex_be() -> Result<()> {
        const EMPTY_SHA1: &str = "da39a3ee5e6b4b0d3255bfef95601890afd80709";
        assert_eq!(Sha1::digest(b"").to_hex_be(), EMPTY_SHA1);
        assert_eq!(Sha1::digest(b""), Sha1::from_hex_be(EMPTY_SHA1)?);
        Ok(())
    }

    #[test]
    fn test_sha1_hex_le() -> Result<()> {
        const EMPTY_SHA1_LE: &str = "0907d8af90186095efbf55320d4b6b5eeea339da";
        assert_eq!(Sha1::digest(b"").to_hex_le(), EMPTY_SHA1_LE);
        assert_eq!(Sha1::digest(b""), Sha1::from_hex_le(EMPTY_SHA1_LE)?);
        Ok(())
    }

    #[test]
    fn test_ripemd160_slice() -> Result<()> {
        const EMPTY_RIPEMD: [u8; 20] = hex!("9c1185a5c5e9fc54612808977ee8f548b2258d31");
        assert_eq!(Ripemd160::digest(b"").as_slice(), EMPTY_RIPEMD);
        assert_eq!(
            Ripemd160::digest(b""),
            Ripemd160::from_slice(&EMPTY_RIPEMD)?
        );
        assert_eq!(
            ErrorKind::InvalidSize((20, 2)).to_string(),
            Ripemd160::from_slice(&[0, 0])
                .unwrap_err()
                .kind()
                .to_string(),
        );
        Ok(())
    }

    #[test]
    fn test_ripemd160_hex_be() -> Result<()> {
        const EMPTY_RIPEMD: &str = "9c1185a5c5e9fc54612808977ee8f548b2258d31";
        assert_eq!(Ripemd160::digest(b"").to_hex_be(), EMPTY_RIPEMD);
        assert_eq!(
            Ripemd160::digest(b""),
            Ripemd160::from_hex_be(EMPTY_RIPEMD)?
        );
        Ok(())
    }

    #[test]
    fn test_ripemd160_hex_le() -> Result<()> {
        const EMPTY_RIPEMD_LE: &str = "318d25b248f5e87e9708286154fce9c5a585119c";
        assert_eq!(Ripemd160::digest(b"").to_hex_le(), EMPTY_RIPEMD_LE);
        assert_eq!(
            Ripemd160::digest(b""),
            Ripemd160::from_hex_le(EMPTY_RIPEMD_LE)?
        );
        Ok(())
    }

    #[test]
    fn test_sha256_slice() -> Result<()> {
        const EMPTY_SHA256: [u8; 32] =
            hex!("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        assert_eq!(Sha256::digest(b"").as_slice(), EMPTY_SHA256);
        assert_eq!(Sha256::digest(b""), Sha256::from_slice(&EMPTY_SHA256)?);
        assert_eq!(
            ErrorKind::InvalidSize((32, 2)).to_string(),
            Sha256::from_slice(&[0, 0]).unwrap_err().kind().to_string(),
        );
        Ok(())
    }

    #[test]
    fn test_sha256_hex_be() -> Result<()> {
        const EMPTY_SHA256: &str =
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        assert_eq!(Sha256::digest(b"").to_hex_be(), EMPTY_SHA256);
        assert_eq!(Sha256::digest(b""), Sha256::from_hex_be(EMPTY_SHA256)?);
        Ok(())
    }

    #[test]
    fn test_sha256_hex_le() -> Result<()> {
        const EMPTY_SHA256_LE: &str =
            "55b852781b9995a44c939b64e441ae2724b96f99c8f4fb9a141cfc9842c4b0e3";
        assert_eq!(Sha256::digest(b"").to_hex_le(), EMPTY_SHA256_LE);
        assert_eq!(Sha256::digest(b""), Sha256::from_hex_le(EMPTY_SHA256_LE)?);
        Ok(())
    }

    #[test]
    fn test_sha256d_slice() -> Result<()> {
        const EMPTY_SHA256D: [u8; 32] =
            hex!("5df6e0e2761359d30a8275058e299fcc0381534545f55cf43e41983f5d4c9456");
        assert_eq!(Sha256d::digest(b"").as_slice(), EMPTY_SHA256D);
        assert_eq!(Sha256d::digest(b""), Sha256d::from_slice(&EMPTY_SHA256D)?);
        assert_eq!(
            ErrorKind::InvalidSize((32, 2)).to_string(),
            Sha256d::from_slice(&[0, 0]).unwrap_err().kind().to_string(),
        );
        Ok(())
    }

    #[test]
    fn test_sha256d_hex_be() -> Result<()> {
        const EMPTY_SHA256D: &str =
            "5df6e0e2761359d30a8275058e299fcc0381534545f55cf43e41983f5d4c9456";
        assert_eq!(Sha256d::digest(b"").to_hex_be(), EMPTY_SHA256D);
        assert_eq!(Sha256d::digest(b""), Sha256d::from_hex_be(EMPTY_SHA256D)?);
        Ok(())
    }

    #[test]
    fn test_sha256d_hex_le() -> Result<()> {
        const EMPTY_SHA256D_LE: &str =
            "56944c5d3f98413ef45cf54545538103cc9f298e0575820ad3591376e2e0f65d";
        assert_eq!(Sha256d::digest(b"").to_hex_le(), EMPTY_SHA256D_LE);
        assert_eq!(
            Sha256d::digest(b""),
            Sha256d::from_hex_le(EMPTY_SHA256D_LE)?
        );
        Ok(())
    }

    #[test]
    fn test_hash160_slice() -> Result<()> {
        const EMPTY_HASH160: [u8; 20] = hex!("b472a266d0bd89c13706a4132ccfb16f7c3b9fcb");
        assert_eq!(Hash160::digest(b"").as_slice(), EMPTY_HASH160);
        assert_eq!(Hash160::digest(b""), Hash160::from_slice(&EMPTY_HASH160)?);
        assert_eq!(
            ErrorKind::InvalidSize((20, 2)).to_string(),
            Hash160::from_slice(&[0, 0]).unwrap_err().kind().to_string(),
        );
        Ok(())
    }

    #[test]
    fn test_hash160_hex_be() -> Result<()> {
        const EMPTY_HASH160: &str = "b472a266d0bd89c13706a4132ccfb16f7c3b9fcb";
        assert_eq!(Hash160::digest(b"").to_hex_be(), EMPTY_HASH160);
        assert_eq!(Hash160::digest(b""), Hash160::from_hex_be(EMPTY_HASH160)?);
        Ok(())
    }

    #[test]
    fn test_hash160_hex_le() -> Result<()> {
        const EMPTY_HASH160_LE: &str = "cb9f3b7c6fb1cf2c13a40637c189bdd066a272b4";
        assert_eq!(Hash160::digest(b"").to_hex_le(), EMPTY_HASH160_LE);
        assert_eq!(
            Hash160::digest(b""),
            Hash160::from_hex_le(EMPTY_HASH160_LE)?
        );
        Ok(())
    }
}
