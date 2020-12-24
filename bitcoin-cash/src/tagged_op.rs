use crate::Op;
use std::borrow::Cow;

#[derive(Clone, Debug)]
pub struct TaggedOp {
    pub op: Op,
    pub src_file: Cow<'static, str>,
    pub src_line: u32,
    pub src_column: u32,
    pub src_code: Vec<(u32, Cow<'static, str>)>,
    pub pushed_names: Option<Vec<Option<Cow<'static, str>>>>,
    pub alt_pushed_names: Option<Vec<Option<Cow<'static, str>>>>,
}

impl TaggedOp {
    pub fn from_op(op: Op) -> Self {
        TaggedOp {
            op,
            src_file: "<unknown>".into(),
            src_line: 0,
            src_column: 0,
            src_code: vec![],
            pushed_names: None,
            alt_pushed_names: None,
        }
    }

    pub fn named(mut self, name: impl Into<Cow<'static, str>>) -> TaggedOp {
        self.pushed_names = Some(vec![Some(name.into())]);
        self
    }
}

impl PartialEq for TaggedOp {
    fn eq(&self, other: &Self) -> bool {
        self.op == other.op
    }
}
