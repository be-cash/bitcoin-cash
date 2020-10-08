use crate::encoding_utils::read_var_int;
use crate::error::{BitcoinCodeError, Error, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use serde::de::Visitor;
use std::io::{self, Read};

struct Input<'de> {
    input: &'de [u8],
    pos: usize,
}

struct Deserializer<'de> {
    input: Input<'de>,
}

struct Access<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    len: usize,
}

impl serde::de::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Error::Msg(msg.to_string())
    }
}

impl<'de> io::Read for Input<'de> {
    fn read(&mut self, buf: &mut [u8]) -> std::result::Result<usize, io::Error> {
        let n_bytes = io::Cursor::new(&self.input[self.pos..]).read(buf)?;
        self.pos += n_bytes;
        Ok(n_bytes)
    }
}

impl<'de> Input<'de> {
    fn read_slice(&mut self, n_bytes: usize) -> std::result::Result<&'de [u8], io::Error> {
        let remaining_slice = &self.input[self.pos..];
        if remaining_slice.len() < n_bytes {
            return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
        }
        self.pos += n_bytes;
        Ok(&remaining_slice[..n_bytes])
    }

    fn is_empty(&self) -> bool {
        self.input[self.pos..].len() == 0
    }
}

macro_rules! impl_nums {
    ($ty:ty, $dser_method:ident, $visitor_method:ident, $reader_method:ident) => {
        #[inline]
        fn $dser_method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
            let value = self.input.$reader_method::<LittleEndian>()?;
            visitor.$visitor_method(value)
        }
    };
}

impl<'de, 'a> serde::de::SeqAccess<'de> for Access<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        if self.len > 0 {
            self.len -= 1;
            let value = serde::de::DeserializeSeed::deserialize(seed, &mut *self.de)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.len)
    }
}

