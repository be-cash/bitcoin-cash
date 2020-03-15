use num_derive::*;
use std::borrow::Cow;

use crate::error::{Error, ErrorKind, Result};
use crate::{serialize_ops, Hash160, Hashed, Ops, Script};

const CHARSET: &[u8] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Address<'a> {
    addr_type: AddressType,
    hash: Hash160,
    cash_addr: Cow<'a, str>,
    prefix: AddressPrefix<'a>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, FromPrimitive)]
pub enum AddressType {
    P2PKH = 0,
    P2SH = 8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Prefix {
    BitcoinCash,
    SimpleLedger,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct AddressPrefix<'a> {
    prefix_str: Cow<'a, str>,
    prefix_kind: Option<Prefix>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CashAddrError {
    InvalidChecksum,
    InvalidBase32Letter(usize, u8),
    InvalidAddressType(u8),
}

impl Prefix {
    pub fn prefix_str(self) -> &'static str {
        match self {
            Prefix::BitcoinCash => "bitcoincash",
            Prefix::SimpleLedger => "simpleledger",
        }
    }

    pub fn from_prefix_str(prefix_str: &str) -> Option<Prefix> {
        match prefix_str {
            "bitcoincash" => Some(Prefix::BitcoinCash),
            "simpleledger" => Some(Prefix::SimpleLedger),
            _ => None,
        }
    }
}

impl Default for Prefix {
    fn default() -> Self {
        Prefix::BitcoinCash
    }
}

impl<'a> Into<AddressPrefix<'a>> for Prefix {
    fn into(self) -> AddressPrefix<'a> {
        AddressPrefix {
            prefix_str: self.prefix_str().into(),
            prefix_kind: Some(self),
        }
    }
}

impl<'a> AddressPrefix<'a> {
    pub fn new(prefix_str: Cow<'a, str>, prefix_kind: Option<Prefix>) -> Self {
        AddressPrefix {
            prefix_str,
            prefix_kind,
        }
    }

    pub fn prefix_str(&self) -> &str {
        &self.prefix_str
    }

    pub fn prefix_kind(&self) -> Option<Prefix> {
        self.prefix_kind
    }
}

impl<'a> Into<AddressPrefix<'a>> for &'a str {
    fn into(self) -> AddressPrefix<'a> {
        AddressPrefix {
            prefix_str: self.into(),
            prefix_kind: Prefix::from_prefix_str(self),
        }
    }
}

impl<'a> Address<'a> {
    pub fn from_hash<P: Into<AddressPrefix<'a>>>(
        prefix: P,
        addr_type: AddressType,
        hash: Hash160,
    ) -> Address<'a> {
        let prefix = prefix.into();
        Address {
            cash_addr: _to_cash_addr(prefix.prefix_str(), addr_type, hash.as_slice()).into(),
            addr_type,
            hash,
            prefix,
        }
    }

    pub fn from_cash_addr(cash_addr: &'a str) -> Result<Address<'a>> {
        let (hash, addr_type, prefix) = _from_cash_addr(cash_addr, Prefix::default().prefix_str())
            .map_err(|err| -> Error { ErrorKind::InvalidCashAddr(err).into() })?;
        let prefix_kind = Prefix::from_prefix_str(&prefix);
        Ok(Address {
            cash_addr: cash_addr.into(),
            addr_type,
            hash: Hash160::from_slice(&hash)?,
            prefix: AddressPrefix::new(prefix, prefix_kind),
        })
    }

    pub fn from_redeem_script<P: Into<AddressPrefix<'a>>>(
        prefix: P,
        redeem_script: Script,
    ) -> Result<Address<'a>> {
        Ok(Address::from_hash(
            prefix,
            AddressType::P2SH,
            Hash160::digest(serialize_ops(redeem_script.ops().iter().map(|op| &op.op))?),
        ))
    }

    pub fn hash(&self) -> &Hash160 {
        &self.hash
    }

    pub fn prefix_str(&self) -> &str {
        self.prefix.prefix_str()
    }

    pub fn prefix_kind(&self) -> Option<Prefix> {
        self.prefix.prefix_kind()
    }

    pub fn cash_addr(&self) -> &str {
        &self.cash_addr
    }

    pub fn addr_type(&self) -> AddressType {
        self.addr_type
    }

    pub fn with_prefix<P: Into<AddressPrefix<'a>>>(&'a self, prefix: P) -> Address<'a> {
        Self::from_hash(prefix, self.addr_type, self.hash.clone())
    }

    pub fn to_owned_address(&self) -> Address<'static> {
        Address {
            addr_type: self.addr_type,
            hash: self.hash.clone(),
            cash_addr: self.cash_addr.to_string().into(),
            prefix: AddressPrefix {
                prefix_str: self.prefix.prefix_str.to_string().into(),
                prefix_kind: self.prefix.prefix_kind,
            },
        }
    }
}

