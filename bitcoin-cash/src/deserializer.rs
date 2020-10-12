use crate::{error::Result, BitcoinCode, ByteArray, ByteArrayError};

pub fn decode_bitcoin_code<T: BitcoinCode>(byte_array: impl Into<ByteArray>) -> Result<T> {
    let (item, leftover) = T::deser(byte_array.into())?;
    if !leftover.is_empty() {
        return Err(ByteArrayError::LeftoverBytes { bytes: leftover }.into());
    }
    return Ok(item);
}

#[cfg(test)]
mod tests {
    use crate::error::Result;
    use crate::{decode_bitcoin_code, BitcoinCode, ByteArray, Hashed, Sha256d};

    #[bitcoin_code(crate = "crate")]
    #[derive(BitcoinCode, PartialEq, Debug)]
    struct TxInput {
        pub prev_tx_hash: Sha256d,
        pub prev_vout: u32,
        pub script: Vec<u8>,
        pub sequence: u32,
    }

    #[bitcoin_code(crate = "crate")]
    #[derive(BitcoinCode, PartialEq, Debug)]
    struct TxOutput {
        pub value: u64,
        pub script: Vec<u8>,
    }

    #[bitcoin_code(crate = "crate")]
    #[derive(BitcoinCode, PartialEq, Debug)]
    struct Tx {
        pub version: u32,
        pub inputs: Vec<TxInput>,
        pub outputs: Vec<TxOutput>,
        pub locktime: u32,
    }

    #[test]
    fn test_struct() -> Result<()> {
        use hex_literal::hex;
        #[bitcoin_code(crate = "crate")]
        #[derive(BitcoinCode, PartialEq, Debug)]
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
        #[bitcoin_code(crate = "crate")]
        #[derive(PartialEq, Debug, BitcoinCode)]
        struct TestPack {
            a: u8,
            b: u16,
            c: u32,
            d: u64,
            e: i8,
            f: i16,
            g: i32,
            h: i64,
        }

        #[bitcoin_code(crate = "crate")]
        #[derive(PartialEq, Debug, BitcoinCode)]
        struct Test {
            int: u32,
            seq: Vec<ByteArray>,
            relay: bool,
            pack: TestPack,
        }

        let sample = Test {
            int: 1,
            seq: vec![b"\x77".into(), b"\x99".into()],
            relay: true,
            pack: TestPack {
                a: 1,
                b: 2,
                c: 3,
                d: 4,
                e: 5,
                f: 6,
                g: 7,
                h: 8,
            },
        };
        let sample_encoded = hex::decode(format!(
            "01000000\
             0201770199\
             01\
             010200030000000400000000000000\
             050600070000000800000000000000",
        ))?;
        assert_eq!(sample, decode_bitcoin_code(sample.ser())?);
        assert_eq!(sample, decode_bitcoin_code(sample_encoded.as_slice())?);
        assert_eq!(sample.ser().as_slice(), sample_encoded.as_slice());
        Ok(())
    }

    #[test]
    fn test_encode_lengths() -> Result<()> {
        #[bitcoin_code(crate = "crate")]
        #[derive(BitcoinCode, PartialEq, Debug)]
        struct Test {
            array: ByteArray,
        };
        fn make(vec: Vec<u8>) -> Test {
            Test { array: vec.into() }
        }
        let encoded = make(vec![0x77; 0xfc]).ser();
        assert_eq!(encoded[0], 0xfc);
        assert_eq!(&encoded[1..], &[0x77; 0xfc][..]);
        let encoded = make(vec![0x88; 0xfd]).ser();
        assert_eq!(&encoded[0..3], &[0xfd, 0xfd, 0x00][..]);
        assert_eq!(&encoded[3..], &[0x88; 0xfd][..]);
        let encoded = make(vec![0x99; 0x103]).ser();
        assert_eq!(&encoded[0..3], &[0xfd, 0x03, 0x01][..]);
        assert_eq!(&encoded[3..], &[0x99; 0x103][..]);
        let encoded = make(vec![0xaa; 0xffff]).ser();
        assert_eq!(&encoded[0..3], &[0xfd, 0xff, 0xff][..]);
        assert_eq!(&encoded[3..], &[0xaa; 0xffff][..]);
        let encoded = make(vec![0xbb; 0x10000]).ser();
        assert_eq!(&encoded[0..5], &[0xfe, 0x00, 0x00, 0x01, 0x00][..]);
        assert_eq!(&encoded[5..], &[0xbb; 0x10000][..]);
        let encoded = make(vec![0xbb; 0x123456]).ser();
        assert_eq!(&encoded[0..5], &[0xfe, 0x56, 0x34, 0x12, 0x00][..]);
        assert_eq!(&encoded[5..], &[0xbb; 0x123456][..]);
        Ok(())
    }

    #[test]
    fn test_decode_lengths() -> Result<()> {
        #[bitcoin_code(crate = "crate")]
        #[derive(BitcoinCode, PartialEq, Debug)]
        struct Test {
            array: ByteArray,
        };
        fn make(vec: Vec<u8>) -> Test {
            Test { array: vec.into() }
        }
        let t: Test = decode_bitcoin_code([&[0xfc][..], &vec![0x77; 0xfc][..]].concat())?;
        assert_eq!(t, make(vec![0x77; 0xfc]));
        let t: Test =
            decode_bitcoin_code([&[0xfd, 0xfd, 0x00][..], &vec![0x88; 0xfd][..]].concat())?;
        assert_eq!(t, make(vec![0x88; 0xfd]));
        let t: Test =
            decode_bitcoin_code([&[0xfd, 0x03, 0x01][..], &vec![0x99; 0x103][..]].concat())?;
        assert_eq!(t, make(vec![0x99; 0x103]));
        let t: Test =
            decode_bitcoin_code([&[0xfd, 0xff, 0xff][..], &vec![0xaa; 0xffff][..]].concat())?;
        assert_eq!(t, make(vec![0xaa; 0xffff]));
        let t: Test = decode_bitcoin_code(
            [
                &[0xfe, 0x00, 0x00, 0x01, 0x00][..],
                &vec![0xbb; 0x10000][..],
            ]
            .concat(),
        )?;
        assert_eq!(t, make(vec![0xbb; 0x10000]));
        let t: Test = decode_bitcoin_code(
            [
                &[0xfe, 0x56, 0x34, 0x12, 0x00][..],
                &vec![0xcc; 0x123456][..],
            ]
            .concat(),
        )?;
        assert_eq!(t, make(vec![0xcc; 0x123456]));
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
        let (tx, _) = Tx::deser(tx_raw.clone().into()).unwrap();
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
