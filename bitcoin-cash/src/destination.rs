use bitcoin_cash_base::{Opcode, PatternOp};

use crate::{Address, AddressType, Hash160, Hashed, Op, Pubkey, Script, SigHashFlags};

#[derive(Clone, Debug)]
pub enum Destination<'a> {
    Nulldata(Vec<Op>),
    Address(Address<'a>),
    P2PK(Vec<u8>),
    Unknown(Script),
}

pub fn script_destination<'a>(addr_prefix: &'a str, script: &Script) -> Destination<'a> {
    const OP_DUP: PatternOp = PatternOp::Code(Opcode::OP_DUP);
    const OP_HASH160: PatternOp = PatternOp::Code(Opcode::OP_HASH160);
    const OP_EQUALVERIFY: PatternOp = PatternOp::Code(Opcode::OP_EQUALVERIFY);
    const OP_CHECKSIG: PatternOp = PatternOp::Code(Opcode::OP_CHECKSIG);
    const OP_EQUAL: PatternOp = PatternOp::Code(Opcode::OP_EQUAL);
    const OP_RETURN: PatternOp = PatternOp::Code(Opcode::OP_RETURN);
    let ops = script.ops_arc();
    let pattern_ops: Vec<PatternOp> = script.ops_arc().iter().map(|op| (&op.op).into()).collect::<Vec<_>>();

    use PatternOp::Array;
    match pattern_ops.as_slice() {
        [OP_DUP, OP_HASH160, Array(hash), OP_EQUALVERIFY, OP_CHECKSIG] => {
            Destination::Address(
                Address::from_hash(
                    addr_prefix,
                    AddressType::P2PKH,
                    Hash160::from_slice(hash).expect("Invalid hash"),
                ),
            )
        }
        [OP_HASH160, Array(hash), OP_EQUAL] => {
            Destination::Address(
                Address::from_hash(
                    addr_prefix,
                    AddressType::P2SH,
                    Hash160::from_slice(hash).expect("Invalid hash"),
                )
            )
        }
        [Array(pk), OP_CHECKSIG] => Destination::P2PK(pk.to_vec()),
        [OP_RETURN, ..] => {
            Destination::Nulldata(ops.iter().skip(1).map(|op| op.op.clone()).collect())
        }
        _ => Destination::Unknown(script.clone()),
    }
}

impl Destination<'_> {
    pub fn address(&self) -> Option<&Address> {
        match self {
            Destination::Address(address) => Some(address),
            _ => None,
        }
    }
}

pub struct P2PKHSpend {
    pub pubkey: Pubkey,
    pub sig_hash_flags: SigHashFlags,
}

pub fn script_p2pkh_spend(script: &Script) -> Option<P2PKHSpend> {
    let pattern_ops: Vec<PatternOp> = script.ops_arc().iter().map(|op| (&op.op).into()).collect::<Vec<_>>();
    use PatternOp::Array;
    match pattern_ops.as_slice() {
        [Array(sig), Array(pubkey)] if pubkey.len() == 33 => {
            let sig_hash_flags_byte = *sig.last()?;
            let sig_hash_flags = SigHashFlags::from_bits(sig_hash_flags_byte as u32)?;
            Some(P2PKHSpend {
                pubkey: Pubkey::from_slice_checked(pubkey)?,
                sig_hash_flags,
            })
        }
        _ => None,
    }
}
