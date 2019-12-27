extern crate proc_macro;

use proc_macro2::*;
use syn::{Stmt, Result, Error};
use quote::{quote, quote_spanned};
use bitcoin_cash_script::{DataType, OpcodeType};

#[derive(Clone)]
struct StackItem {
    data_type: DataType,
    ident: Ident,
    lit: Option<syn::Lit>,
}

#[derive(Clone)]
struct ReplaceOpcodes {
    script_ident: proc_macro2::TokenStream,
    stack: Vec<StackItem>,
    alt_stack: Vec<StackItem>,
    n_ident: usize,

}

impl ReplaceOpcodes {
    fn evaluate_stmt(&mut self, stmt: &syn::Stmt) -> Result<TokenStream> {
        match stmt {
            Stmt::Expr(expr) => {
                self.evaluate_expr(expr)
            }
            Stmt::Semi(expr, semi) => {
                let new_expr = self.evaluate_expr(expr)?;
                Ok(quote! {
                    #new_expr #semi
                })
            }
            _ => panic!("Stmt")
        }
    }

    fn evaluate_expr(&mut self, expr: &syn::Expr) -> Result<TokenStream> {
        use syn::Expr::*;
        let script_ident = self.script_ident.clone();
        match expr {
            Array(_expr_array) => {panic!("Array")}
            Assign(_expr_assign) => {panic!("Assign")}
            AssignOp(_expr_assign_op) => {panic!("AssignOp")}
            Async(_expr_async) => {panic!("Async")}
            Await(_expr_await) => {panic!("Await")}
            Binary(_expr_binary) => {panic!("Binary")}
            Block(_expr_block) => {panic!("Block")}
            Box(_expr_box) => {panic!("Box")}
            Break(_expr_break) => {panic!("Break")}
            Call(_expr_call) => {panic!("Call")}
            Cast(_expr_cast) => {panic!("Cast")}
            Closure(_expr_closure) => {panic!("Closure")}
            Continue(_expr_continue) => {panic!("Continue")}
            Field(_expr_field) => {panic!("Field")}
            ForLoop(_expr_for_loop) => {panic!("ForLoop")}
            Group(_expr_group) => {panic!("Group")}
            If(_expr_if) => {panic!("If")}
            Index(_expr_index) => {panic!("Index")}
            Let(_expr_let) => {panic!("Let")}
            Lit(expr_lit) => {
                let lit = &expr_lit.lit;
                match lit {
                    syn::Lit::ByteStr(lit) => {
                        let span = lit.span();
                        let new_ident = self.make_ident(span);
                        self.stack.push(StackItem {
                            data_type: DataType::ByteArray(Some(lit.value().len())),
                            ident: new_ident.clone(),
                            lit: Some(expr_lit.lit.clone()),
                        });
                        Ok(quote_spanned!{span=>
                            let #new_ident = bitcoin_cash_script::BitcoinByteArray(#lit.to_vec());
                            #script_ident.push(bitcoin_cash_script::Op::PushByteArray(#lit.to_vec()));
                        })
                    }
                    syn::Lit::Int(lit) => {
                        let span = lit.span();
                        let new_ident = self.make_ident(span);
                        self.stack.push(StackItem {
                            data_type: DataType::Integer,
                            ident: new_ident.clone(),
                            lit: Some(expr_lit.lit.clone()),
                        });
                        Ok(quote_spanned!{span=>
                            let #new_ident = bitcoin_cash_script::BitcoinInteger(#lit);
                            #script_ident.push(bitcoin_cash_script::Op::PushInteger(#lit));
                        })
                    }
                    syn::Lit::Bool(lit) => {
                        let span = lit.span;
                        let new_ident = self.make_ident(span);
                        self.stack.push(StackItem {
                            data_type: DataType::Integer,
                            ident: new_ident.clone(),
                            lit: Some(expr_lit.lit.clone()),
                        });
                        Ok(quote_spanned!{span=>
                            let #new_ident = bitcoin_cash_script::BitcoinBoolean(#lit);
                            #script_ident.push(bitcoin_cash_script::Op::PushBoolean(#lit));
                        })
                    }
                    _ => unimplemented!(),
                }
            }
            Loop(_expr_loop) => {panic!("Loop")}
            Macro(_expr_macro) => {panic!("Macro")}
            Match(_expr_match) => {panic!("Match")}
            MethodCall(_expr_method_call) => {panic!("MethodCall")}
            Paren(_expr_paren) => {panic!("Paren")}
            Path(expr_path) => {
                if expr_path.path.segments.len() != 1 {
                    return Ok(quote! { #expr_path });
                }
                for path_segment in expr_path.path.segments.iter() {
                    let ident = &path_segment.ident;
                    let ident_name = format!("{}", ident);
                    match bitcoin_cash_script::MAP_NAME_TO_ENUM.get(&ident_name) {
                        Some(opcode) => return self.evaluate_opcode(ident.clone(), *opcode, ident.span()),
                        None => return Ok(quote! { #expr_path }),
                    }
                }
                unreachable!()
            }
            Range(_expr_range) => {panic!("Range")}
            Reference(_expr_reference) => {panic!("Reference")}
            Repeat(_expr_repeat) => {panic!("Repeat")}
            Return(_expr_return) => {panic!("Return")}
            Struct(_expr_struct) => {panic!("Struct")}
            Try(_expr_try) => {panic!("Try")}
            TryBlock(_expr_try_block) => {panic!("TryBlock")}
            Tuple(_expr_tuple) => {panic!("Tuple")}
            Type(_expr_type) => {panic!("Type")}
            Unary(_expr_unary) => {panic!("Unary")}
            Unsafe(_expr_unsafe) => {panic!("Unsafe")}
            Verbatim(_token_stream) => {panic!("Verbatim")}
            While(_expr_while) => {panic!("While")}
            Yield(_expr_yield) => {panic!("Yield")}
            _ => {unimplemented!()}
        }
    }

    fn pop_stack(stack: &mut Vec<StackItem>, opcode: OpcodeType, span: Span) -> Result<StackItem> {
        match stack.pop() {
            Some(item) => Ok(item),
            None => Err(error_empty_stack(opcode, span)),
        }
    }

    fn pop(&mut self, opcode: OpcodeType, span: Span) -> Result<StackItem> {
        Self::pop_stack(&mut self.stack, opcode, span)
    }

    fn evaluate_opcode(&mut self, ident: Ident, opcode: OpcodeType, span: Span,) -> Result<TokenStream> {
        use OpcodeType::*;
        let script_ident = self.script_ident.clone();
        match opcode {
            OP_TOALTSTACK => {
                let stack_item = self.pop(opcode, span)?;
                self.alt_stack.push(stack_item);
                return Ok(quote! {span=> 
                    #script_ident.push(bitcoin_cash_script::Op::Code(#ident));
                });
            }
            OP_FROMALTSTACK => {
                self.stack.push(Self::pop_stack(&mut self.alt_stack, opcode, span)?);
                return Ok(quote! {span=>
                    #script_ident.push(bitcoin_cash_script::Op::Code(#ident));
                });
            }
            OP_PICK | OP_ROLL => {
                let stack_item = self.pop(opcode, span)?;
                if stack_item.data_type != DataType::Boolean {
                    Err(Error::new(span, format!("{:?} fails due to invalid indexing data type {:?}", opcode, stack_item.data_type)))?
                }
                let lit = match stack_item.lit {
                    Some(syn::Lit::Int(lit)) => lit,
                    _ => Err(Error::new(span, format!("{:?} expects an integer literal as top stack item", opcode)))?,
                };
                let item_idx = lit.base10_parse::<usize>()?;
                if item_idx >= self.stack.len() {
                    Err(Error::new(span, format!("{:?} tried to access {} items deep, but stack only has {} items", opcode, item_idx, self.stack.len())))?
                }
                match opcode {
                    OP_PICK => {
                        self.stack.push(self.stack[self.stack.len() - item_idx - 1].clone());   
                    }
                    OP_ROLL => {
                        let rolled_stack_item = self.stack.remove(self.stack.len() - item_idx - 1);
                        self.stack.push(rolled_stack_item);
                    }
                    _ => unreachable!(),
                }
                return Ok(quote_spanned! {span=>
                    #script_ident.push(bitcoin_cash_script::Op::Code(#ident));
                })
            }
            _ => {}
        }
        let behavior = opcode.behavior();
        match behavior.output_order {
            Some(new_order) => {
                let old_items = self.stack
                    .drain(self.stack.len() - behavior.input_types.len()..)
                    .collect::<Vec<_>>();
                let mut output_idents = Vec::with_capacity(new_order.len());
                for new_idx in new_order {
                    let new_ident = self.make_ident(span);
                    output_idents.push(new_ident.clone());
                    let old_item = &old_items[*new_idx];
                    self.stack.push(StackItem {
                        data_type: old_item.data_type,
                        ident: new_ident,
                        lit: None,
                    });
                }
                let input_params = old_items.iter()
                    .map(|stack_item| stack_item.ident.clone());
                let output_let = if new_order.len() == 0 {
                    quote! {}
                } else if new_order.len() == 1 {
                    let output_ident = &output_idents[0];
                    quote_spanned! {span=> let #output_ident = }
                } else {
                    quote_spanned! {span=> let (#(#output_idents),*) = }
                };
                return Ok(quote_spanned! {span=>
                    #output_let bitcoin_cash_script::func::#ident( #(#input_params),* );
                    #script_ident.push(bitcoin_cash_script::Op::Code(#ident));
                })
            }
            None => {
                let mut tokens = Vec::<TokenStream>::new();
                if behavior.input_types.len() > self.stack.len() {
                    Err(Error::new(span, format!("{:?} needs {} items, but stack only has {} items", opcode, behavior.input_types.len(), self.stack.len())))?
                }
                let input_items = &self.stack.drain(self.stack.len() - behavior.input_types.len()..).collect::<Vec<_>>();
                let input_params = input_items.iter()
                    .map(|stack_item| stack_item.ident.clone());
                let mut output_idents = Vec::with_capacity(behavior.output_types.len());
                for output_type in behavior.output_types {
                    let new_ident = self.make_ident(span);
                    output_idents.push(new_ident.clone());
                    self.stack.push(StackItem {
                        data_type: *output_type,
                        ident: new_ident,
                        lit: None,
                    })
                }
                let output_let = if behavior.output_types.len() == 0 {
                    quote! {}
                } else if behavior.output_types.len() == 1 {
                    let output = &output_idents[0];
                    quote_spanned! {span=> let #output = }
                } else {
                    quote_spanned! {span=> let (#(#output_idents),*) = }
                };
                tokens.push(quote_spanned! {span=>
                    #output_let bitcoin_cash_script::func::#ident( #(#input_params),* );
                    #script_ident.push(bitcoin_cash_script::Op::Code(#ident));
                });
                return Ok(tokens.into_iter().collect())
            }
        };
    }

    fn make_ident(&mut self, span: Span) -> Ident {
        let ident = Ident::new(&format!("__id_{}", self.n_ident), span);
        self.n_ident += 1;
        ident
    }
}

fn error_empty_stack(opcode: OpcodeType, span: Span) -> syn::Error {
    syn::Error::new(span, format!("{:?} fails due to empty stack", opcode))
}

#[proc_macro_attribute]
pub fn script(_attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);
    let fn_attrs = input.attrs;
    let fn_vis = input.vis;
    let fn_sig = input.sig;
    let script_ident = quote! {__script_vec};
    let mut new_stmts: Vec<TokenStream> = Vec::new();
    let mut replace_opcodes = ReplaceOpcodes {
        script_ident: script_ident.clone(),
        stack: vec![],
        alt_stack: vec![],
        n_ident: 0,
    };
    for stmt in &input.block.stmts {
        match replace_opcodes.evaluate_stmt(stmt) {
            Ok(tokens) => new_stmts.push(tokens),
            Err(err) => new_stmts.push(err.to_compile_error()),
        }
    }

    let result = quote! {
        #[allow(redundant_semicolon)]
        #(#fn_attrs)*
        #fn_vis #fn_sig {
            use bitcoin_cash_script::OpcodeType::*;
            let mut #script_ident = Vec::new();
            #(#new_stmts)*
            return #script_ident;
        }
        
    };
    result.into()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
