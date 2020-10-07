use bitcoin_cash::{error, ByteArray, Hashed, Op, Opcode, Script, Sha256d, TaggedOp, TxOutput};

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

pub fn slp_amount_ops<'a>(output_amounts: impl IntoIterator<Item = &'a u64>) -> Vec<Op> {
    output_amounts
        .into_iter()
        .enumerate()
        .map(|(idx, &output_amount)| Op::PushByteArray {
            array: ByteArray::new(
                format!("token_output_quantity{}", idx + 1),
                output_amount.to_be_bytes().to_vec(),
            ),
            is_minimal: false,
        })
        .collect()
}

pub fn slp_genesis_output(
    slp_token_type: SlpTokenType,
    token_ticker: &str,
    token_name: &str,
    token_document_url: &str,
    token_document_hash: &str,
    decimals: u8,
    mint_baton_vout: Option<u8>,
    initial_token_mint_quantity: u64,
) -> TxOutput {
    let byte_arrays = vec![
        ByteArray::from_slice("lokad_id", b"SLP\0"),
        ByteArray::new("token_type", vec![slp_token_type as u8]),
        ByteArray::new(
            "transaction_type",
            SlpTxType::GENESIS.to_string().into_bytes(),
        ),
        ByteArray::new("token_ticker", token_ticker.as_bytes().to_vec()),
        ByteArray::new("token_name", token_name.as_bytes().to_vec()),
        ByteArray::new("token_document_url", token_document_url.as_bytes().to_vec()),
        ByteArray::new(
            "token_document_hash",
            token_document_hash.as_bytes().to_vec(),
        ),
        ByteArray::new("decimals", decimals.to_be_bytes().as_ref()),
        ByteArray::new(
            "mint_baton_vout",
            mint_baton_vout
                .map(|vout| vout.to_be_bytes().to_vec())
                .unwrap_or(vec![]),
        ),
        ByteArray::new(
            "initial_token_mint_quantity",
            initial_token_mint_quantity.to_be_bytes().as_ref(),
        ),
    ];
    let mut ops = vec![Op::Code(Opcode::OP_RETURN)];
    for byte_array in byte_arrays {
        ops.push(Op::PushByteArray {
            array: byte_array,
            is_minimal: false,
        });
    }
    TxOutput {
        value: 0,
        script: Script::new(ops.into_iter().map(TaggedOp::from_op).collect::<Vec<_>>()),
    }
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
            array: ByteArray::new("transaction_type", SlpTxType::SEND.to_string().into_bytes()),
            is_minimal: false,
        },
        Op::PushByteArray {
            array: ByteArray::new("token_id", token_id.to_vec()),
            is_minimal: false,
        },
    ];
    ops.extend(slp_amount_ops(output_amounts.iter()));
    TxOutput {
        value: 0,
        script: Script::new(ops.into_iter().map(TaggedOp::from_op).collect::<Vec<_>>()),
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

    pub fn as_slice_be(&self) -> &[u8] {
        self.0.as_slice()
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec_le()
    }
}
