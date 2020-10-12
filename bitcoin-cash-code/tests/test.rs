use bitcoin_cash::{BitcoinCode, ByteArray, Script, Sha256d};

#[test]
fn test_tx() {
    #[bitcoin_code]
    #[derive(BitcoinCode)]
    pub struct TxOutpoint {
        pub tx_hash: Sha256d,
        pub vout: u32,
    }

    #[derive(BitcoinCode)]
    pub struct TxInput {
        pub prev_out: TxOutpoint,
        pub script: Script,
        pub sequence: u32,
    }

    #[derive(BitcoinCode)]
    pub struct TxOutput {
        pub value: u64,
        pub script: Script,
    }

    #[derive(BitcoinCode)]
    pub struct UnhashedTx {
        pub version: i32,
        pub inputs: Vec<TxInput>,
        pub outputs: Vec<TxOutput>,
        pub lock_time: u32,
    }

    #[derive(BitcoinCode)]
    pub struct Tx {
        raw: ByteArray,
    }
}
