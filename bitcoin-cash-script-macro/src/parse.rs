use proc_macro2::Span;
use quote::ToTokens;
use syn::spanned::Spanned;

use crate::ir;

pub fn parse_script(
    attrs: syn::AttributeArgs,
    func: syn::ItemFn,
) -> Result<ir::Script, syn::Error> {
    let input_struct = if attrs.len() == 1 {
        if let &[syn::NestedMeta::Meta(syn::Meta::Path(ref input_struct))] = attrs.as_slice() {
            if input_struct.segments.len() == 1 {
                Some(input_struct.segments[0].ident.clone())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };
    let input_struct = input_struct.ok_or(syn::Error::new(
        func.sig.span(),
        "Expected parameter for input struct",
    ))?;
    if let syn::ReturnType::Default = func.sig.output {
    } else {
        return Err(syn::Error::new(
            func.sig.span(),
            "A script's return type should be empty (no `->`)",
        ));
    }
    Ok(ir::Script {
        input_struct,
        attrs: func.attrs,
        vis: func.vis,
        inputs: parse_script_inputs(func.sig.span(), func.sig.inputs.iter())?,
        sig: func.sig,
        stmts: parse_stmts(func.block.stmts)?,
    })
}

fn parse_script_inputs<'a>(
    sig_span: Span,
    mut inputs: impl Iterator<Item = &'a syn::FnArg>,
) -> Result<Vec<ir::ScriptInput>, syn::Error> {
    if let Some(first) = inputs.next() {
        if let syn::FnArg::Typed(_) = first {
        } else {
            return Err(syn::Error::new(sig_span, "A script cannot be a method; the first parameter must contain the necessary constructor parameters."));
        }
    } else {
        return Err(syn::Error::new(
            sig_span,
            "A script must take at least one parameter, the constructor parameters.",
        ));
    }
    inputs
        .map(|input| {
            if let syn::FnArg::Typed(param) = input {
                parse_script_input(sig_span, param)
            } else {
                Err(syn::Error::new(sig_span, "Cannot have self parameters"))
            }
        })
        .collect()
}

fn parse_script_input(sig_span: Span, input: &syn::PatType) -> Result<ir::ScriptInput, syn::Error> {
    if let syn::Pat::Ident(pat_ident) = &*input.pat {
        return Ok(ir::ScriptInput {
            ident: pat_ident.ident.clone(),
            ty: (*input.ty).clone(),
        });
    } else {
        return Err(syn::Error::new(
            sig_span,
            "Currently, only plain identifiers are supported as script inputs.",
        ));
    }
}

fn parse_stmts(stmts: Vec<syn::Stmt>) -> Result<Vec<ir::Stmt>, syn::Error> {
    let mut result_stmts = Vec::new();
    for stmt in stmts {
        result_stmts.append(&mut parse_stmt(stmt)?);
    }
    let (stmts, last_opcode) = parse_op_if(&result_stmts)?;
    if let Some(last_opcode) = last_opcode {
        return unexpected_error(last_opcode.ident);
    }
    Ok(stmts)
}

fn parse_stmt(stmt: syn::Stmt) -> Result<Vec<ir::Stmt>, syn::Error> {
    match stmt {
        syn::Stmt::Local(local) => {
            let span = local.span();
            let src = format!("{}", local.to_token_stream());
            let (_, expr) = local.init.ok_or(syn::Error::new(
                span,
                format!("Expected opcode after `let`"),
            ))?;
            let outputs = match local.pat.clone() {
                syn::Pat::Ident(pat_ident)
                    if pat_ident.attrs.len() == 0
                        && pat_ident.by_ref.is_none()
                        && pat_ident.mutability.is_none()
                        && pat_ident.subpat.is_none() =>
                {
                    Ok(vec![pat_ident.ident])
                }
                syn::Pat::Tuple(pat_tuple) if pat_tuple.attrs.len() == 0 => pat_tuple
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
            match *expr {
                expr @ syn::Expr::Call(_) | expr @ syn::Expr::Path(_) => {
                    Ok(vec![ir::Stmt::Opcode(
                        src,
                        parse_opcode(expr, Some(outputs))?,
                    )])
                }
                expr => {
                    if outputs.len() != 1 {
                        return unexpected_error_msg(local.pat, "Expected single output");
                    }
                    Ok(vec![ir::Stmt::Push(
                        src,
                        ir::Push {
                            span: expr.span(),
                            expr,
                            output_name: Some(outputs[0].clone()),
                        },
                    )])
                }
            }
        }
        syn::Stmt::Expr(expr) | syn::Stmt::Semi(expr, _) => parse_stmt_expr(expr),
        syn::Stmt::Item(item) => unexpected_error(item),
    }
}

fn parse_stmt_expr(expr: syn::Expr) -> Result<Vec<ir::Stmt>, syn::Error> {
    match expr {
        syn::Expr::ForLoop(expr_for_loop) => Ok(vec![ir::Stmt::ForLoop(ir::ForLoop {
            span: expr_for_loop.span(),
            attrs: expr_for_loop.attrs,
            pat: expr_for_loop.pat,
            expr: *expr_for_loop.expr,
            stmts: parse_stmts(expr_for_loop.body.stmts)?,
        })]),
        syn::Expr::If(expr_if) => Ok(vec![ir::Stmt::RustIf(ir::RustIf {
            span: expr_if.span(),
            attrs: expr_if.attrs,
            cond: *expr_if.cond,
            then_branch: parse_stmts(expr_if.then_branch.stmts)?,
            else_branch: expr_if
                .else_branch
                .map(|(_, expr)| parse_stmt_expr(*expr))
                .map_or(Ok(None), |v| v.map(Some))?,
        })]),
        syn::Expr::Block(expr_block) => parse_stmts(expr_block.block.stmts),
        expr @ syn::Expr::Call(_) | expr @ syn::Expr::Path(_) => {
            let src = format!("{}", expr.to_token_stream());
            Ok(vec![ir::Stmt::Opcode(src, parse_opcode(expr, None)?)])
        }
        expr => {
            let src = format!("{}", expr.to_token_stream());
            Ok(vec![ir::Stmt::Push(
                src,
                ir::Push {
                    span: expr.span(),
                    expr,
                    output_name: None,
                },
            )])
        }
    }
}

fn parse_opcode(
    expr: syn::Expr,
    output_names: Option<Vec<syn::Ident>>,
) -> Result<ir::Opcode, syn::Error> {
    let span = expr.span();
    let (path, input_names) = match expr {
        syn::Expr::Call(expr_call) => {
            if expr_call.attrs.len() > 0 {
                return unexpected_error(&expr_call.attrs[0]);
            };
            let path = if let syn::Expr::Path(path) = *expr_call.func {
                path
            } else {
                return unexpected_error(expr_call.func);
            };
            let inputs = parse_opcode_inputs(expr_call.args)?;
            (path, Some(inputs))
        }
        syn::Expr::Path(expr_path) => (expr_path, None),
        expr_other => unexpected_error(expr_other)?,
    };
    let path = path.path;
    if path.segments.len() > 1 {
        return unexpected_error_msg(&path.segments[1], "Expected opcode.");
    }
    let ident = path.segments[0].ident.clone();
    Ok(ir::Opcode {
        span,
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
                /*if has_seen_expr {
                    return unexpected_error_msg(
                        ident,
                        "Push input expressions must all appear after the last stack identifier input. \
                         Use { expr } syntax to avoid this error, if this is a push input expression.",
                    );
                }*/
                let ident = ident.path.segments[0].ident.clone();
                inputs.push(ir::OpcodeInput::Ident(ident));
            }
            other_expr => {
                inputs.push(ir::OpcodeInput::Expr(other_expr));
            }
        }
    }
    Ok(inputs)
}

fn parse_op_if(stmts: &[ir::Stmt]) -> Result<(Vec<ir::Stmt>, Option<ir::Opcode>), syn::Error> {
    let mut i = 0;
    let mut new_stmts = Vec::new();
    while i < stmts.len() {
        let stmt = &stmts[i];
        if let ir::Stmt::Opcode(src, opcode) = stmt {
            match opcode.ident.to_string().as_str() {
                "OP_IF" | "OP_NOTIF" => {
                    i += 1;
                    let (then_stmts, last_opcode) = parse_op_if(&stmts[i..])?;
                    let last_opcode = last_opcode.ok_or(syn::Error::new(
                        opcode.span,
                        format!("{} has no corresponding OP_ELSE/OP_ENDIF", opcode.ident),
                    ))?;
                    i += then_stmts.len();
                    let (else_stmts, else_opcode, endif_opcode) =
                        if last_opcode.ident.to_string().as_str() == "OP_ELSE" {
                            i += 1;
                            let (else_stmts, endif_opcode) = parse_op_if(&stmts[i..])?;
                            let endif_opcode = endif_opcode.ok_or(syn::Error::new(
                                opcode.span,
                                format!("{} has no corresponding OP_ENDIF", last_opcode.ident),
                            ))?;
                            if endif_opcode.ident.to_string().as_str() != "OP_ENDIF" {
                                return unexpected_error_msg(
                                    &opcode.ident,
                                    "Multiple OP_ELSE not supported.",
                                );
                            }
                            i += else_stmts.len() + 1;
                            (else_stmts, Some(last_opcode), endif_opcode)
                        } else {
                            (vec![], None, last_opcode)
                        };
                    new_stmts.push(ir::Stmt::ScriptIf(
                        src.clone(),
                        ir::ScriptIf {
                            if_opcode: opcode.clone(),
                            else_opcode,
                            endif_opcode,
                            then_stmts,
                            else_stmts,
                        },
                    ));
                    continue;
                }
                "OP_ELSE" => {
                    return Ok((new_stmts, Some(opcode.clone())));
                }
                "OP_ENDIF" => {
                    return Ok((new_stmts, Some(opcode.clone())));
                }
                _ => {
                    i += 1;
                }
            }
        } else {
            i += 1;
        }
        new_stmts.push(stmt.clone());
    }
    Ok((new_stmts, None))
}

fn unexpected_error<T>(token: impl Spanned + ToTokens) -> Result<T, syn::Error> {
    Err(syn::Error::new(
        token.span(),
        format!("Unexpected `{}`.", token.to_token_stream()),
    ))
}

fn unexpected_error_msg<T>(token: impl Spanned + ToTokens, msg: &str) -> Result<T, syn::Error> {
    Err(syn::Error::new(
        token.span(),
        format!("Unexpected `{}`. {}", token.to_token_stream(), msg),
    ))
}
