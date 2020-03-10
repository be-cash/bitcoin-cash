use crate::Op;
use std::borrow::Cow;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaggedOp {
    pub op: Op,
    pub src: Cow<'static, str>,
    pub input_names: Option<Vec<Cow<'static, str>>>,
    pub output_names: Option<Vec<Cow<'static, str>>>,
}
