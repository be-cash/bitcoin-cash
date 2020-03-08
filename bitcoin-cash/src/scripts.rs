use crate::{
    ops::{OpcodeType::*, Ops},
    Address, AddressType, ByteArray, Hashed, InputScriptBuilder, Pubkey, Script, SigHashFlags,
    TxBuilder, TxOutput, TxPreimage,
};

pub struct P2PKHBuilder<'b> {
    pub pubkey: &'b Pubkey,
    pub sig_hash_flags: SigHashFlags,
}

#[bitcoin_cash_script_macro::script(P2PKHInputs)]
pub fn p2pkh_script(address: &Address, sig: ByteArray<'static>, pubkey: Vec<u8>) {
    OP_DUP(pubkey);
    let pk_hashed = OP_HASH160(pubkey);
    let pk_hash = address.hash().as_slice();
    OP_EQUALVERIFY(pk_hashed, pk_hash);
    let success = OP_CHECKSIG(sig, pubkey);
}

#[bitcoin_cash_script_macro::script(P2SHInputs)]
pub fn p2sh_script(address: &Address, redeem_script: Vec<u8>) {
    let script_hashed = OP_HASH160(redeem_script);
    let script_hash = address.hash().as_slice();
    let success = OP_EQUAL(script_hashed, script_hash);
}

impl Into<Script<'static>> for &'_ Address<'_> {
    fn into(self) -> Script<'static> {
        match self.addr_type() {
            AddressType::P2SH => Script::minimal(p2sh_script(self).ops().into_owned().into()),
            AddressType::P2PKH => Script::minimal(p2pkh_script(self).ops().into_owned().into()),
        }
    }
}

impl Into<Script<'static>> for Address<'_> {
    fn into(self) -> Script<'static> {
        match self.addr_type() {
            AddressType::P2SH => Script::minimal(p2sh_script(&self).ops().into_owned().into()),
            AddressType::P2PKH => Script::minimal(p2pkh_script(&self).ops().into_owned().into()),
        }
    }
}

impl<'b> InputScriptBuilder for P2PKHBuilder<'b> {
    type Script = P2PKHInputs;
    fn sig_hash_flags(&self) -> Vec<SigHashFlags> {
        vec![self.sig_hash_flags]
    }
    fn build_script(
        &self,
        _tx_preimage: &[TxPreimage],
        _unsigned_tx: &TxBuilder,
        mut sigs: Vec<ByteArray<'static>>,
        _lock_script: &Script,
        _tx_outputs: &[TxOutput],
    ) -> Self::Script {
        P2PKHInputs {
            pubkey: self.pubkey.as_slice().to_vec(),
            sig: sigs.remove(0),
        }
    }
    fn is_p2sh(&self) -> bool {
        false
    }
}
