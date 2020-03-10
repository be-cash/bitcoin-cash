use proc_macro2::Span;
use quote::ToTokens;

pub struct Script {
    pub input_struct: syn::Ident,
    pub crate_ident: Option<syn::Ident>,
    pub script_variants: Vec<ScriptVariant>,
    pub attrs: Vec<syn::Attribute>,
    pub vis: syn::Visibility,
    pub sig: syn::Signature,
    pub inputs: Vec<ScriptInput>,
    pub stmts: Vec<Stmt>,
}

pub struct ScriptVariant {
    pub name: syn::Ident,
    pub predicate: VariantPredicate,
}

#[derive(Clone)]
pub struct VariantPredicate(pub Vec<VariantPredicateConjunction>);
#[derive(Clone)]
pub struct VariantPredicateConjunction(pub Vec<VariantPredicateAtom>);

#[derive(Clone)]
pub struct VariantPredicateAtom {
    pub var_name: String,
    pub is_positive: bool,
}

pub struct ScriptInput {
    pub ident: syn::Ident,
    pub ty: syn::Type,
    pub variants: Option<Vec<syn::Ident>>,
}

#[derive(Clone)]
pub enum Stmt {
    Push(String, PushStmt),
    Opcode(String, OpcodeStmt),
    ForLoop(ForLoopStmt),
    RustIf(RustIfStmt),
    ScriptIf(String, ScriptIfStmt),
}

#[derive(Clone)]
pub struct PushStmt {
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
pub struct OpcodeStmt {
    pub span: Span,
    pub ident: syn::Ident,
    pub input_names: Option<Vec<OpcodeInput>>,
    pub output_names: Option<Vec<syn::Ident>>,
}

#[derive(Clone)]
pub struct ForLoopStmt {
    pub span: Span,
    pub attrs: Vec<syn::Attribute>,
    pub pat: syn::Pat,
    pub expr: syn::Expr,
    pub stmts: Vec<Stmt>,
}

#[derive(Clone)]
pub struct RustIfStmt {
    pub span: Span,
    pub attrs: Vec<syn::Attribute>,
    pub cond: syn::Expr,
    pub then_branch: Vec<Stmt>,
    pub else_branch: Option<Vec<Stmt>>,
}

#[derive(Clone)]
pub struct ScriptIfStmt {
    pub if_opcode: OpcodeStmt,
    pub else_opcode: Option<OpcodeStmt>,
    pub endif_opcode: OpcodeStmt,
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

impl std::fmt::Display for VariantPredicateAtom {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            fmt,
            "{}{}",
            if self.is_positive { "" } else { "!" },
            self.var_name
        )
    }
}

impl std::fmt::Debug for OpcodeInput {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(fmt, "{}", self)
    }
}

impl VariantPredicate {
    pub fn holds(&self, instantiations: &[VariantPredicateAtom]) -> bool {
        if self.0.len() == 0 {
            return true;
        }
        for conjunction in self.0.iter() {
            let conjunction_holds = conjunction.0.iter().all(|atom| {
                instantiations.iter().all(|inst| {
                    inst.var_name != atom.var_name || inst.is_positive == atom.is_positive
                })
            });
            if conjunction_holds {
                return true;
            }
        }
        false
    }
}
