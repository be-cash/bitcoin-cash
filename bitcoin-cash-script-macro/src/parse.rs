use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use std::collections::HashMap;
use syn::spanned::Spanned;

use crate::ir;

pub fn parse_script(
    attrs: syn::AttributeArgs,
    func: syn::ItemFn,
) -> Result<ir::Script, syn::Error> {
    let (input_struct, crate_ident, script_variants) =
        parse_attrs(attrs).map_err(|msg| syn::Error::new(func.sig.span(), &msg))?;
    if let syn::ReturnType::Default = func.sig.output {
    } else {
        return Err(syn::Error::new(
            func.sig.span(),
            "A script's return type should be empty (no `->`)",
        ));
    }
    let docs = parse_docs(&func.attrs, &input_struct, &script_variants);
    let (inputs, param_type) = parse_script_inputs(func.sig.span(), func.sig.inputs.iter())?;
    Ok(ir::Script {
        input_struct,
        crate_ident,
        script_variants,
        attrs: func.attrs,
        vis: func.vis,
        param_type,
        inputs,
        sig: func.sig,
        stmts: parse_stmts(func.block.stmts)?,
        docs,
    })
}

fn parse_script_inputs<'a>(
    sig_span: Span,
    mut inputs: impl Iterator<Item = &'a syn::FnArg>,
) -> Result<(Vec<ir::ScriptInput>, Box<syn::Type>), syn::Error> {
    let param_type = if let Some(first) = inputs.next() {
        if let syn::FnArg::Typed(param_type) = first {
            param_type.ty.clone()
        } else {
            return Err(syn::Error::new(sig_span, "A script cannot be a method; the first parameter must contain the necessary constructor parameters."));
        }
    } else {
        return Err(syn::Error::new(
            sig_span,
            "A script must take at least one parameter, the constructor parameters.",
        ));
    };
    Ok((
        inputs
            .map(|input| {
                if let syn::FnArg::Typed(param) = input {
                    parse_script_input(sig_span, param)
                } else {
                    Err(syn::Error::new(sig_span, "Cannot have self parameters"))
                }
            })
            .collect::<Result<_, _>>()?,
        param_type,
    ))
}

fn single_path(path: &syn::Path) -> Result<syn::Ident, ()> {
    if path.segments.len() == 1 {
        Ok(path.segments[0].ident.clone())
    } else {
        Err(())
    }
}

fn parse_attrs(
    attrs: syn::AttributeArgs,
) -> Result<(syn::Ident, Option<syn::Ident>, Vec<ir::ScriptVariant>), String> {
    if attrs.is_empty() {
        return Err("Must provide at least input struct name".into());
    }
    let input_struct = if let syn::NestedMeta::Meta(syn::Meta::Path(input_struct)) = &attrs[0] {
        single_path(input_struct)
            .map_err(|_| "Input struct name cannot have a module".to_string())?
    } else {
        return Err("Invalid input struct name".into());
    };
    let mut variants = Vec::new();
    let mut crate_name = None;
    for attr in attrs.into_iter().skip(1) {
        if let syn::NestedMeta::Meta(syn::Meta::NameValue(variant)) = attr {
            let name = single_path(&variant.path).map_err(|_| "Variant cannot have a module")?;
            if &name.to_string() == "crate" {
                crate_name = Some(syn::Ident::new(
                    &parse_string_lit(&variant.lit)
                        .ok_or_else(|| "Invalid crate name, must be string.".to_string())?,
                    name.span(),
                ));
            } else {
                variants.push(ir::ScriptVariant {
                    name,
                    predicate: parse_predicate(&variant.lit)?,
                })
            }
        } else {
            return Err(
                "Invalid variant, must be of form `VariantName=\"conditions\"`".to_string(),
            );
        }
    }
    Ok((input_struct, crate_name, variants))
}

fn parse_docs(
    attrs: &[syn::Attribute],
    input_struct: &syn::Ident,
    variants: &[ir::ScriptVariant],
) -> ir::ScriptDocs {
    let mut input_struct_docs = Vec::new();
    let mut variant_docs = HashMap::new();
    let attrs = attrs.into_iter();
    let mut found_input_struct = false;
    let mut current_variant_name = None;
    'attr: for attr in attrs {
        if let Some(doc) = parse_doc_attribute(attr) {
            let doc_trimmed = doc.trim();
            let prefix = "# ";
            if doc_trimmed.starts_with(prefix) && *input_struct == doc_trimmed[prefix.len()..] {
                found_input_struct = true;
                continue;
            }
            let prefix = "## ";
            if found_input_struct && doc_trimmed.starts_with(prefix) {
                for variant in variants {
                    if variant.name == doc_trimmed[prefix.len()..] {
                        current_variant_name = Some(&variant.name);
                        variant_docs.insert(variant.name.clone(), Vec::new());
                        continue 'attr;
                    }
                }
            }
            if let Some(current_variant_name) = current_variant_name {
                variant_docs
                    .get_mut(current_variant_name)
                    .unwrap()
                    .push(doc);
            } else if found_input_struct {
                input_struct_docs.push(doc);
            }
        }
    }
    ir::ScriptDocs {
        input_struct: input_struct_docs,
        variants: variant_docs,
    }
}

