use crate::gen_source;
use crate::ir;
use crate::state::{StackItem, State, VariantStates};
use bitcoin_cash_base::{Integer, Opcode};
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use std::collections::HashMap;
use std::iter::FromIterator;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;

pub type Error = syn::Error;

pub struct GenerateScript {
    pub variant_states: VariantStates,
    pub script_ident: TokenStream,
    pub n_ident: usize,
    pub stmt_idx: usize,
    pub max_line_widths: Vec<u32>,
    pub formatted_lines: Vec<Vec<String>>,
}

impl GenerateScript {
    pub fn run(&mut self, script: Result<ir::Script, syn::Error>) -> TokenStream {
        match self.run_script(script.map_err(Into::into)) {
            Ok(compiled_script) => compiled_script,
            Err(err) => err.to_compile_error(),
        }
    }

    fn run_script(&mut self, script: Result<ir::Script, Error>) -> Result<TokenStream, Error> {
        let mut script = script?;
        let stmt_token_streams = make_stmt_token_streams(&script.stmts);
        let mut new_stmts = Vec::with_capacity(script.stmts.len());
        let mut struct_fields = Vec::with_capacity(script.inputs.len());
        let mut enum_variant_fields = HashMap::with_capacity(script.script_variants.len());
        let mut impl_pushops = Vec::with_capacity(script.inputs.len());
        let mut impl_types = Vec::with_capacity(script.inputs.len());
        let mut impl_names = Vec::with_capacity(script.inputs.len());
        self.formatted_lines =
            gen_source::format_stmts(&self.max_line_widths, stmt_token_streams.into_iter())
                .map_err(|err| Error::new(script.sig.span(), err))?;
        let crate_ident = script
            .crate_ident
            .as_ref()
            .map(|ident| ident.to_token_stream())
            .unwrap_or_else(|| quote! { bitcoin_cash });
        for variant in script.script_variants.iter() {
            self.variant_states.states.insert(
                variant.name.clone(),
                State {
                    condition: variant.predicate.clone(),
                    stack: vec![],
                    alt_stack: vec![],
                },
            );
            enum_variant_fields.insert(variant.name.clone(), Vec::new());
        }
        if script.script_variants.is_empty() {
            self.variant_states.states.insert(
                script.input_struct.clone(),
                State {
                    condition: ir::VariantPredicate(vec![]),
                    stack: vec![],
                    alt_stack: vec![],
                },
            );
        }
        for input in script.inputs {
            let span = input.ident.span();
            let ident = &input.ident;
            let ty = &input.ty;
            let ident_str = ident.to_string();
            new_stmts.push(quote_spanned! {span=>
                let #ident = <#ty as Default>::default().to_data();
            });
            struct_fields.push(quote! {
                pub #ident: #ty
            });
            let source = input.token_stream.to_string();
            let src_code = self.max_line_widths.iter().map(|max_line_width| {
                quote! {
                    (#max_line_width, #source.into())
                }
            });
            impl_pushops.push(quote! {
                #crate_ident::TaggedOp {
                    op: self.#ident.to_pushop(),
                    src_file: file!().into(),
                    src_line: line!(),
                    src_column: column!(),
                    src_code: vec![#(#src_code),*],
                    pushed_names: Some(vec![Some(#ident_str.into())]),
                    alt_pushed_names: Some(vec![]),
                }
            });
            impl_types.push(quote! {
                <#ty as Default>::default().to_data_type()
            });
            impl_names.push(ident_str.clone());
            let stack_item = StackItem {
                ident: input.ident.clone(),
                name: ident_str,
                has_generated_name: false,
                integer: None,
            };
            if let Some(variants) = &input.variants {
                for variant in variants {
                    let stack = self
                        .variant_states
                        .states
                        .get_mut(&variant)
                        .ok_or_else(|| {
                            Error::new(span, format!("Undefined variant `{}`.", variant))
                        })?;
                    stack.stack.push(stack_item.clone());
                    let fields = enum_variant_fields.get_mut(&variant).unwrap();
                    fields.push((ident.clone(), ty.clone(), input.token_stream.clone()));
                }
            } else {
                self.push(stack_item);
                for variant in script.script_variants.iter() {
                    let fields = enum_variant_fields.get_mut(&variant.name).unwrap();
                    fields.push((ident.clone(), ty.clone(), input.token_stream.clone()));
                }
            }
        }
        for stmt in script.stmts {
            new_stmts.push(self.run_stmt(stmt.stmt, &crate_ident)?);
        }
        let attrs = script.attrs;
        let vis = script.vis;
        let mut inputs = Punctuated::new();
        inputs.push(script.sig.inputs[0].clone());
        script.sig.inputs = inputs;
        script.sig.output = syn::ReturnType::Default;
        let input_struct = script.input_struct;
        let sig = script.sig;
        let generics = sig.generics.clone();
        let generics_idents: Punctuated<_, syn::token::Comma> =
            Punctuated::from_iter(generics.params.iter().map(|param| match param {
                syn::GenericParam::Type(ty) => ty.ident.to_token_stream(),
                syn::GenericParam::Lifetime(lt) => lt.lifetime.to_token_stream(),
                _ => panic!("Generic const not supported"),
            }));
        let script_ident = &self.script_ident;

        let (input_struct_enum, impl_ops, impl_types, impl_names) =
            if script.script_variants.is_empty() {
                (
                    quote! {
                        #vis struct #input_struct #generics {
                            #(#struct_fields),*
                        }
                    },
                    quote! {
                        vec![
                            #(#impl_pushops),*
                        ]
                    },
                    quote! {
                        vec![
                            #(#impl_types),*
                        ]
                    },
                    quote! {
                        &[
                            #(#impl_names),*
                        ]
                    },
                )
            } else {
                let mut enum_variants = Vec::with_capacity(script.script_variants.len());
                let mut match_ops = Vec::with_capacity(script.script_variants.len());
                let mut match_types = Vec::with_capacity(script.script_variants.len());
                let mut match_names = Vec::with_capacity(script.script_variants.len());
                for (variant_name, variant_fields) in enum_variant_fields {
                    let variant_name_str = variant_name.to_string();
                    let variant_fields_quote = variant_fields.iter().map(|(ident, ty, _)| {
                        quote! {
                            #ident: #ty
                        }
                    });
                    enum_variants.push(quote! {
                        #variant_name {
                            #(#variant_fields_quote),*
                        }
                    });

                    let unpack_variant = variant_fields.iter().map(|(ident, _, _)| ident);
                    let variant_pushops = variant_fields.iter().map(|(ident, _, token_stream)| {
                        let ident_str = ident.to_string();
                        let source = token_stream.to_string();
                        let src_code = self.max_line_widths.iter().map(|max_line_width| {
                            quote! {
                                (#max_line_width, #source.into())
                            }
                        });
                        quote! {
                            #crate_ident::TaggedOp {
                                op: #ident.to_pushop(),
                                src_file: file!().into(),
                                src_line: line!(),
                                src_column: column!(),
                                src_code: vec![#(#src_code),*],
                                pushed_names: Some(vec![Some(#ident_str.into())]),
                                alt_pushed_names: Some(vec![]),
                            }
                        }
                    });
                    match_ops.push(quote! {
                        #input_struct::#variant_name { #(#unpack_variant),* } => vec![
                            #(#variant_pushops),*
                        ]
                    });

                    let variant_types = variant_fields.iter().map(|(_, ty, _)| {
                        quote! {
                            <#ty as Default>::default().to_data_type()
                        }
                    });
                    match_types.push(quote! {
                        Some(#variant_name_str) => vec![
                            #(#variant_types),*
                        ]
                    });
                    let field_names_str =
                        variant_fields.iter().map(|(ident, _, _)| ident.to_string());
                    match_names.push(quote! {
                        Some(#variant_name_str) => &[
                            #(#field_names_str),*
                        ]
                    });
                }
                let match_none = quote! {
                    None => panic!("Must provide enum variant name")
                };
                let match_unknown = quote! {
                    Some(variant) => panic!(format!("Unknown variant: {}", variant))
                };
                match_types.push(match_unknown.clone());
                match_types.push(match_none.clone());
                match_names.push(match_unknown);
                match_names.push(match_none);
                (
                    quote! {
                        #vis enum #input_struct #generics {
                            #(#enum_variants),*
                        }
                    },
                    quote! {
                        match self {
                            #(#match_ops),*
                        }
                    },
                    quote! {
                        match variant_name {
                            #(#match_types),*
                        }
                    },
                    quote! {
                        match variant_name {
                            #(#match_names),*
                        }
                    },
                )
            };

        Ok(quote! {
            #input_struct_enum

            impl #generics #crate_ident::Ops for #input_struct<#generics_idents> {
                fn ops(&self) -> std::borrow::Cow<[#crate_ident::TaggedOp]> {
                    use #crate_ident::BitcoinDataType;
                    #impl_ops.into()
                }
            }

            impl #generics #crate_ident::InputScript for #input_struct<#generics_idents> {
                fn types(variant_name: Option<&str>) -> Vec<#crate_ident::DataType> {
                    use #crate_ident::BitcoinDataType;
                    #impl_types
                }

                fn names(variant_name: Option<&str>) -> &'static [&'static str] {
                    #impl_names
                }
            }

            #[allow(redundant_semicolon)]
            #(#attrs)*
            #vis #sig -> #crate_ident::TaggedScript<#input_struct<#generics_idents>> {
                use #crate_ident::BitcoinDataType;
                let mut #script_ident = Vec::new();
                #(#new_stmts)*
                return #crate_ident::TaggedScript::new(#script_ident);
            }
        })
    }

    fn run_stmt(
        &mut self,
        stmt: ir::Stmt,
        crate_ident: &TokenStream,
    ) -> Result<TokenStream, Error> {
        match stmt {
            ir::Stmt::ForLoop(for_loop) => Err(Error::new(
                for_loop.span,
                "For loops not implemented yet".to_string(),
            )),
            ir::Stmt::RustIf(if_stmt) => Err(Error::new(
                if_stmt.span,
                "`if` not implemented yet".to_string(),
            )),
            ir::Stmt::Push(push) => self.run_push(push, crate_ident),
            ir::Stmt::Opcode(opcode) => self.run_opcode(opcode, crate_ident),
            ir::Stmt::ScriptIf(script_if) => self.run_if(script_if, crate_ident),
        }
    }

    fn run_push(
        &mut self,
        push: ir::PushStmt,
        crate_ident: &TokenStream,
    ) -> Result<TokenStream, Error> {
        let src = self.next_formatted_stmts();
        let has_generated_name = push.output_name.is_none();
        let span = push.span;
        let output_name = push
            .output_name
            .as_ref()
            .map(|ident| {
                let name = ident.to_string();
                quote! { Some(#name.into()) }
            })
            .unwrap_or(quote! { None });
        let ident = push
            .output_name
            .or_else(|| Some(self.make_ident(span)))
            .unwrap();
        self.variant_states.push(StackItem {
            ident: ident.clone(),
            name: ident.to_string(),
            has_generated_name,
            integer: None, // TODO
        });
        let script_ident = &self.script_ident;
        let expr = push.expr;
        Ok(quote_spanned! {span=>
            let #ident = (#expr).to_data();
            #script_ident.push(#crate_ident::TaggedOp {
                src_code: #src,
                src_line: line!(),
                src_column: column!(),
                src_file: file!().into(),
                op: (#expr).to_pushop(),
                pushed_names: Some(vec![#output_name]),
                alt_pushed_names: Some(vec![]),
            });
        })
    }

    fn run_opcode(
        &mut self,
        opcode: ir::OpcodeStmt,
        crate_ident: &TokenStream,
    ) -> Result<TokenStream, Error> {
        use Opcode::*;
        let script_ident = self.script_ident.clone();
        let ident = &opcode.ident;
        let expr_span = opcode.expr_span;
        let opcode_type = bitcoin_cash_base::MAP_NAME_TO_ENUM.get(&ident.to_string());
        match opcode_type {
            Some(&opcode_type @ OP_TOALTSTACK) => {
                let src = self.next_formatted_stmts();
                let mut item = self.pop(opcode_type, expr_span)?;
                Self::verify_item_name(opcode_type, &opcode, &item)?;
                Self::update_item_name(opcode_type, &opcode, &mut item)?;
                let name = item.name_tokens();
                self.push_alt(item);
                Ok(quote_spanned! {expr_span=>
                    #script_ident.push(#crate_ident::TaggedOp {
                        src_code: #src,
                        src_line: line!(),
                        src_column: column!(),
                        src_file: file!().into(),
                        op: #crate_ident::Op::Code(#ident),
                        pushed_names: Some(vec![]),
                        alt_pushed_names: Some(vec![#name]),
                    });
                })
            }
            Some(&opcode_type @ OP_FROMALTSTACK) => {
                let src = self.next_formatted_stmts();
                let mut item = self.pop_alt(opcode_type, expr_span)?;
                Self::verify_item_name(opcode_type, &opcode, &item)?;
                Self::update_item_name(opcode_type, &opcode, &mut item)?;
                let name = item.name_tokens();
                self.push(item);
                Ok(quote_spanned! {expr_span=>
                    #script_ident.push(#crate_ident::TaggedOp {
                        src_code: #src,
                        src_line: line!(),
                        src_column: column!(),
                        src_file: file!().into(),
                        op: #crate_ident::Op::Code(#ident),
                        pushed_names: Some(vec![#name]),
                        alt_pushed_names: Some(vec![]),
                    });
                })
            }
            Some(&opcode_type @ OP_PICK) | Some(&opcode_type @ OP_ROLL) => {
                let src = self.next_formatted_stmts();
                let stack_item = self.pop(opcode_type, expr_span)?;
                Self::verify_item_name(opcode_type, &opcode, &stack_item)?;
                let item_depth = match stack_item.integer {
                    Some(integer) => integer as usize,
                    _ => 0, // take top stack item if not known statically
                };
                let mut item = match opcode_type {
                    OP_PICK => self
                        .variant_states
                        .pick(item_depth)
                        .map_err(|err| error_opcode(err, opcode_type, expr_span))?,
                    OP_ROLL => self
                        .variant_states
                        .roll(item_depth)
                        .map_err(|err| error_opcode(err, opcode_type, expr_span))?,
                    _ => unreachable!(),
                };
                Self::update_item_name(opcode_type, &opcode, &mut item)?;
                let name = item.name_tokens();
                self.push(item);
                let ident = opcode.ident;
                let input_name = stack_item.ident;
                Ok(quote_spanned! {expr_span=>
                    #crate_ident::func::#ident(#input_name);
                    #script_ident.push(#crate_ident::TaggedOp {
                        src_code: #src,
                        src_line: line!(),
                        src_column: column!(),
                        src_file: file!().into(),
                        op: #crate_ident::Op::Code(#ident),
                        pushed_names: Some(vec![#name]),
                        alt_pushed_names: Some(vec![]),
                    });
                })
            }
            Some(&opcode_type) => self.run_other_opcode(opcode_type, opcode, crate_ident),
            None => self.run_opcode_function(opcode, crate_ident),
        }
    }

    fn run_other_opcode(
        &mut self,
        opcode_type: Opcode,
        opcode: ir::OpcodeStmt,
        crate_ident: &TokenStream,
    ) -> Result<TokenStream, Error> {
        let src = self.next_formatted_stmts();
        let expr_span = opcode.expr_span;
        let outputs_span = opcode.outputs_span;
        let behavior = opcode_type.behavior();
        let mut input_items = Vec::new();
        for _ in 0..behavior.input_types.len() {
            let item = self.pop(opcode_type, expr_span)?;
            input_items.push(item);
        }
        input_items.reverse();
        if let Some(input_names) = opcode.input_names.clone() {
            if input_items.len() != input_names.len() {
                return Err(Error::new(
                    expr_span,
                    format!(
                        "Expected {} input names but got {}.",
                        input_items.len(),
                        input_names.len()
                    ),
                ));
            }
            for (input_item, input_name) in input_items.iter().zip(input_names) {
                if let ir::OpcodeInput::Ident(ident) = input_name {
                    if &ident.to_string() == "__" {
                        continue;
                    }
                    if input_item.has_generated_name {
                        return Err(Error::new(
                            ident.span(),
                            format!(
                                "Expected named top stack item for `{}` but got unnamed.",
                                ident
                            ),
                        ));
                    } else if input_item.name != ident.to_string() {
                        return Err(Error::new(
                            ident.span(),
                            format!(
                                "Mismatched stack item name, expected `{}` but got `{}`.",
                                input_item.name,
                                ident,
                                // TODO: stack_names,
                            ),
                        ));
                    }
                }
            }
        }
        let mut pushed_stack_items = Vec::with_capacity(behavior.output_types.len());
        if let Some(output_names) = opcode.output_names {
            if output_names.len() != behavior.output_types.len() {
                return Err(Error::new(
                    outputs_span,
                    format!(
                        "Invalid number of output names. {:?} creates {} items, but {} defined.",
                        opcode_type,
                        behavior.output_types.len(),
                        output_names.len(),
                    ),
                ));
            }
            for (idx, ident) in output_names.into_iter().enumerate() {
                let new_ident = self.make_ident(ident.span());
                let name = ident.to_string();
                if &name == "__" {
                    if opcode_type.retains_input() {
                        let new_idx = 0;
                        pushed_stack_items.push(StackItem {
                            ident: new_ident,
                            ..input_items[new_idx].clone()
                        });
                        continue;
                    } else if let Some(output_order) = behavior.output_order {
                        let new_idx = output_order[idx];
                        pushed_stack_items.push(StackItem {
                            ident: new_ident,
                            ..input_items[new_idx].clone()
                        });
                        continue;
                    } else {
                        return Err(Error::new(
                            outputs_span,
                            format!(
                                "Cannot use `__` as output placeholder for opcode {:?}",
                                opcode_type
                            ),
                        ));
                    }
                }
                pushed_stack_items.push(StackItem {
                    name,
                    ident: new_ident,
                    has_generated_name: false,
                    integer: None,
                });
            }
        } else {
            match behavior.output_order {
                Some(output_order) => {
                    for &new_idx in output_order {
                        let new_ident = self.make_ident(input_items[new_idx].ident.span());
                        pushed_stack_items.push(StackItem {
                            ident: new_ident,
                            ..input_items[new_idx].clone()
                        });
                    }
                }
                None => {
                    for _ in 0..behavior.output_types.len() {
                        let ident = self.make_ident(outputs_span);
                        if opcode_type.retains_input() {
                            let new_idx = 0;
                            pushed_stack_items.push(StackItem {
                                ident,
                                ..input_items[new_idx].clone()
                            });
                        } else {
                            pushed_stack_items.push(StackItem {
                                name: ident.to_string(),
                                ident,
                                has_generated_name: true,
                                integer: None,
                            });
                        }
                    }
                }
            }
        }
        let output_idents = pushed_stack_items
            .iter()
            .map(|item| item.ident.clone())
            .collect::<Vec<_>>();
        let pushed_names = pushed_stack_items
            .iter()
            .map(|item| item.name_tokens())
            .collect::<Vec<_>>();
        for stack_item in pushed_stack_items {
            self.push(stack_item);
        }
        let output_let = if output_idents.len() == 1 {
            let output_ident = &output_idents[0];
            quote_spanned! {outputs_span=> let #output_ident = }
        } else {
            quote_spanned! {outputs_span=> let (#(#output_idents),*) = }
        };
        let input_idents = if let Some(input_names) = &opcode.input_names {
            input_names
                .iter()
                .zip(&input_items)
                .map(|(input, stack_item)| {
                    let mut ident = stack_item.ident.clone();
                    let span = match input {
                        ir::OpcodeInput::Expr(expr) => expr.span(),
                        ir::OpcodeInput::Ident(input_ident) => input_ident.span(),
                    };
                    ident.set_span(span);
                    quote_spanned! {span=>#ident.clone()}
                })
                .collect::<Vec<_>>()
        } else {
            input_items
                .iter()
                .map(|stack_item| {
                    let mut ident = stack_item.ident.clone();
                    ident.set_span(expr_span);
                    quote_spanned! {expr_span=>#ident.clone()}
                })
                .collect()
        };
        let ident = opcode.ident;
        let script_ident = &self.script_ident;
        let prefix = quote_spanned! {outputs_span=>
            #output_let #crate_ident::func::#ident
        };
        let inputs = quote! {
            #(#input_idents),*
        };
        let push = quote_spanned! {expr_span=>
            #script_ident.push(#crate_ident::TaggedOp {
                src_code: #src,
                src_line: line!(),
                src_column: column!(),
                src_file: file!().into(),
                op: #crate_ident::Op::Code(#ident),
                pushed_names: Some(vec![#(#pushed_names),*]),
                alt_pushed_names: Some(vec![]),
            });
        };
        Ok(quote! {
            #prefix ( #inputs );
            #push
        })
    }

    fn run_opcode_function(
        &mut self,
        opcode: ir::OpcodeStmt,
        crate_ident: &TokenStream,
    ) -> Result<TokenStream, Error> {
        match opcode.ident.to_string().as_str() {
            "depth_of" => {
                let src = self.next_formatted_stmts();
                if let Some(&[ir::OpcodeInput::Ident(ref ident)]) = opcode.input_names.as_deref() {
                    let (depth, _) = self
                        .variant_states
                        .find_item(ident)
                        .map_err(|err| Error::new(opcode.expr_span, err))?;
                    let has_generated_name = opcode.output_names.is_none();
                    let span = opcode.expr_span;
                    let mut name = quote! { None };
                    let ident = if let Some(&[ref ident]) = opcode.output_names.as_deref() {
                        let ident_str = ident.to_string();
                        name = quote! { Some(#ident_str.into()) };
                        ident.clone()
                    } else {
                        self.make_ident(opcode.outputs_span)
                    };
                    let depth = depth as Integer;
                    self.push(StackItem {
                        ident: ident.clone(),
                        name: ident.to_string(),
                        has_generated_name,
                        integer: Some(depth),
                    });
                    let script_ident = &self.script_ident;
                    Ok(quote_spanned! {span=>
                        let #ident = (#depth).to_data();
                        #script_ident.push(#crate_ident::TaggedOp {
                            src_code: #src,
                            src_line: line!(),
                            src_column: column!(),
                            src_file: file!().into(),
                            op: (#depth).to_pushop(),
                            pushed_names: Some(vec![#name]),
                            alt_pushed_names: Some(vec![]),
                        });
                    })
                } else {
                    Err(Error::new(opcode.expr_span, "Expected 1 variable name"))
                }
            }
            "transmute" => {
                if let Some(&[ir::OpcodeInput::Ident(ref ident), ref type_input]) =
                    opcode.input_names.as_deref()
                {
                    let type_expr = match type_input {
                        ir::OpcodeInput::Expr(type_expr) => type_expr.to_token_stream(),
                        ir::OpcodeInput::Ident(ident) => ident.to_token_stream(),
                    };
                    let span = opcode.expr_span;
                    if let Some(&[ref out_ident]) = opcode.output_names.as_deref() {
                        if out_ident != ident {
                            return Err(Error::new(
                                opcode.outputs_span,
                                "Input and output name must be the same",
                            ));
                        }
                    }
                    let (_, item) = self
                        .variant_states
                        .find_item(ident)
                        .map_err(|err| Error::new(span, err))?;
                    let item_ident = &item.ident;
                    Ok(quote_spanned! {span=>
                        let #item_ident = <#type_expr as Default>::default().to_data();
                    })
                } else {
                    Err(Error::new(
                        opcode.expr_span,
                        format!("Expected 1 parameter, got {:?}", opcode.input_names),
                    ))
                }
            }
            func => Err(Error::new(
                opcode.expr_span,
                format!("Unknown opcode/function: {}", func),
            )),
        }
    }

    fn run_if(
        &mut self,
        script_if: ir::ScriptIfStmt,
        crate_ident: &TokenStream,
    ) -> Result<TokenStream, Error> {
        let mut tokens = Vec::new();
        tokens.push(self.run_opcode(script_if.if_opcode.clone(), crate_ident)?);
        let predicate_name = script_if
            .if_opcode
            .input_names
            .as_ref()
            .and_then(|input_names| input_names.get(0))
            .and_then(|input_name| {
                if let ir::OpcodeInput::Ident(ident) = input_name {
                    Some(ident.to_string())
                } else {
                    None
                }
            })
            .ok_or_else(|| {
                Error::new(
                    script_if.if_opcode.expr_span,
                    "Must provide variable name to `OP_IF`",
                )
            })?;
        let stack_before = self.variant_states.clone();
        self.variant_states
            .predicate_atoms
            .push(ir::VariantPredicateAtom {
                var_name: predicate_name.clone(),
                is_positive: true,
            });
        let predicate_held_if = self.variant_states.predicate_atoms.clone();

        let mut then_tokens = Vec::new();
        for stmt in script_if.then_stmts {
            then_tokens.push(self.run_stmt(stmt.stmt, crate_ident)?);
        }
        if let Some(else_opcode) = script_if.else_opcode {
            then_tokens.push(self.run_opcode(else_opcode, crate_ident)?);
        }
        let mut stack_after_then = std::mem::replace(&mut self.variant_states, stack_before);
        self.variant_states
            .predicate_atoms
            .push(ir::VariantPredicateAtom {
                var_name: predicate_name,
                is_positive: false,
            });
        let mut else_tokens = Vec::new();
        for stmt in script_if.else_stmts {
            else_tokens.push(self.run_stmt(stmt.stmt, crate_ident)?);
        }
        else_tokens.push(self.run_opcode(script_if.endif_opcode, crate_ident)?);
        let predicate_held_else = self.variant_states.predicate_atoms.clone();
        self.variant_states.predicate_atoms.pop().unwrap();
        for (variant_name, stack) in self.variant_states.states.iter_mut() {
            let held_if = stack.condition.holds(&predicate_held_if);
            let held_else = stack.condition.holds(&predicate_held_else);
            if held_if && !held_else {
                std::mem::swap(
                    stack_after_then.states.get_mut(variant_name).unwrap(),
                    stack,
                );
            }
        }

        tokens.push(quote! {
            #(#then_tokens)*
            #(#else_tokens)*
        });

        Ok(tokens.into_iter().collect())
    }

    fn next_formatted_stmts(&mut self) -> TokenStream {
        let stmt_idx = self.stmt_idx;
        self.stmt_idx += 1;
        let stmts = self
            .formatted_lines
            .iter()
            .zip(self.max_line_widths.iter())
            .map(|(lines, max_line_width)| {
                let line = lines
                    .get(stmt_idx)
                    .map(|line| line.as_str())
                    .unwrap_or_else(|| "<unknown>");
                quote! {
                    (#max_line_width, #line.into())
                }
            });
        quote! {
            vec![#(#stmts),*]
        }
    }

    fn verify_item_name(
        opcode_type: Opcode,
        opcode: &ir::OpcodeStmt,
        item: &StackItem,
    ) -> Result<(), Error> {
        if let Some(input_names) = &opcode.input_names {
            if input_names.len() != 1 {
                return Err(error_opcode(
                    format!("Expected 1 argument, got {}.", input_names.len()),
                    opcode_type,
                    opcode.expr_span,
                ));
            }
            if input_names[0].to_string() != item.name {
                return Err(error_opcode(
                    format!(
                        "Expected top altstack item named `{}`, but actual name is `{}`.",
                        input_names[0], item.name,
                    ),
                    opcode_type,
                    opcode.expr_span,
                ));
            }
        }
        Ok(())
    }

    fn update_item_name(
        opcode_type: Opcode,
        opcode: &ir::OpcodeStmt,
        item: &mut StackItem,
    ) -> Result<(), Error> {
        if let Some(output_names) = &opcode.output_names {
            if output_names.len() != 1 {
                return Err(error_opcode(
                    format!("Pushes 1 item, got {}.", output_names.len()),
                    opcode_type,
                    opcode.outputs_span,
                ));
            }
            item.name = output_names[0].to_string();
        }
        Ok(())
    }

    fn pop(&mut self, opcode: Opcode, span: Span) -> Result<StackItem, Error> {
        self.variant_states
            .pop()
            .map_err(|err| error_opcode(err, opcode, span))
    }

    fn pop_alt(&mut self, opcode: Opcode, span: Span) -> Result<StackItem, Error> {
        self.variant_states
            .pop_alt()
            .map_err(|err| error_opcode(err, opcode, span))
    }

    fn push(&mut self, stack_item: StackItem) {
        self.variant_states.push(stack_item)
    }

    fn push_alt(&mut self, stack_item: StackItem) {
        self.variant_states.push_alt(stack_item)
    }

    fn make_ident(&mut self, span: Span) -> syn::Ident {
        let ident = syn::Ident::new(&format!("__id_{}", self.n_ident), span);
        self.n_ident += 1;
        ident
    }
}

fn error_opcode<D: std::fmt::Display>(msg: D, opcode: Opcode, span: Span) -> Error {
    Error::new(span, format!("{:?}: {}", opcode, msg))
}

fn make_stmt_token_streams<'a>(
    stmts: impl IntoIterator<Item = &'a ir::TaggedStmt>,
) -> Vec<&'a TokenStream> {
    let mut vec = Vec::new();
    for stmt in stmts {
        match &stmt.stmt {
            ir::Stmt::Push(_) => vec.push(&stmt.token_stream),
            ir::Stmt::Opcode(opcode) => {
                if &opcode.ident.to_string() != "transmute" {
                    vec.push(&stmt.token_stream);
                }
            }
            ir::Stmt::ForLoop(for_loop) => {
                vec.extend_from_slice(&make_stmt_token_streams(&for_loop.stmts))
            }
            ir::Stmt::RustIf(rust_if) => {
                vec.extend_from_slice(&make_stmt_token_streams(&rust_if.then_branch));
                if let Some(else_branch) = &rust_if.else_branch {
                    vec.extend_from_slice(&make_stmt_token_streams(else_branch));
                }
            }
            ir::Stmt::ScriptIf(script_if) => {
                vec.push(&script_if.if_token_stream);
                vec.extend_from_slice(&make_stmt_token_streams(&script_if.then_stmts));
                if let Some(else_token_stream) = &script_if.else_token_stream {
                    vec.push(else_token_stream);
                }
                vec.extend_from_slice(&make_stmt_token_streams(&script_if.else_stmts));
                vec.push(&script_if.endif_token_stream);
            }
        }
    }
    vec
}