fn _map_to_b32(data: impl Iterator<Item = u8>) -> String {
    String::from_utf8(data.map(|x| CHARSET[x as usize]).collect()).unwrap()
}

fn _map_from_b32(string: &str) -> std::result::Result<Vec<u8>, CashAddrError> {
    string
        .as_bytes()
        .iter()
        .enumerate()
        .map(|(i, x)| {
            CHARSET
                .iter()
                .position(|c| x == c)
                .map(|x| x as u8)
                .ok_or(CashAddrError::InvalidBase32Letter(i, *x))
        })
        .collect()
}

fn _convert_bits(
    data: impl Iterator<Item = u8>,
    from_bits: u32,
    to_bits: u32,
    pad: bool,
) -> Option<Vec<u8>> {
    let mut acc = 0;
    let mut bits = 0;
    let mut ret = Vec::new();
    let maxv = (1 << to_bits) - 1;
    let max_acc = (1 << (from_bits + to_bits - 1)) - 1;
    for value in data {
        let value = value as u32;
        if (value >> from_bits) != 0 {
            return None;
        }
        acc = ((acc << from_bits) | value) & max_acc;
        bits += from_bits;
        while bits >= to_bits {
            bits -= to_bits;
            ret.push(((acc >> bits) & maxv) as u8);
        }
    }
    if pad {
        if bits != 0 {
            ret.push(((acc << (to_bits - bits)) & maxv) as u8);
        }
    } else if bits >= from_bits || ((acc << (to_bits - bits)) & maxv != 0) {
        return None;
    }
    Some(ret)
}

fn _poly_mod(values: impl Iterator<Item = u8>) -> u64 {
    let mut c = 1;
    for value in values {
        let c0 = (c >> 35) as u8;
        c = ((c & 0x07_ffff_ffffu64) << 5u64) ^ (value as u64);
        if c0 & 0x01 != 0 {
            c ^= 0x98_f2bc_8e61
        }
        if c0 & 0x02 != 0 {
            c ^= 0x79_b76d_99e2
        }
        if c0 & 0x04 != 0 {
            c ^= 0xf3_3e5f_b3c4
        }
        if c0 & 0x08 != 0 {
            c ^= 0xae_2eab_e2a8
        }
        if c0 & 0x10 != 0 {
            c ^= 0x1e_4f43_e470
        }
    }
    c ^ 1
}

fn _calculate_checksum(prefix: &str, payload: impl Iterator<Item = u8>) -> Vec<u8> {
    let poly = _poly_mod(
        prefix
            .as_bytes()
            .iter()
            .map(|x| *x & 0x1f)
            .chain([0].iter().cloned())
            .chain(payload)
            .chain([0, 0, 0, 0, 0, 0, 0, 0].iter().cloned()),
    );
    (0..8)
        .map(|i| ((poly >> (5 * (7 - i))) & 0x1f) as u8)
        .collect()
}

fn _verify_checksum(prefix: &str, payload: impl Iterator<Item = u8>) -> bool {
    let poly = _poly_mod(
        prefix
            .as_bytes()
            .iter()
            .map(|x| *x & 0x1f)
            .chain([0].iter().cloned())
            .chain(payload),
    );
    poly == 0
}

fn _to_cash_addr(prefix: &str, addr_type: AddressType, addr_bytes: &[u8]) -> String {
    let version = addr_type as u8;
    let payload = _convert_bits(
        [version].iter().chain(addr_bytes.iter()).cloned(),
        8,
        5,
        true,
    )
    .unwrap();
    let checksum = _calculate_checksum(prefix, payload.iter().cloned());
    String::from(prefix)
        + ":"
        + &_map_to_b32(payload.iter().cloned().chain(checksum.iter().cloned()))
}

fn _from_cash_addr<'a>(
    addr_string: &str,
    default_prefix: &'a str,
) -> std::result::Result<([u8; 20], AddressType, Cow<'a, str>), CashAddrError> {
    let addr_string = addr_string.to_ascii_lowercase();
    let (prefix, payload_base32): (Cow<'a, _>, _) = if let Some(pos) = addr_string.find(':') {
        let (prefix, payload_base32) = addr_string.split_at(pos + 1);
        (
            prefix[..prefix.len() - 1].to_string().into(),
            payload_base32,
        )
    } else {
        (default_prefix.into(), &addr_string[..])
    };
    let decoded = _map_from_b32(payload_base32)?;
    if !_verify_checksum(&prefix, decoded.iter().cloned()) {
        return Err(CashAddrError::InvalidChecksum);
    }
    let converted = _convert_bits(decoded.iter().cloned(), 5, 8, true).unwrap();
    let mut addr = [0; 20];
    addr.copy_from_slice(&converted[1..converted.len() - 6]);
    Ok((
        addr,
        match converted[0] {
            0 => AddressType::P2PKH,
            8 => AddressType::P2SH,
            x => return Err(CashAddrError::InvalidAddressType(x)),
        },
        prefix,
    ))
}