fn parse_doc_attribute(attr: &syn::Attribute) -> Option<String> {
    let meta = attr.parse_meta().ok()?;
    let meta = match meta {
        syn::Meta::NameValue(meta) => meta,
        _ => return None,
    };
    if single_path(&meta.path).ok()? != "doc" {
        return None;
    }
    parse_string_lit(&meta.lit)
}

fn parse_string_lit(lit: &syn::Lit) -> Option<String> {
    if let syn::Lit::Str(predicate_str) = lit {
        Some(predicate_str.value())
    } else {
        None
    }
}

fn parse_predicate(predicate_lit: &syn::Lit) -> Result<ir::VariantPredicate, String> {
    let predicate_str = parse_string_lit(predicate_lit)
        .ok_or_else(|| "Invalid predicate literal, must be string.".to_string())?;
    Ok(ir::VariantPredicate(
        predicate_str
            .split("||")
            .map(|conjunction| {
                Ok(ir::VariantPredicateConjunction(
                    conjunction
                        .split("&&")
                        .map(|mut predicate_atom| {
                            if predicate_atom.is_empty() {
                                return Err("Empty predicate name".to_string());
                            }
                            let is_negated = &predicate_atom[..1] == "!";
                            if is_negated {
                                predicate_atom = &predicate_atom[1..];
                            }
                            if predicate_atom.is_empty() {
                                return Err("Empty predicate name".to_string());
                            }
                            if !predicate_atom
                                .chars()
                                .all(|c| c.is_alphanumeric() || c == '_')
                            {
                                Err("Not a valid predicate name".to_string())
                            } else {
                                Ok(ir::VariantPredicateAtom {
                                    var_name: predicate_atom.to_string(),
                                    is_positive: !is_negated,
                                })
                            }
                        })
                        .collect::<Result<Vec<_>, String>>()?,
                ))
            })
            .collect::<Result<Vec<_>, String>>()?,
    ))
}

