use crate::ir;
use bitcoin_cash_script::{Integer, OpcodeType};
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};

#[derive(Clone)]
pub struct StackItem {
    ident: syn::Ident,
    name: String,
    has_generated_name: bool,
    integer: Option<Integer>,
}

pub struct GenerateScript {
    pub script_ident: TokenStream,
    pub stack: Vec<StackItem>,
    pub alt_stack: Vec<StackItem>,
    pub n_ident: usize,
}

pub struct Error {
    pub errs: Vec<syn::Error>,
}
impl Into<Error> for syn::Error {
    fn into(self) -> Error {
        Error { errs: vec![self] }
    }
}
impl Error {
    fn new<D: std::fmt::Display>(span: Span, msg: D) -> Self {
        syn::Error::new(span, msg).into()
    }
}

impl GenerateScript {
    pub fn run(&mut self, script: Result<ir::Script, syn::Error>) -> TokenStream {
        match self.run_script(script.map_err(Into::into)) {
            Ok(compiled_script) => compiled_script,
            Err(err) => err
                .errs
                .into_iter()
                .map(|err| err.to_compile_error())
                .collect(),
        }
    }

    fn run_script(&mut self, script: Result<ir::Script, Error>) -> Result<TokenStream, Error> {
        let mut script = script?;
        let mut new_stmts = Vec::with_capacity(script.stmts.len());
        let mut struct_fields = Vec::with_capacity(script.inputs.len());
        let mut impl_pushops = Vec::with_capacity(script.inputs.len());
        let mut impl_types = Vec::with_capacity(script.inputs.len());
        let mut impl_names = Vec::with_capacity(script.inputs.len());
        for input in script.inputs {
            let span = input.ident.span();
            let ident = &input.ident;
            let ty = &input.ty;
            let ident_str = format!("{}", ident);
            new_stmts.push(quote_spanned! {span=>
                let #ident = <#ty as Default>::default().to_data();
            });
            struct_fields.push(quote! {
                pub #ident: #ty
            });
            impl_pushops.push(quote! {
                self.#ident.to_pushop()
            });
            impl_types.push(quote! {
                <#ty as Default>::default().to_data_type()
            });
            impl_names.push(ident_str.clone());
            self.push(StackItem {
                ident: input.ident,
                name: ident_str,
                has_generated_name: false,
                integer: None,
            });
        }
        for stmt in script.stmts {
            new_stmts.push(self.run_stmt(stmt)?);
        }
        let attrs = script.attrs;
        let vis = script.vis;
        let mut inputs = syn::punctuated::Punctuated::new();
        inputs.push(script.sig.inputs[0].clone());
        script.sig.inputs = inputs;
        script.sig.output = syn::ReturnType::Default;
        let input_struct = script.input_struct;
        let sig = script.sig;
        let script_ident = &self.script_ident;

        Ok(quote! {
            #vis struct #input_struct {
                #(#struct_fields),*
            }

            impl bitcoin_cash_script::Ops for #input_struct {
                fn ops(&self) -> std::borrow::Cow<[bitcoin_cash_script::Op]> {
                    use bitcoin_cash_script::BitcoinDataType;
                    vec![
                        #(#impl_pushops),*
                    ].into()
                }
            }

            impl bitcoin_cash_script::InputScript for #input_struct {
                fn types() -> Vec<bitcoin_cash_script::DataType> {
                    use bitcoin_cash_script::BitcoinDataType;
                    vec![
                        #(#impl_types),*
                    ]
                }

                fn names() -> &'static [&'static str] {
                    &[
                        #(#impl_names),*
                    ]
                }
            }

            #[allow(redundant_semicolon)]
            #(#attrs)*
            #vis #sig -> bitcoin_cash_script::TaggedScript<#input_struct> {
                use bitcoin_cash_script::BitcoinDataType;
                let mut #script_ident = Vec::new();
                #(#new_stmts)*
                return bitcoin_cash_script::TaggedScript::new(#script_ident);
            }
        })
    }

    fn run_stmt(&mut self, stmt: ir::Stmt) -> Result<TokenStream, Error> {
        match stmt {
            ir::Stmt::ForLoop(for_loop) => Err(Error::new(
                for_loop.span,
                format!("For loops not implemented yet"),
            )),
            ir::Stmt::RustIf(if_stmt) => {
                Err(Error::new(if_stmt.span, format!("If not implemented yet")))
            }
            ir::Stmt::Push(src, push) => self.run_push(src, push),
            ir::Stmt::Opcode(src, opcode) => self.run_opcode(src, opcode),
            ir::Stmt::ScriptIf(src, script_if) => {
                let mut tokens = Vec::new();
                tokens.push(self.run_opcode(src, script_if.if_opcode)?);
                let stack_before = self.stack.clone();
                let endif_span = script_if.endif_opcode.span;
                let mut then_tokens = Vec::new();
                for stmt in script_if.then_stmts {
                    then_tokens.push(self.run_stmt(stmt)?);
                }
                if let Some(else_opcode) = script_if.else_opcode {
                    then_tokens
                        .push(self.run_opcode(format!("{}", else_opcode.ident), else_opcode)?);
                }
                let stack_after_then = std::mem::replace(&mut self.stack, stack_before);
                let mut else_tokens = Vec::new();
                for stmt in script_if.else_stmts {
                    else_tokens.push(self.run_stmt(stmt)?);
                }
                else_tokens.push(self.run_opcode(
                    format!("{}", script_if.endif_opcode.ident),
                    script_if.endif_opcode,
                )?);
                if stack_after_then.len() != self.stack.len() {
                    let then_names = stack_after_then
                        .iter()
                        .map(|item| format!("{}", item.ident))
                        .collect::<Vec<_>>()
                        .join(", ");
                    let else_names = self
                        .stack
                        .iter()
                        .map(|item| format!("{}", item.ident))
                        .collect::<Vec<_>>()
                        .join(", ");
                    return Err(Error::new(endif_span, format!(
                        "Stacks in this branch result in different stack heights. After OP_IF: [{}], after OP_ELSE: [{}]",
                        then_names,
                        else_names,
                    )));
                }
                for (idx, (then_item, else_item)) in stack_after_then
                    .iter()
                    .rev()
                    .zip(self.stack.iter_mut().rev())
                    .enumerate()
                {
                    if then_item.has_generated_name != else_item.has_generated_name {
                        return Err(Error::new(
                            endif_span,
                            format!(
                                "Branch results in inconsistent stack item names. \
                                 Item {} from the top {} a name in the OP_IF branch while \
                                 it {} in the OP_ELSE branch.",
                                idx,
                                if !then_item.has_generated_name {
                                    "has"
                                } else {
                                    "doesn't have"
                                },
                                if !else_item.has_generated_name {
                                    "does"
                                } else {
                                    "doesn't"
                                },
                            ),
                        ));
                    } else if !then_item.has_generated_name && !else_item.has_generated_name {
                        if then_item.name != else_item.name {
                            return Err(Error::new(
                                endif_span,
                                format!(
                                    "Branch results in inconsistent stack item names. \
                                     Item {} from the top is named `{}` in the OP_IF branch while \
                                     it is named `{}` in the OP_ELSE branch.",
                                    idx, then_item.name, else_item.name,
                                ),
                            ));
                        }
                    }
                    else_item.integer = None;
                }
                let then_outputs = stack_after_then
                    .iter()
                    .map(|item| item.ident.clone())
                    .collect::<Vec<_>>();
                let else_outputs = self
                    .stack
                    .iter()
                    .map(|item| item.ident.clone())
                    .collect::<Vec<_>>();

                tokens.push(quote! {
                    let (#(#else_outputs),* ,) = bitcoin_cash_script::func::SECOND({
                        #(#then_tokens)*
                        (#(#then_outputs.clone()),* ,)
                    }, {
                        #(#else_tokens)*
                        (#(#else_outputs.clone()),* ,)
                    });
                });

                Ok(tokens.into_iter().collect())
            }
        }
    }

    fn run_push(&mut self, src: String, push: ir::Push) -> Result<TokenStream, Error> {
        let has_generated_name = push.output_name.is_none();
        let span = push.span;
        let output_names = Self::to_vec_str_tokens(
            push.output_name
                .as_ref()
                .map(|ident| vec![ident.clone()])
                .as_ref()
                .map(Vec::as_slice),
        );
        let ident = push
            .output_name
            .or_else(|| Some(self.make_ident(span)))
            .unwrap();
        self.stack.push(StackItem {
            ident: ident.clone(),
            name: ident.to_string(),
            has_generated_name,
            integer: None, // TODO
        });
        let script_ident = &self.script_ident;
        let expr = push.expr;
        Ok(quote_spanned! {span=>
            let #ident = (#expr).to_data();
            #script_ident.push(bitcoin_cash_script::TaggedOp {
                src: #src.into(),
                op: (#expr).to_pushop(),
                input_names: None,
                output_names: #output_names,
            });
        })
    }

    fn run_opcode(&mut self, src: String, opcode: ir::Opcode) -> Result<TokenStream, Error> {
        use OpcodeType::*;
        let script_ident = self.script_ident.clone();
        let ident = &opcode.ident;
        let span = opcode.span;
        let input_names = Self::to_vec_str_tokens(opcode.input_names.as_ref().map(Vec::as_slice));
        let output_names = Self::to_vec_str_tokens(opcode.output_names.as_ref().map(Vec::as_slice));
        let opcode_type = bitcoin_cash_script::MAP_NAME_TO_ENUM.get(&ident.to_string());
        match opcode_type {
            Some(&opcode_type @ OP_TOALTSTACK) => {
                let stack_item = self.pop(opcode_type, span)?;
                self.alt_stack.push(stack_item);
                Ok(quote_spanned! {span=>
                    #script_ident.push(bitcoin_cash_script::TaggedOp {
                        src: #src.into(),
                        op: bitcoin_cash_script::Op::Code(#ident),
                        input_names: #input_names,
                        output_names: #output_names,
                    });
                })
            }
            Some(&opcode_type @ OP_FROMALTSTACK) => {
                self.stack
                    .push(Self::pop_stack(&mut self.alt_stack, opcode_type, span)?);
                Ok(quote_spanned! {span=>
                    #script_ident.push(bitcoin_cash_script::TaggedOp {
                        src: #src.into(),
                        op: bitcoin_cash_script::Op::Code(#ident),
                        input_names: #input_names,
                        output_names: #output_names,
                    });
                })
            }
            Some(&opcode_type @ OP_PICK) | Some(&opcode_type @ OP_ROLL) => {
                let stack_item = self.pop(opcode_type, span)?;
                let item_idx = match stack_item.integer {
                    Some(integer) => integer as usize,
                    _ => Err(Error::new(
                        span,
                        format!(
                            "{:?} expects an integer literal as top stack item",
                            opcode_type
                        ),
                    ))?,
                };
                if item_idx >= self.stack.len() {
                    Err(Error::new(
                        span,
                        format!(
                            "{:?} tried to access {} items deep, but stack only has {} items",
                            opcode_type,
                            item_idx,
                            self.stack.len(),
                        ),
                    ))?
                }
                match opcode_type {
                    OP_PICK => {
                        self.stack
                            .push(self.stack[self.stack.len() - item_idx - 1].clone());
                    }
                    OP_ROLL => {
                        let rolled_stack_item = self.stack.remove(self.stack.len() - item_idx - 1);
                        self.stack.push(rolled_stack_item);
                    }
                    _ => unreachable!(),
                }
                let ident = opcode.ident;
                let input_name = stack_item.ident;
                Ok(quote_spanned! {span=>
                    bitcoin_cash_script::func::#ident(#input_name);
                    #script_ident.push(bitcoin_cash_script::TaggedOp {
                        src: #src.into(),
                        op: bitcoin_cash_script::Op::Code(#ident),
                        input_names: #input_names,
                        output_names: #output_names,
                    });
                })
            }
            Some(&opcode_type) => {
                self.run_other_opcode(src, opcode_type, opcode, input_names, output_names)
            }
            None => self.run_opcode_function(src, opcode),
        }
    }

    fn run_other_opcode(
        &mut self,
        src: String,
        opcode_type: OpcodeType,
        opcode: ir::Opcode,
        input_names: TokenStream,
        output_names: TokenStream,
    ) -> Result<TokenStream, Error> {
        let span = opcode.span;
        let behavior = opcode_type.behavior();
        if self.stack.len() < behavior.input_types.len() {
            return Err(error_empty_stack(opcode_type, span));
        }
        let input_items = self
            .stack
            .drain(self.stack.len() - behavior.input_types.len()..)
            .collect::<Vec<_>>();
        if let Some(input_names) = opcode.input_names {
            if input_items.len() != input_names.len() {
                return Err(Error::new(
                    span,
                    format!(
                        "Expected {} input names but got {}.",
                        input_items.len(),
                        input_names.len()
                    ),
                ));
            }
            for (input_item, input_name) in input_items.iter().zip(input_names) {
                if let ir::OpcodeInput::Ident(ident) = input_name {
                    if input_item.has_generated_name {
                        return Err(Error::new(
                            ident.span(),
                            format!(
                                "Expected named top stack item for `{}` but got unnamed.",
                                ident
                            ),
                        ));
                    } else if input_item.name != ident.to_string() {
                        let stack_names = self
                            .stack
                            .iter()
                            .chain(input_items.iter())
                            .map(|item| item.name.clone())
                            .collect::<Vec<_>>()
                            .join(", ");
                        return Err(Error::new(
                            ident.span(),
                            format!(
                                "Mismatched stack item name, expected `{}` but got `{}`. Current stack: [{}]",
                                input_item.name,
                                ident,
                                stack_names,
                            ),
                        ));
                    }
                }
            }
        }
        let mut output_idents = Vec::with_capacity(behavior.output_types.len());
        if let Some(output_names) = opcode.output_names {
            if output_names.len() != behavior.output_types.len() {
                return Err(Error::new(
                    opcode.span,
                    format!(
                        "Invalid number of output names. {:?} creates {} items, but {} defined.",
                        opcode_type,
                        behavior.output_types.len(),
                        output_names.len(),
                    ),
                ));
            }
            for ident in output_names {
                let new_ident = self.make_ident(ident.span());
                output_idents.push(new_ident.clone());
                self.stack.push(StackItem {
                    name: format!("{}", ident),
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
                        output_idents.push(new_ident.clone());
                        self.stack.push(StackItem {
                            ident: new_ident,
                            ..input_items[new_idx].clone()
                        });
                    }
                }
                None => {
                    for _ in 0..behavior.output_types.len() {
                        let ident = self.make_ident(span);
                        output_idents.push(ident.clone());
                        self.stack.push(StackItem {
                            name: ident.to_string(),
                            ident,
                            has_generated_name: true,
                            integer: None,
                        });
                    }
                }
            }
        }
        let output_let = if output_idents.len() == 1 {
            let output_ident = &output_idents[0];
            quote_spanned! {span=> let #output_ident = }
        } else {
            quote_spanned! {span=> let (#(#output_idents),*) = }
        };
        let input_idents = input_items.iter().map(|stack_item| {
            let mut ident = stack_item.ident.clone();
            ident.set_span(span);
            ident
        });
        let ident = opcode.ident;
        let script_ident = &self.script_ident;
        Ok(quote_spanned! {span=>
            #output_let bitcoin_cash_script::func::#ident( #(#input_idents.clone()),* );
            #script_ident.push(bitcoin_cash_script::TaggedOp {
                src: #src.into(),
                op: bitcoin_cash_script::Op::Code(#ident),
                input_names: #input_names,
                output_names: #output_names,
            });
        })
    }

    fn run_opcode_function(
        &mut self,
        src: String,
        opcode: ir::Opcode,
    ) -> Result<TokenStream, Error> {
        match opcode.ident.to_string().as_str() {
            "depth_of" => {
                if let Some(&[ir::OpcodeInput::Ident(ref ident)]) = opcode.input_names.as_deref() {
                    if let Some(depth) = self
                        .stack
                        .iter()
                        .rev()
                        .position(|stack| stack.name == ident.to_string())
                    {
                        let has_generated_name = opcode.output_names.is_none();
                        let span = opcode.span;
                        let output_names = Self::to_vec_str_tokens(opcode.output_names.as_deref());
                        let ident = if let Some(&[ref ident]) = opcode.output_names.as_deref() {
                            ident.clone()
                        } else {
                            self.make_ident(span)
                        };
                        let depth = depth as Integer;
                        self.stack.push(StackItem {
                            ident: ident.clone(),
                            name: ident.to_string(),
                            has_generated_name,
                            integer: Some(depth),
                        });
                        let script_ident = &self.script_ident;
                        Ok(quote_spanned! {span=>
                            let #ident = (#depth).to_data();
                            #script_ident.push(bitcoin_cash_script::TaggedOp {
                                src: #src.into(),
                                op: (#depth).to_pushop(),
                                input_names: None,
                                output_names: #output_names,
                            });
                        })
                    } else {
                        Err(Error::new(opcode.span, "Couldn't find stack item"))
                    }
                } else {
                    Err(Error::new(opcode.span, "Expected 1 parameter"))
                }
            }
            "transmute" => {
                if let Some(
                    &[ir::OpcodeInput::Ident(ref ident), ir::OpcodeInput::Expr(ref type_expr)],
                ) = opcode.input_names.as_deref()
                {
                    let span = opcode.span;
                    if let Some(&[ref out_ident]) = opcode.output_names.as_deref() {
                        if out_ident != ident {
                            return Err(Error::new(
                                opcode.span,
                                "Input and output name must be the same",
                            ));
                        }
                    };
                    let ident_name = ident.to_string();
                    let item = self
                        .stack
                        .iter()
                        .find(|item| item.name == ident_name)
                        .ok_or(Error::new(opcode.span, "Couldn't find stack item"))?;
                    let item_ident = &item.ident;
                    Ok(quote_spanned! {span=>
                        let #item_ident = <#type_expr as Default>::default().to_data();
                    })
                } else {
                    Err(Error::new(
                        opcode.span,
                        format!("Expected 1 parameter, got {:?}", opcode.input_names),
                    ))
                }
            }
            _ => Err(Error::new(opcode.span, "Unknown opcode/function")),
        }
    }

    fn to_vec_str_tokens(slice: Option<&[impl std::fmt::Display]>) -> TokenStream {
        match slice {
            Some(slice) => {
                let names = slice.iter().map(|ident| {
                    let name = format!("{}", ident);
                    quote! {#name.into()}
                });
                quote! {Some(vec![#(#names),*])}
            }
            None => quote! {None},
        }
    }

    fn pop_stack(
        stack: &mut Vec<StackItem>,
        opcode: OpcodeType,
        span: Span,
    ) -> Result<StackItem, Error> {
        stack.pop().ok_or(error_empty_stack(opcode, span))
    }

    fn pop(&mut self, opcode: OpcodeType, span: Span) -> Result<StackItem, Error> {
        Self::pop_stack(&mut self.stack, opcode, span)
    }

    fn push(&mut self, stack_item: StackItem) {
        self.stack.push(stack_item)
    }

    fn make_ident(&mut self, span: Span) -> syn::Ident {
        let ident = syn::Ident::new(&format!("__id_{}", self.n_ident), span);
        self.n_ident += 1;
        ident
    }
}

fn error_empty_stack(opcode: OpcodeType, span: Span) -> Error {
    Error::new(span, format!("{:?} fails due to empty stack", opcode))
}
