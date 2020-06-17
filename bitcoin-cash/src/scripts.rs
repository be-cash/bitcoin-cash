use crate::{
    Address, AddressType, ByteArray, Opcode::*, Pubkey, Script, SigHashFlags, Signatory, TxBuilder,
    TxOutput, TxPreimage, MAX_SIGNATURE_SIZE,
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
    let pk_hash = address.hash();
    OP_EQUALVERIFY(pk_hashed, pk_hash);
    let success = OP_CHECKSIG(sig, pubkey);
}

#[crate::script(P2SHInputs, crate = "crate")]
pub fn p2sh_script(address: &Address, redeem_script: ByteArray) {
    let script_hashed = OP_HASH160(redeem_script);
    let script_hash = address.hash();
    let success = OP_EQUAL(script_hashed, script_hash);
}

impl Into<Script> for &'_ Address<'_> {
    fn into(self) -> Script {
        match self.addr_type() {
            AddressType::P2SH => p2sh_script(self).into(),
            AddressType::P2PKH => p2pkh_script(self).into(),
        }
    }
}

impl Into<Script> for Address<'_> {
    fn into(self) -> Script {
        match self.addr_type() {
            AddressType::P2SH => p2sh_script(&self).into(),
            AddressType::P2PKH => p2pkh_script(&self).into(),
        }
    }
}

impl<'b> Signatory for P2PKHBuilder<'b> {
    type Script = P2PKHInputs;
    type Signatures = ByteArray;
    fn sig_hash_flags(&self) -> Vec<SigHashFlags> {
        vec![self.sig_hash_flags]
    }
    fn placeholder_signatures(&self) -> Self::Signatures {
        ByteArray::new_unnamed(vec![0; MAX_SIGNATURE_SIZE])
    }
    fn build_script(
        &self,
        _tx_preimage: &[TxPreimage],
        _unsigned_tx: &TxBuilder,
        sigs: Self::Signatures,
        _lock_script: &Script,
        _tx_outputs: &[TxOutput],
    ) -> Self::Script {
        P2PKHInputs {
            pubkey: self.pubkey.as_slice().into(),
            sig: sigs.concat(ByteArray::new(
                "sig_hash",
                [self.sig_hash_flags.bits() as u8].as_ref(),
            )),
        }
    }
    fn is_p2sh(&self) -> bool {
        false
    }
}
