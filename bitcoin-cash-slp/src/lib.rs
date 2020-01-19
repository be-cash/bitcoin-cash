use bitcoin_cash::{
    error,
    ops::{Op, OpcodeType},
    ByteArray, Hashed, Script, Sha256d, TxOutput,
};

#[derive(Copy, Clone, Debug, Hash)]
pub struct TokenId(Sha256d);

#[derive(Copy, Clone, Debug, Hash)]
pub enum SlpTokenType {
    Fungible = 1,
}

#[derive(Copy, Clone, Debug, Hash)]
pub enum SlpTxType {
    GENESIS,
    SEND,
    MINT,
    COMMIT,
}

pub fn slp_send_output(
    slp_token_type: SlpTokenType,
    token_id: &TokenId,
    output_amounts: &[u64],
) -> TxOutput<'static> {
    let mut ops = vec![
        Op::Code(OpcodeType::OP_RETURN),
        Op::PushByteArray(ByteArray::from_slice(b"SLP\0")),
        Op::PushByteArray(vec![slp_token_type as u8].into()),
        Op::PushByteArray(SlpTxType::SEND.to_string().into_bytes().into()),
        Op::PushByteArray(token_id.to_vec().into()),
    ];
    for &output_amount in output_amounts {
        ops.push(Op::PushByteArray(
            output_amount.to_be_bytes().to_vec().into(),
        ));
    }
    TxOutput {
        value: 0,
        script: Script::new(ops.into()),
    }
}

impl std::fmt::Display for SlpTxType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}

impl TokenId {
    pub fn from_slice(token_id: &[u8]) -> error::Result<Self> {
        Ok(TokenId(Sha256d::from_slice_le(token_id)?))
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec_le()
    }
}
