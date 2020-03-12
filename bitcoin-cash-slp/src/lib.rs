use bitcoin_cash::{error, ByteArray, Hashed, Op, Opcode, Script, Sha256d, TxOutput};

#[derive(Clone, Debug, Hash)]
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
) -> TxOutput {
    let mut ops = vec![
        Op::Code(Opcode::OP_RETURN),
        Op::PushByteArray {
            array: ByteArray::from_slice("lokad_id", b"SLP\0"),
            is_minimal: false,
        },
        Op::PushByteArray {
            array: ByteArray::new("token_type", vec![slp_token_type as u8]),
            is_minimal: false,
        },
        Op::PushByteArray {
            array: ByteArray::new(
                "transaction_type",
                SlpTxType::SEND.to_string().into_bytes(),
            ),
            is_minimal: false,
        },
        Op::PushByteArray {
            array: ByteArray::new("token_id", token_id.to_vec()),
            is_minimal: false,
        },
    ];
    for (idx, &output_amount) in output_amounts.iter().enumerate() {
        ops.push(Op::PushByteArray {
            array: ByteArray::new(
                format!("token_output_quantity{}", idx + 1),
                output_amount.to_be_bytes().to_vec(),
            ),
            is_minimal: false,
        });
    }
    TxOutput {
        value: 0,
        script: Script::new(ops),
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

    pub fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec_le()
    }
}
