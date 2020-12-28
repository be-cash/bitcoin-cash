extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::spanned::Spanned;

fn single_path(path: &syn::Path) -> Option<syn::Ident> {
    if path.segments.len() == 1 {
        Some(path.segments[0].ident.clone())
    } else {
        None
    }
}

fn parse_string_lit(lit: &syn::Lit) -> Option<String> {
    if let syn::Lit::Str(predicate_str) = lit {
        Some(predicate_str.value())
    } else {
        None
    }
}

fn generate(item_struct: syn::ItemStruct) -> Result<TokenStream, syn::Error> {
    let struct_name = &item_struct.ident;

    let mut crate_name = quote! {bitcoin_cash};

    for attr in &item_struct.attrs {
        match attr.parse_meta()? {
            syn::Meta::List(list) if list.path.to_token_stream().to_string() == "bitcoin_code" => {
                for nested_attr in list.nested.iter() {
                    match nested_attr {
                        syn::NestedMeta::Meta(attr_meta) => match attr_meta {
                            syn::Meta::NameValue(name_value) => {
                                let ident = single_path(&name_value.path).ok_or_else(|| {
                                    syn::Error::new(attr.span(), "Invalid attribute, invalid name")
                                })?;
                                match ident.to_string().as_str() {
                                    "crate" => {
                                        let crate_name_str = parse_string_lit(&name_value.lit)
                                            .ok_or_else(|| {
                                                syn::Error::new(
                                                    name_value.lit.span(),
                                                    "Invalid attribute, invalid value",
                                                )
                                            })?;
                                        crate_name =
                                            syn::Ident::new(&crate_name_str, name_value.lit.span())
                                                .to_token_stream();
                                    }
                                    _ => {
                                        return Err(syn::Error::new(
                                            name_value.span(),
                                            "Invalid attribute, unknown name",
                                        ))
                                    }
                                }
                            }
                            _ => {
                                return Err(syn::Error::new(
                                    attr_meta.span(),
                                    "Invalid parameter, must provide values like this: a=\"b\"",
                                ))
                            }
                        },
                        syn::NestedMeta::Lit(_) => {
                            return Err(syn::Error::new(
                                nested_attr.span(),
                                "Invalid literal, must provide values like this: a=\"b\"",
                            ))
                        }
                    }
                }
            }
            _ => {}
        }
    }

    let fields = match &item_struct.fields {
        syn::Fields::Named(fields) => fields,
        syn::Fields::Unnamed(_) => {
            return Err(syn::Error::new(
                struct_name.span(),
                "Cannot use macro for unnamed struct fields",
            ))
        }
        syn::Fields::Unit => {
            return Err(syn::Error::new(
                struct_name.span(),
                "Cannot use macro for unit structs",
            ))
        }
    };

    let parts_ident = quote! {__serialize_parts};

    let mut ser_calls = Vec::new();
    let mut deser_calls = Vec::new();
    let mut field_idents = Vec::new();

    for field in fields.named.iter() {
        let mut is_skipped = false;
        for attr in &field.attrs {
            match attr.parse_meta()? {
                syn::Meta::Path(_) => {
                    return Err(syn::Error::new(attr.span(), "Invalid attribute"))
                }
                syn::Meta::NameValue(_) => {
                    return Err(syn::Error::new(attr.span(), "Invalid attribute"))
                }
                syn::Meta::List(list) => {
                    let params = list
                        .nested
                        .iter()
                        .map(|param| param.to_token_stream().to_string())
                        .collect::<Vec<_>>();
                    match params
                        .iter()
                        .map(|param| param.as_str())
                        .collect::<Vec<_>>()
                        .as_slice()
                    {
                        &["skip"] => is_skipped = true,
                        _ => return Err(syn::Error::new(list.nested.span(), "Invalid attribute")),
                    }
                }
            }
        }
        let field_name = field.ident.as_ref().unwrap();
        field_idents.push(field_name);
        let field_name_str = field_name.to_string();
        let field_type = &field.ty;
        if !is_skipped {
            ser_calls.push(quote! {
                #parts_ident.push(self.#field_name.ser().named(#field_name_str));
            });
            deser_calls.push(quote! {
                let (#field_name, data) = <#field_type as #crate_name::BitcoinCode>::deser_rest(data)?;
            })
        } else {
            deser_calls.push(quote! {
                let #field_name = Default::default();
            })
        }
    }

    let capacity = fields.named.len();

    let result = quote! {
        impl #crate_name::BitcoinCode for #struct_name {
            fn ser(&self) -> #crate_name::ByteArray {
                let mut #parts_ident = Vec::with_capacity(#capacity);
                #(#ser_calls)*
                #crate_name::ByteArray::from_parts(#parts_ident)
            }

            fn deser_rest(data: #crate_name::ByteArray) -> std::result::Result<(Self, #crate_name::ByteArray), #crate_name::error::Error> {
                #(#deser_calls)*
                Ok((
                    #struct_name { #(#field_idents),* },
                    data,
                ))
            }
        }
    };
    Ok(result)
}

#[proc_macro_derive(BitcoinCode, attributes(bitcoin_code))]
pub fn serialize_macro(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item_struct = syn::parse_macro_input!(item as syn::ItemStruct);

    let result = match generate(item_struct) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    };

    result.into()
}
