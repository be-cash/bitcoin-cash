#![deny(missing_docs)]
#![deny(missing_doc_code_examples)]

//! Base crate used in the bitcoin-cash crate.

extern crate proc_macro;

mod gen_source;
mod generate;
mod ir;
mod parse;
mod state;

use quote::quote;

/// Write complex Bitcoin Cash scripts using this macro:
/// ```
/// use bitcoin_cash::{Opcode::*, Address, ByteArray, Hashed, BitcoinCode};
/// struct Params {
///   address: Address<'static>,
/// }
/// #[bitcoin_cash::script(P2PKHInputs)]
/// fn p2pkh_script(params: Params, signature: ByteArray, public_key: ByteArray) {
///   OP_DUP(public_key);
///   let pkh = OP_HASH160(public_key);
///   let address = { params.address.hash().as_slice() };
///   OP_EQUALVERIFY(pkh, address);
///   OP_CHECKSIG(signature, public_key);
/// }
/// let serialized = Params {
///   address: Address::from_cash_addr("bitcoincash:qzt646a0weknq639ck5aq39afcq2n3c0xslfzmdyej")
///     .unwrap()
/// }
/// .p2pkh_script().script().ser();
///
/// assert_eq!(hex::encode(serialized), "1976a91497aaebaf766d306a25c5a9d044bd4e00a9c70f3488ac");
/// ```
///
/// This generates a inherent method for the first parameter of the given function
/// which builds a script, and either a struct or an enum for the script inputs.
///
/// There are two modes of operation, one which generates a struct and one which generates an enum.
#[proc_macro_attribute]
pub fn script(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let attr = syn::parse_macro_input!(attr as syn::AttributeArgs);
    let func = syn::parse_macro_input!(item as syn::ItemFn);
    let parsed_script = parse::parse_script(attr, func);
    let script_ident = quote! {__script_vec};
    let mut generate_script = generate::GenerateScript {
        script_ident,
        variant_states: state::VariantStates {
            states: Default::default(),
            predicate_atoms: vec![],
        },
        n_ident: 0,
        stmt_idx: 0,
        max_line_widths: vec![30, 40, 60, 80],
        formatted_lines: vec![],
    };
    let result = generate_script.run(parsed_script);
    result.into()
}
