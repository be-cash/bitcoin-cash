extern crate proc_macro;

mod generate;
mod ir;
mod parse;
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
        script_ident: script_ident.clone(),
        stack: vec![],
        alt_stack: vec![],
        n_ident: 0,
    };
    let result = generate_script.run(parsed_script);
    result.into()
}
