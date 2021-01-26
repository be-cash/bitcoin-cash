use bitcoin_cash::{ByteArray, Hashed, Op, Opcode, Script, Sha256d, TaggedOp, TxInput, TxOutput, UnhashedTx, UnsignedTxInput, error};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct TokenId(Sha256d);

#[derive(Deserialize, Serialize, Copy, Clone, Debug, Hash)]
pub enum SlpTokenType {
    Fungible = 1,
    Nft1Child = 0x41,
    Nft1Group = 0x81,
}

#[derive(Copy, Clone, Debug, Hash)]
pub enum SlpTxType {
    GENESIS,
    SEND,
    MINT,
    COMMIT,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SlpTx {
    pub version: i32,
    pub inputs: Vec<SlpTxInput>,
    pub outputs: Vec<SlpTxOutput>,
    pub lock_time: u32,
    pub slp_data: Option<SlpData>,
    pub tx_hash: Sha256d,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SlpData {
    pub token_id: TokenId,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SlpTxInput {
    pub index: usize,
    pub token: SlpToken,
    pub input: TxInput,
    pub prev_script: Script,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SlpTxOutput {
    pub token: SlpToken,
    pub output: TxOutput,
}

#[derive(Deserialize, Serialize, Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SlpToken {
    pub amount: u64,
    pub is_mint_baton: bool,
    pub action: SlpAction,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SlpAction {
    NonSlp,
    NonSlpBurn,
    SlpParseError,
    SlpUnsupportedVersion,
    SlpV1Genesis,
    SlpV1Mint,
    SlpV1Send,
    SlpNft1GroupGenesis,
    SlpNft1GroupMint,
    SlpNft1GroupSend,
    SlpNft1UniqueChildGenesis,
    SlpNft1UniqueChildSend,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SlpUtxo {
    pub input: UnsignedTxInput,
    pub slp_token: SlpToken,
    pub slp_data: Option<SlpData>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TokenMeta {
    pub ticker: String,
    pub name: String,
    pub document_url: String,
    pub document_hash: Vec<u8>,
    pub decimals: u8,
    pub slp_token_type: SlpTokenType,
}

pub struct SlpGenesisParams<'a> {
    pub slp_token_type: SlpTokenType,
    pub token_ticker: &'a str,
    pub token_name: &'a str,
    pub token_document_url: &'a str,
    pub token_document_hash: &'a str,
    pub decimals: u8,
    pub mint_baton_vout: Option<u8>,
    pub initial_token_mint_quantity: u64,
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

pub fn slp_genesis_output(params: SlpGenesisParams<'_>) -> TxOutput {
    let byte_arrays = vec![
        ByteArray::from_slice("lokad_id", b"SLP\0"),
        ByteArray::new("token_type", vec![params.slp_token_type as u8]),
        ByteArray::new(
            "transaction_type",
            SlpTxType::GENESIS.to_string().into_bytes(),
        ),
        ByteArray::new("token_ticker", params.token_ticker.as_bytes().to_vec()),
        ByteArray::new("token_name", params.token_name.as_bytes().to_vec()),
        ByteArray::new(
            "token_document_url",
            params.token_document_url.as_bytes().to_vec(),
        ),
        ByteArray::new(
            "token_document_hash",
            params.token_document_hash.as_bytes().to_vec(),
        ),
        ByteArray::new("decimals", params.decimals.to_be_bytes().as_ref()),
        ByteArray::new(
            "mint_baton_vout",
            params
                .mint_baton_vout
                .map(|vout| vout.to_be_bytes().to_vec())
                .unwrap_or_else(Vec::new),
        ),
        ByteArray::new(
            "initial_token_mint_quantity",
            params.initial_token_mint_quantity.to_be_bytes().as_ref(),
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
    #[deprecated]
    pub fn from_slice(token_id: &[u8]) -> error::Result<Self> {
        Ok(TokenId(Sha256d::from_slice_le(token_id)?))
    }

    pub fn from_hash(hash: Sha256d) -> Self {
        TokenId(hash)
    }

    pub fn as_slice_be(&self) -> &[u8] {
        self.0.as_slice()
    }

    pub fn hash(&self) -> &Sha256d {
        &self.0
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec_le()
    }
}

impl Default for SlpAction {
    fn default() -> Self {
        SlpAction::NonSlp
    }
}

impl SlpTx {
    pub fn into_unhashed_tx(self) -> UnhashedTx {
        UnhashedTx {
            version: self.version,
            inputs: self.inputs.into_iter().map(|input| input.input).collect(),
            outputs: self.outputs.into_iter().map(|output| output.output).collect(),
            lock_time: self.lock_time,
        }
    }
}