#[cfg(test)]
mod tests {
    use super::{Address, AddressType, Hash160, Prefix, Result};

    #[test]
    fn test_from_hash1() -> Result<()> {
        let addr = Address::from_hash(
            Prefix::BitcoinCash,
            AddressType::P2PKH,
            Hash160::new([0; 20]),
        );
        assert_eq!(
            addr.cash_addr(),
            "bitcoincash:qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqfnhks603"
        );
        Ok(())
    }

    #[test]
    fn test_from_hash2() -> Result<()> {
        let addr = Address::from_hash(
            Prefix::SimpleLedger,
            AddressType::P2PKH,
            Hash160::new([0; 20]),
        );
        assert_eq!(
            addr.cash_addr(),
            "simpleledger:qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq9gud9630"
        );
        Ok(())
    }

    #[test]
    fn test_from_hash3() -> Result<()> {
        let addr = Address::from_hash(
            Prefix::BitcoinCash,
            AddressType::P2SH,
            Hash160::new([0; 20]),
        );
        assert_eq!(
            addr.cash_addr(),
            "bitcoincash:pqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq7k2ehe5v"
        );
        Ok(())
    }

    #[test]
    fn test_from_hash4() -> Result<()> {
        let addr = Address::from_hash("redridinghood", AddressType::P2SH, Hash160::new([0; 20]));
        assert_eq!(
            addr.cash_addr(),
            "redridinghood:pqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqxmg9w0gt"
        );
        Ok(())
    }

    #[test]
    fn test_from_cash_addr1() -> Result<()> {
        let addr =
            Address::from_cash_addr("bitcoincash:qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqfnhks603")?;
        assert_eq!(addr.addr_type(), AddressType::P2PKH);
        assert_eq!(
            addr.cash_addr(),
            "bitcoincash:qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqfnhks603"
        );
        assert_eq!(addr.hash(), &Hash160::new([0; 20]));
        assert_eq!(addr.prefix_kind(), Some(Prefix::BitcoinCash));
        assert_eq!(addr.prefix_str(), "bitcoincash");
        Ok(())
    }

    #[test]
    fn test_from_cash_addr2() -> Result<()> {
        let addr =
            Address::from_cash_addr("simpleledger:qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq9gud9630")?;
        assert_eq!(addr.addr_type(), AddressType::P2PKH);
        assert_eq!(
            addr.cash_addr(),
            "simpleledger:qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq9gud9630"
        );
        assert_eq!(addr.hash(), &Hash160::new([0; 20]));
        assert_eq!(addr.prefix_kind(), Some(Prefix::SimpleLedger));
        assert_eq!(addr.prefix_str(), "simpleledger");
        Ok(())
    }

    #[test]
    fn test_from_cash_addr3() -> Result<()> {
        let addr =
            Address::from_cash_addr("bitcoincash:pqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq7k2ehe5v")?;
        assert_eq!(addr.addr_type(), AddressType::P2SH);
        assert_eq!(
            addr.cash_addr(),
            "bitcoincash:pqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq7k2ehe5v"
        );
        assert_eq!(addr.hash(), &Hash160::new([0; 20]));
        assert_eq!(addr.prefix_kind(), Some(Prefix::BitcoinCash));
        assert_eq!(addr.prefix_str(), "bitcoincash");
        Ok(())
    }

    #[test]
    fn test_from_cash_addr4() -> Result<()> {
        let addr =
            Address::from_cash_addr("redridinghood:pqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqxmg9w0gt")?;
        assert_eq!(addr.addr_type(), AddressType::P2SH);
        assert_eq!(
            addr.cash_addr(),
            "redridinghood:pqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqxmg9w0gt"
        );
        assert_eq!(addr.hash(), &Hash160::new([0; 20]));
        assert_eq!(addr.prefix_kind(), None);
        assert_eq!(addr.prefix_str(), "redridinghood");
        Ok(())
    }

    #[test]
    fn test_with_prefix() -> Result<()> {
        let addr =
            Address::from_cash_addr("redridinghood:pqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqxmg9w0gt")?;
        let new_addr = addr.with_prefix("prelude");
        assert_eq!(new_addr.addr_type(), AddressType::P2SH);
        assert_eq!(
            new_addr.cash_addr(),
            "prelude:pqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqrs52h40n"
        );
        assert_eq!(new_addr.hash(), &Hash160::new([0; 20]));
        assert_eq!(new_addr.prefix_kind(), None);
        assert_eq!(new_addr.prefix_str(), "prelude");
        Ok(())
    }
}