fn parse_script_input(sig_span: Span, input: &syn::PatType) -> Result<ir::ScriptInput, syn::Error> {
    let mut variants = None;
    let mut attrs = Vec::new();
    for attr in &input.attrs {
        let err = |s| syn::Error::new(sig_span, s);
        match single_path(&attr.path) {
            Ok(ident) if &ident.to_string() == "variant" => ident,
            _ => {
                attrs.push(attr.clone());
                continue;
            }
        };
        match attr.parse_meta()? {
            syn::Meta::List(meta_list) => {
                variants = Some(
                    meta_list
                        .nested
                        .iter()
                        .map(|nested_meta| {
                            if let syn::NestedMeta::Meta(syn::Meta::Path(path)) = nested_meta {
                                single_path(path).map_err(|_| err("Variant cannot have a module"))
                            } else {
                                Err(err("Invalid variant"))
                            }
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                );
            }
            syn::Meta::Path(path) => {
                variants = Some(vec![
                    single_path(&path).map_err(|_| err("Variant cannot have a module"))?
                ])
            }
            _ => return Err(err("Invalid variant")),
        }
    }
    if let syn::Pat::Ident(pat_ident) = &*input.pat {
        Ok(ir::ScriptInput {
            token_stream: input.to_token_stream(),
            ident: pat_ident.ident.clone(),
            ty: (*input.ty).clone(),
            variants,
            attrs,
        })
    } else {
        Err(syn::Error::new(
            sig_span,
            "Only plain identifiers are supported as script inputs.",
        ))
    }
}

fn parse_stmts(stmts: Vec<syn::Stmt>) -> Result<Vec<ir::TaggedStmt>, syn::Error> {
    let mut result_stmts = Vec::new();
    for stmt in stmts {
        result_stmts.append(&mut parse_stmt(stmt)?);
    }
    parse_op_if(result_stmts)
}

fn parse_stmt(stmt: syn::Stmt) -> Result<Vec<ir::TaggedStmt>, syn::Error> {
    let token_stream = stmt.to_token_stream();
    match stmt {
        syn::Stmt::Local(local) => {
            let span = local.span();
            let (_, expr) = local
                .init
                .ok_or_else(|| syn::Error::new(span, "Expected opcode after `let`"))?;
            let outputs = match local.pat.clone() {
                syn::Pat::Ident(pat_ident)
                    if pat_ident.attrs.is_empty()
                        && pat_ident.by_ref.is_none()
                        && pat_ident.mutability.is_none()
                        && pat_ident.subpat.is_none() =>
                {
                    Ok(vec![pat_ident.ident])
                }
                syn::Pat::Tuple(pat_tuple) if pat_tuple.attrs.is_empty() => pat_tuple
                    .elems
                    .into_iter()
                    .map(|pat| {
                        if let syn::Pat::Ident(pat_ident) = pat {
                            Ok(pat_ident.ident)
                        } else {
                            Err(pat)
                        }
                    })
                    .collect::<Result<Vec<syn::Ident>, syn::Pat>>(),
                pat_other => Err(pat_other),
            };
            let outputs = match outputs {
                Ok(outputs) => outputs,
                Err(pat) => {
                    return unexpected_error_msg(
                        pat,
                        "Expected `let x = ...` or `let (x, y) = ...` or similar.",
                    )
                }
            };
            Ok(vec![ir::TaggedStmt {
                token_stream,
                stmt: match *expr {
                    expr @ syn::Expr::Call(_) | expr @ syn::Expr::Path(_) => {
                        ir::Stmt::Opcode(parse_opcode(local.pat.span(), expr, Some(outputs))?)
                    }
                    expr => {
                        if outputs.len() != 1 {
                            return unexpected_error_msg(local.pat, "Expected single output");
                        }
                        ir::Stmt::Push(ir::PushStmt {
                            span: expr.span(),
                            expr,
                            output_name: Some(outputs[0].clone()),
                        })
                    }
                },
            }])
        }
        syn::Stmt::Expr(expr) | syn::Stmt::Semi(expr, _) => parse_stmt_expr(expr),
        syn::Stmt::Item(item) => unexpected_error_msg(item, "Unexpected Item"),
    }
}

fn parse_stmt_expr(expr: syn::Expr) -> Result<Vec<ir::TaggedStmt>, syn::Error> {
    let token_stream = expr.to_token_stream();
    match expr {
        syn::Expr::ForLoop(expr_for_loop) => Ok(vec![ir::TaggedStmt {
            token_stream,
            stmt: ir::Stmt::ForLoop(ir::ForLoopStmt {
                span: expr_for_loop.span(),
                attrs: expr_for_loop.attrs,
                pat: expr_for_loop.pat,
                expr: *expr_for_loop.expr,
                stmts: parse_stmts(expr_for_loop.body.stmts)?,
            }),
        }]),
        syn::Expr::If(expr_if) => Ok(vec![ir::TaggedStmt {
            token_stream,
            stmt: ir::Stmt::RustIf(ir::RustIfStmt {
                span: expr_if.span(),
                attrs: expr_if.attrs,
                cond: *expr_if.cond,
                then_branch: parse_stmts(expr_if.then_branch.stmts)?,
                else_branch: expr_if
                    .else_branch
                    .map(|(_, expr)| parse_stmt_expr(*expr))
                    .map_or(Ok(None), |v| v.map(Some))?,
            }),
        }]),
        syn::Expr::Block(expr_block) => parse_stmts(expr_block.block.stmts),
        expr @ syn::Expr::Call(_) | expr @ syn::Expr::Path(_) => Ok(vec![ir::TaggedStmt {
            token_stream,
            stmt: ir::Stmt::Opcode(parse_opcode(expr.span(), expr, None)?),
        }]),
        expr => Ok(vec![ir::TaggedStmt {
            token_stream,
            stmt: ir::Stmt::Push(ir::PushStmt {
                span: expr.span(),
                expr,
                output_name: None,
            }),
        }]),
    }
}

fn parse_opcode(
    outputs_span: Span,
    expr: syn::Expr,
    output_names: Option<Vec<syn::Ident>>,
) -> Result<ir::OpcodeStmt, syn::Error> {
    let span = expr.span();
    let (path, input_names) = match expr {
        syn::Expr::Call(expr_call) => {
            if !expr_call.attrs.is_empty() {
                return unexpected_error_msg(&expr_call.attrs[0], "Unexpected attribute");
            };
            let path = if let syn::Expr::Path(path) = *expr_call.func {
                path
            } else {
                return unexpected_error_msg(expr_call.func, "Expected path");
            };
            let inputs = parse_opcode_inputs(expr_call.args)?;
            (path, Some(inputs))
        }
        syn::Expr::Path(expr_path) => (expr_path, None),
        expr_other => unexpected_error_msg(expr_other, "Expected call or path")?,
    };
    let path = path.path;
    if path.segments.len() > 1 {
        return unexpected_error_msg(&path.segments[1], "Expected opcode.");
    }
    let ident = path.segments[0].ident.clone();
    Ok(ir::OpcodeStmt {
        outputs_span,
        expr_span: span,
        ident,
        input_names,
        output_names,
    })
}

fn parse_opcode_inputs(
    args: impl IntoIterator<Item = syn::Expr>,
) -> Result<Vec<ir::OpcodeInput>, syn::Error> {
    let mut inputs = Vec::new();
    for arg in args {
        match arg {
            syn::Expr::Path(ident) if ident.path.segments.len() == 1 => {
                let ident = ident.path.segments[0].ident.clone();
                inputs.push(ir::OpcodeInput::Ident(ident));
            }
            other_expr => {
                inputs.push(ir::OpcodeInput::Expr(Box::new(other_expr)));
            }
        }
    }
    Ok(inputs)
}

fn parse_op_if(stmts: Vec<ir::TaggedStmt>) -> Result<Vec<ir::TaggedStmt>, syn::Error> {
    let mut new_stmts = Vec::new();
    let mut if_stack = Vec::new();
    let mut is_then = true;
    struct If {
        if_opcode: ir::OpcodeStmt,
        if_token_stream: TokenStream,
        else_opcode: Option<ir::OpcodeStmt>,
        else_token_stream: Option<TokenStream>,
        then_stmts: Vec<ir::TaggedStmt>,
        else_stmts: Vec<ir::TaggedStmt>,
    }
    for stmt in stmts {
        let stmt = match stmt {
            ir::TaggedStmt {
                token_stream,
                stmt: ir::Stmt::Opcode(opcode),
            } => match opcode.ident.to_string().as_str() {
                "OP_IF" | "OP_NOTIF" => {
                    if_stack.push(If {
                        if_opcode: opcode,
                        if_token_stream: token_stream,
                        else_opcode: None,
                        else_token_stream: None,
                        then_stmts: vec![],
                        else_stmts: vec![],
                    });
                    is_then = true;
                    continue;
                }
                "OP_ELSE" => {
                    is_then = false;
                    let top_if = if_stack.last_mut().ok_or_else(|| {
                        syn::Error::new(opcode.expr_span, "No previous OP_IF found.")
                    })?;
                    top_if.else_opcode = Some(opcode);
                    top_if.else_token_stream = Some(token_stream);
                    continue;
                }
                "OP_ENDIF" => {
                    let top_if = if_stack.pop().ok_or_else(|| {
                        syn::Error::new(opcode.expr_span, "No previous OP_IF found.")
                    })?;
                    ir::TaggedStmt {
                        token_stream: quote! {},
                        stmt: ir::Stmt::ScriptIf(ir::ScriptIfStmt {
                            if_token_stream: top_if.if_token_stream,
                            if_opcode: top_if.if_opcode,
                            else_token_stream: top_if.else_token_stream,
                            else_opcode: top_if.else_opcode,
                            endif_opcode: opcode,
                            endif_token_stream: token_stream,
                            then_stmts: top_if.then_stmts,
                            else_stmts: top_if.else_stmts,
                        }),
                    }
                }
                _ => ir::TaggedStmt {
                    token_stream,
                    stmt: ir::Stmt::Opcode(opcode),
                },
            },
            stmt => stmt,
        };
        if if_stack.is_empty() {
            new_stmts.push(stmt);
        } else {
            let top_if = if_stack.last_mut().unwrap();
            if is_then {
                top_if.then_stmts.push(stmt);
            } else {
                top_if.else_stmts.push(stmt);
            }
        }
    }
    if let Some(last_if) = if_stack.last() {
        Err(syn::Error::new(
            last_if.if_opcode.expr_span,
            "Unclosed OP_IF.",
        ))
    } else {
        Ok(new_stmts)
    }
}

fn unexpected_error_msg<T>(token: impl Spanned + ToTokens, msg: &str) -> Result<T, syn::Error> {
    Err(syn::Error::new(
        token.span(),
        format!("Unexpected `{}`. {}", token.to_token_stream(), msg),
    ))
}