impl<'de, 'a> serde::de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        BitcoinCodeError::DeserializeAnyNotSupported.into_err()
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let value: u8 = serde::Deserialize::deserialize(self)?;
        match value {
            1 => visitor.visit_bool(true),
            0 => visitor.visit_bool(false),
            value => BitcoinCodeError::InvalidBoolEncoding(value).into_err(),
        }
    }

    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u8(self.input.read_u8()?)
    }

    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i8(self.input.read_i8()?)
    }

    impl_nums!(u16, deserialize_u16, visit_u16, read_u16);
    impl_nums!(u32, deserialize_u32, visit_u32, read_u32);
    impl_nums!(u64, deserialize_u64, visit_u64, read_u64);
    impl_nums!(i16, deserialize_i16, visit_i16, read_i16);
    impl_nums!(i32, deserialize_i32, visit_i32, read_i32);
    impl_nums!(i64, deserialize_i64, visit_i64, read_i64);

    fn deserialize_f32<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        BitcoinCodeError::DataTypeNotSupported("f32").into_err()
    }
    fn deserialize_f64<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        BitcoinCodeError::DataTypeNotSupported("f64").into_err()
    }
    fn deserialize_u128<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        BitcoinCodeError::DataTypeNotSupported("u128").into_err()
    }
    fn deserialize_i128<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        BitcoinCodeError::DataTypeNotSupported("i128").into_err()
    }
    fn deserialize_char<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        BitcoinCodeError::DataTypeNotSupported("char").into_err()
    }

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_unit()
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let len = read_var_int(&mut self.input)? as usize;
        let slice = self.input.read_slice(len)?;
        visitor.visit_borrowed_str(&std::str::from_utf8(slice)?)
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let len = read_var_int(&mut self.input)? as usize;
        let mut buf = vec![0; len];
        self.input.read_exact(&mut buf)?;
        visitor.visit_string(String::from_utf8(buf).map_err(|err| err.utf8_error())?)
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let len = read_var_int(&mut self.input)? as usize;
        visitor.visit_borrowed_bytes(self.input.read_slice(len)?)
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let len = read_var_int(&mut self.input)? as usize;
        let mut buf = vec![0; len];
        self.input.read_exact(&mut buf)?;
        visitor.visit_byte_buf(buf)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _enum: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value> {
        BitcoinCodeError::MethodNotSupported("deserialize_enum").into_err()
    }

    fn deserialize_tuple<V: Visitor<'de>>(self, len: usize, visitor: V) -> Result<V::Value> {
        visitor.visit_seq(Access { de: self, len })
    }

    fn deserialize_option<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        BitcoinCodeError::DataTypeNotSupported("Option<T>").into_err()
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let len = read_var_int(&mut self.input)? as usize;
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_map<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        BitcoinCodeError::MethodNotSupported("deserialize_map").into_err()
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        self.deserialize_tuple(fields.len(), visitor)
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        BitcoinCodeError::MethodNotSupported("deserialize_identifier").into_err()
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &str,
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_unit()
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value> {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value> {
        BitcoinCodeError::MethodNotSupported("deserialize_ignored_any").into_err()
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

pub fn decode_bitcoin_code<'a, T>(input: &'a [u8]) -> Result<T>
where
    T: serde::de::Deserialize<'a>,
{
    let mut deserializer = Deserializer {
        input: Input { input, pos: 0 },
    };
    let t = T::deserialize(&mut deserializer)?;
    if deserializer.input.is_empty() {
        Ok(t)
    } else {
        BitcoinCodeError::LeftoverBytes.into_err()
    }
}

#[cfg(test)]
mod tests {
    use super::decode_bitcoin_code;
    use crate::error::Result;
    use crate::{Hashed, SerializeExt, Sha256d};
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, PartialEq, Debug)]
    struct TxInput {
        pub prev_tx_hash: Sha256d,
        pub prev_vout: u32,
        pub script: Vec<u8>,
        pub sequence: u32,
    }

    #[derive(Deserialize, Serialize, PartialEq, Debug)]
    struct TxOutput {
        pub value: u64,
        pub script: Vec<u8>,
    }

    #[derive(Deserialize, Serialize, PartialEq, Debug)]
    struct Tx {
        pub version: u32,
        pub inputs: Vec<TxInput>,
        pub outputs: Vec<TxOutput>,
        pub locktime: u32,
    }

    #[test]
    fn test_struct() -> Result<()> {
        use hex_literal::hex;
        #[derive(Deserialize, PartialEq, Debug)]
        struct Test {
            int: u32,
            seq: Vec<Vec<u8>>,
        }

        let j = hex!("010000000201770199");
        let expected = Test {
            int: 1,
            seq: vec![b"\x77".to_vec(), b"\x99".to_vec()],
        };
        assert_eq!(expected, decode_bitcoin_code(&j)?);
        Ok(())
    }

    #[test]
    fn test_fields() -> Result<()> {
        #[derive(Deserialize, Serialize, PartialEq, Debug)]
        struct TestElement<'a> {
            name: String,
            name_ref: &'a str,
        }

        #[derive(Deserialize, Serialize, PartialEq, Debug)]
        struct TestPack(u8, u16, u32, u64, i8, i16, i32, i64);

        #[derive(Deserialize, Serialize, PartialEq, Debug)]
        struct TestUnit;

        #[derive(Deserialize, Serialize, PartialEq, Debug)]
        struct Test<'a> {
            int: u32,
            seq: Vec<Vec<u8>>,
            relay: bool,
            slice: &'a [u8],
            pack: TestPack,
            unit_struct: TestUnit,
            unit: (),
            vec: Vec<(TestElement<'a>, ())>,
        }

        let sample = Test {
            int: 1,
            seq: vec![b"\x77".to_vec(), b"\x99".to_vec()],
            relay: true,
            slice: b"whatever",
            pack: TestPack(1, 2, 3, 4, 5, 6, 7, 8),
            unit_struct: TestUnit,
            unit: (),
            vec: vec![
                (
                    TestElement {
                        name: "banana".to_string(),
                        name_ref: "teracotta",
                    },
                    (),
                ),
                (
                    TestElement {
                        name: "pie".to_string(),
                        name_ref: "what",
                    },
                    (),
                ),
            ],
        };
        let sample_encoded = hex::decode(format!(
            "01000000\
             0201770199\
             01\
             08{}\
             010200030000000400000000000000\
             050600070000000800000000000000\
             02\
             06{}09{}\
             03{}04{}",
            hex::encode(b"whatever"),
            hex::encode(b"banana"),
            hex::encode(b"teracotta"),
            hex::encode(b"pie"),
            hex::encode(b"what"),
        ))?;
        assert_eq!(sample, decode_bitcoin_code(&sample.ser())?);
        assert_eq!(sample, decode_bitcoin_code(&sample_encoded)?);
        assert_eq!(sample.ser().as_slice(), sample_encoded.as_slice());
        Ok(())
    }

    #[test]
    fn test_encode_lengths() -> Result<()> {
        #[derive(Deserialize, Serialize, PartialEq, Debug)]
        struct Test(Vec<u8>);
        let encoded = Test(vec![0x77; 0xfc]).ser();
        assert_eq!(encoded[0], 0xfc);
        assert_eq!(&encoded[1..], &[0x77; 0xfc][..]);
        let encoded = Test(vec![0x88; 0xfd]).ser();
        assert_eq!(&encoded[0..3], &[0xfd, 0xfd, 0x00][..]);
        assert_eq!(&encoded[3..], &[0x88; 0xfd][..]);
        let encoded = Test(vec![0x99; 0x103]).ser();
        assert_eq!(&encoded[0..3], &[0xfd, 0x03, 0x01][..]);
        assert_eq!(&encoded[3..], &[0x99; 0x103][..]);
        let encoded = Test(vec![0xaa; 0xffff]).ser();
        assert_eq!(&encoded[0..3], &[0xfd, 0xff, 0xff][..]);
        assert_eq!(&encoded[3..], &[0xaa; 0xffff][..]);
        let encoded = Test(vec![0xbb; 0x10000]).ser();
        assert_eq!(&encoded[0..5], &[0xfe, 0x00, 0x00, 0x01, 0x00][..]);
        assert_eq!(&encoded[5..], &[0xbb; 0x10000][..]);
        let encoded = Test(vec![0xbb; 0x123456]).ser();
        assert_eq!(&encoded[0..5], &[0xfe, 0x56, 0x34, 0x12, 0x00][..]);
        assert_eq!(&encoded[5..], &[0xbb; 0x123456][..]);
        Ok(())
    }

    #[test]
    fn test_decode_lengths() -> Result<()> {
        #[derive(Deserialize, Serialize, PartialEq, Debug)]
        struct Test(Vec<u8>);
        let t: Test = decode_bitcoin_code(&[&[0xfc][..], &vec![0x77; 0xfc][..]].concat())?;
        assert_eq!(t, Test(vec![0x77; 0xfc]));
        let t: Test =
            decode_bitcoin_code(&[&[0xfd, 0xfd, 0x00][..], &vec![0x88; 0xfd][..]].concat())?;
        assert_eq!(t, Test(vec![0x88; 0xfd]));
        let t: Test =
            decode_bitcoin_code(&[&[0xfd, 0x03, 0x01][..], &vec![0x99; 0x103][..]].concat())?;
        assert_eq!(t, Test(vec![0x99; 0x103]));
        let t: Test =
            decode_bitcoin_code(&[&[0xfd, 0xff, 0xff][..], &vec![0xaa; 0xffff][..]].concat())?;
        assert_eq!(t, Test(vec![0xaa; 0xffff]));
        let t: Test = decode_bitcoin_code(
            &[
                &[0xfe, 0x00, 0x00, 0x01, 0x00][..],
                &vec![0xbb; 0x10000][..],
            ]
            .concat(),
        )?;
        assert_eq!(t, Test(vec![0xbb; 0x10000]));
        let t: Test = decode_bitcoin_code(
            &[
                &[0xfe, 0x56, 0x34, 0x12, 0x00][..],
                &vec![0xcc; 0x123456][..],
            ]
            .concat(),
        )?;
        assert_eq!(t, Test(vec![0xcc; 0x123456]));
        Ok(())
    }

    #[test]
    fn test_tx() -> Result<()> {
        let tx_raw = hex::decode(
            "0100000002b1c5c527d23f2f559ccac3748568806e617b38d76894b1e36c5e795e10ebe29400000000fc0047\
            304402207a8ed9b57865ce56935b60794526c9c48833f752394ba920faddb08a14cbd02502200879a7fe141b\
            dab57d28fc778f85fa0dc5df1f6af2bcd1d793387e2f4ef7492a414730440220623322db152ba053ed861fcd\
            148d5fc0cc49158019911cf2a8411693e0bb95c602201f04cf4f81ec37e32aa030b89b359e3c49491014bac2\
            d70a201e14932647ef11414c6952210257be20743c0bc14d33e6c0fe5b887b6cf47883b8924282a6948db577\
            4502acd4210280b4d5ca10b008b757999dae9bdad11a2c856490c3582bcdc9ff8ca458529bd12103ec98d577\
            ea245b65ca8c77f94463920ca53356b99b5ce49691c65e26f5b5683153aeffffffff3cbac2af90aa4e622214\
            8238ef3d5e268fa577ff01ecc67b8b534039c7403b8206000000fdfd000047304402207ac1f3a87aeef786e1\
            550b7832ca355da2195cff83bb5546569511920b2a2b5c02207d28e7b02d57f59bfad26c681071a88b4e3903\
            b07243735a9be5149bb28be2ed41483045022100a8a8af3a437c0dfa9c5bab36230d9e72727c972859a6facc\
            c76b506a4ae3294702206b924799bdc4880a707ec4b15e76e2503e2693dc8c1694fde237b60ad71cb637414c\
            69522102a9c79875e2de1a769831dba1a9cf20b7bddc48fbcbdb9c5eeab67d7fb682a01c21031187abcee948\
            d3c93c065fe5e560f7ae9cb7735b4e70507cebdf18ac7860a793210314d67175239913da79a0c905e086ad84\
            2a0470fcb774929eec0c9cc1222a6ef153aeffffffff01a0d90800000000001976a914f450f83dd8d1b09326\
            ae64857c3e9dfaa8a34ee688ac00000000"
        ).unwrap();
        assert_eq!(
            Sha256d::digest(tx_raw.as_slice()),
            Sha256d::from_hex_le(
                "fff9979f9c7afb3cbe7fe34083e6dd206e33b19df176772feefd55d71667bae1"
            )?,
        );
        let tx = decode_bitcoin_code::<Tx>(&tx_raw).unwrap();
        assert_eq!(tx.version, 1);
        assert_eq!(tx.locktime, 0);
        let tx_in0 = &tx.inputs[0];
        let tx_in1 = &tx.inputs[1];
        assert_eq!(tx.inputs.len(), 2);
        assert_eq!(
            tx_in0.prev_tx_hash,
            Sha256d::from_hex_le(
                "94e2eb105e795e6ce3b19468d7387b616e80688574c3ca9c552f3fd227c5c5b1"
            )?
        );
        assert_eq!(tx_in0.prev_vout, 0);
        assert_eq!(
            hex::encode(&tx_in0.script),
            "0047304402207a8ed9b57865ce56935b60794526c9c48833f752394ba920faddb08a14cbd02502200879a\
             7fe141bdab57d28fc778f85fa0dc5df1f6af2bcd1d793387e2f4ef7492a414730440220623322db152ba05\
             3ed861fcd148d5fc0cc49158019911cf2a8411693e0bb95c602201f04cf4f81ec37e32aa030b89b359e3c4\
             9491014bac2d70a201e14932647ef11414c6952210257be20743c0bc14d33e6c0fe5b887b6cf47883b8924\
             282a6948db5774502acd4210280b4d5ca10b008b757999dae9bdad11a2c856490c3582bcdc9ff8ca458529\
             bd12103ec98d577ea245b65ca8c77f94463920ca53356b99b5ce49691c65e26f5b5683153ae",
        );
        assert_eq!(tx_in0.sequence, 0xffff_ffff);

        assert_eq!(
            tx_in1.prev_tx_hash,
            Sha256d::from_hex_le(
                "823b40c73940538b7bc6ec01ff77a58f265e3def38821422624eaa90afc2ba3c"
            )?
        );
        assert_eq!(tx_in1.prev_vout, 6);
        assert_eq!(
            hex::encode(&tx_in1.script),
            "0047304402207ac1f3a87aeef786e1550b7832ca355da2195cff83bb5546569511920b2a2b5c02207d28e\
             7b02d57f59bfad26c681071a88b4e3903b07243735a9be5149bb28be2ed41483045022100a8a8af3a437c0\
             dfa9c5bab36230d9e72727c972859a6faccc76b506a4ae3294702206b924799bdc4880a707ec4b15e76e25\
             03e2693dc8c1694fde237b60ad71cb637414c69522102a9c79875e2de1a769831dba1a9cf20b7bddc48fbc\
             bdb9c5eeab67d7fb682a01c21031187abcee948d3c93c065fe5e560f7ae9cb7735b4e70507cebdf18ac786\
             0a793210314d67175239913da79a0c905e086ad842a0470fcb774929eec0c9cc1222a6ef153ae",
        );
        assert_eq!(tx_in1.sequence, 0xffff_ffff);

        let tx_out0 = &tx.outputs[0];
        assert_eq!(tx.outputs.len(), 1);
        assert_eq!(tx_out0.value, 5_800_00);
        assert_eq!(
            hex::encode(&tx_out0.script),
            "76a914f450f83dd8d1b09326ae64857c3e9dfaa8a34ee688ac",
        );

        assert_eq!(tx.ser().as_slice(), tx_raw.as_slice());

        Ok(())
    }
}
