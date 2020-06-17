extern crate proc_macro;

mod gen_source;
mod generate;
mod ir;
mod parse;
mod state;

use quote::quote;

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
        script_ident: script_ident,
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
