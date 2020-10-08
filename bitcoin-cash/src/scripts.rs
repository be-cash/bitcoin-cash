use crate::{
    error::{Error, Result},
    Address, AddressType, ByteArray,
    Opcode::*,
    Pubkey, Script, SigHashFlags, Signatory, SignatoryKindOne, TaggedScript, TxOutput, TxPreimage,
    MAX_SIGNATURE_SIZE,
};

#[derive(Clone)]
pub struct P2PKHSignatory {
    pub pubkey: Pubkey,
    pub sig_hash_flags: SigHashFlags,
}

struct ParamsAddress<'a>(&'a Address<'a>);

#[crate::script(P2PKHInputs, crate = "crate")]
pub fn p2pkh_script(params: ParamsAddress<'_>, sig: ByteArray, pubkey: ByteArray) {
    OP_DUP(pubkey);
    let pk_hashed = OP_HASH160(pubkey);
    let pk_hash = params.0.hash();
    OP_EQUALVERIFY(pk_hashed, pk_hash);
    let success = OP_CHECKSIG(sig, pubkey);
}

#[crate::script(P2SHInputs, crate = "crate")]
pub fn p2sh_script(params: ParamsAddress<'_>, redeem_script: ByteArray) {
    let script_hashed = OP_HASH160(redeem_script);
    let script_hash = params.0.hash();
    let success = OP_EQUAL(script_hashed, script_hash);
}

impl Into<Script> for &'_ Address<'_> {
    fn into(self) -> Script {
        match self.addr_type() {
            AddressType::P2SH => ParamsAddress(self).p2sh_script().into(),
            AddressType::P2PKH => ParamsAddress(self).p2pkh_script().into(),
        }
    }
}

impl Into<Script> for Address<'_> {
    fn into(self) -> Script {
        match self.addr_type() {
            AddressType::P2SH => ParamsAddress(&self).p2sh_script().into(),
            AddressType::P2PKH => ParamsAddress(&self).p2pkh_script().into(),
        }
    }
}

impl Address<'_> {
    pub fn p2pkh_script(&self) -> Result<TaggedScript<P2PKHInputs>> {
        if self.addr_type() != AddressType::P2PKH {
            return Err(Error::InvalidAddressType);
        }
        Ok(ParamsAddress(self).p2pkh_script())
    }
}

impl<'b> Signatory for P2PKHSignatory {
    type Script = P2PKHInputs;
    type Signatures = ByteArray;
    type Kind = SignatoryKindOne;
    fn sig_hash_flags(&self) -> SigHashFlags {
        self.sig_hash_flags
    }
    fn placeholder_signatures(&self) -> Self::Signatures {
        ByteArray::new_unnamed(vec![0; MAX_SIGNATURE_SIZE])
    }
    fn build_script(
        &self,
        _tx_preimage: &TxPreimage,
        _estimazed_size: Option<usize>,
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
