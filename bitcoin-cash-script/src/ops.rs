use crate::{DataType, Op, TaggedOp};
use std::borrow::Cow;

pub trait Ops {
    fn ops(&self) -> Cow<[Op]>;
}

pub trait InputScript: Ops {
    fn types() -> Vec<DataType>;
    fn names() -> &'static [&'static str];
}

pub struct TaggedScript<I: InputScript> {
    tagged_ops: Vec<TaggedOp>,
    input_params: std::marker::PhantomData<I>,
}

impl<I: InputScript> TaggedScript<I> {
    pub fn new(tagged_ops: Vec<TaggedOp>) -> Self {
        TaggedScript {
            tagged_ops,
            input_params: std::marker::PhantomData,
        }
    }

    pub fn tagged_ops(&self) -> &[TaggedOp] {
        &self.tagged_ops
    }
}

impl<I: InputScript> Ops for TaggedScript<I> {
    fn ops(&self) -> Cow<[Op]> {
        self.tagged_ops.iter().map(|op| op.op.clone()).collect()
    }
}

#[derive(Clone, Debug)]
pub struct TaggedScriptOps {
    tagged_ops: Vec<TaggedOp>,
    ops: Vec<Op>,
}

impl TaggedScriptOps {
    pub fn new(tagged_ops: Vec<TaggedOp>) -> Self {
        let ops = tagged_ops.iter().map(|op| op.op.clone()).collect();
        TaggedScriptOps { tagged_ops, ops }
    }

    pub fn tagged_ops(&self) -> &[TaggedOp] {
        &self.tagged_ops
    }
}

impl Ops for TaggedScriptOps {
    fn ops(&self) -> Cow<[Op]> {
        self.ops.as_slice().into()
    }
}

impl<I: InputScript> Into<TaggedScriptOps> for TaggedScript<I> {
    fn into(self) -> TaggedScriptOps {
        TaggedScriptOps::new(self.tagged_ops)
    }
}

impl<I: InputScript> Clone for TaggedScript<I> {
    fn clone(&self) -> Self {
        TaggedScript::new(self.tagged_ops().to_vec())
    }
} 
