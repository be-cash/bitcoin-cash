use proc_macro2::Span;
use quote::ToTokens;

pub struct Script {
    pub input_struct: syn::Ident,
    pub attrs: Vec<syn::Attribute>,
    pub vis: syn::Visibility,
    pub sig: syn::Signature,
    pub inputs: Vec<ScriptInput>,
    pub stmts: Vec<Stmt>,
}

pub struct ScriptInput {
    pub ident: syn::Ident,
    pub ty: syn::Type,
}

#[derive(Clone)]
pub enum Stmt {
    Push(String, Push),
    Opcode(String, Opcode),
    ForLoop(ForLoop),
    RustIf(RustIf),
    ScriptIf(String, ScriptIf),
}

#[derive(Clone)]
pub struct Push {
    pub span: Span,
    pub expr: syn::Expr,
    pub output_name: Option<syn::Ident>,
}

#[derive(Clone)]
pub enum OpcodeInput {
    Ident(syn::Ident),
    Expr(syn::Expr),
}

#[derive(Clone)]
pub struct Opcode {
    pub span: Span,
    pub ident: syn::Ident,
    pub input_names: Option<Vec<OpcodeInput>>,
    pub output_names: Option<Vec<syn::Ident>>,
}

#[derive(Clone)]
pub struct ForLoop {
    pub span: Span,
    pub attrs: Vec<syn::Attribute>,
    pub pat: syn::Pat,
    pub expr: syn::Expr,
    pub stmts: Vec<Stmt>,
}

#[derive(Clone)]
pub struct RustIf {
    pub span: Span,
    pub attrs: Vec<syn::Attribute>,
    pub cond: syn::Expr,
    pub then_branch: Vec<Stmt>,
    pub else_branch: Option<Vec<Stmt>>,
}

#[derive(Clone)]
pub struct ScriptIf {
    pub if_opcode: Opcode,
    pub else_opcode: Option<Opcode>,
    pub endif_opcode: Opcode,
    pub then_stmts: Vec<Stmt>,
    pub else_stmts: Vec<Stmt>,
}

impl std::fmt::Display for OpcodeInput {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            OpcodeInput::Expr(expr) => write!(fmt, "{}", expr.to_token_stream()),
            OpcodeInput::Ident(ident) => write!(fmt, "{}", ident),
        }
    }
}

impl std::fmt::Debug for OpcodeInput {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(fmt, "{}", self)
    }
}
