use crate::{
    Address, AddressType, ByteArray, Hashed, InputScriptBuilder, Opcode::*, Ops, Pubkey, Script,
    SigHashFlags, TxBuilder, TxOutput, TxPreimage,
};

#[derive(Clone)]
pub struct P2PKHBuilder<'b> {
    pub pubkey: &'b Pubkey,
    pub sig_hash_flags: SigHashFlags,
}

#[crate::script(P2PKHInputs, crate = "crate")]
pub fn p2pkh_script(address: &Address, sig: ByteArray, pubkey: ByteArray) {
    OP_DUP(pubkey);
    let pk_hashed = OP_HASH160(pubkey);
    let pk_hash = address.hash().as_byte_array();
    OP_EQUALVERIFY(pk_hashed, pk_hash);
    let success = OP_CHECKSIG(sig, pubkey);
}

#[crate::script(P2SHInputs, crate = "crate")]
pub fn p2sh_script(address: &Address, redeem_script: ByteArray) {
    let script_hashed = OP_HASH160(redeem_script);
    let script_hash = address.hash().as_byte_array();
    let success = OP_EQUAL(script_hashed, script_hash);
}

impl Into<Script> for &'_ Address<'_> {
    fn into(self) -> Script {
        match self.addr_type() {
            AddressType::P2SH => Script::new(p2sh_script(self).ops().into_owned().into()),
            AddressType::P2PKH => Script::new(p2pkh_script(self).ops().into_owned().into()),
        }
    }
}

impl Into<Script> for Address<'_> {
    fn into(self) -> Script {
        match self.addr_type() {
            AddressType::P2SH => Script::new(p2sh_script(&self).ops().into_owned().into()),
            AddressType::P2PKH => Script::new(p2pkh_script(&self).ops().into_owned().into()),
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
        mut sigs: Vec<ByteArray>,
        _lock_script: &Script,
        _tx_outputs: &[TxOutput],
    ) -> Self::Script {
        P2PKHInputs {
            pubkey: self.pubkey.as_slice().into(),
            sig: sigs.remove(0).concat(ByteArray::new(
                "sig_hash",
                [self.sig_hash_flags.bits() as u8].as_ref(),
            )),
        }
    }
    fn is_p2sh(&self) -> bool {
        false
    }
}
