use crate::{DataType, Op, Script, TaggedOp};
use std::borrow::Cow;

pub trait Ops {
    fn ops(&self) -> Cow<[TaggedOp]>;
}

pub trait InputScript: Ops {
    fn types(variant_name: Option<&str>) -> Vec<DataType>;
    fn names(variant_name: Option<&str>) -> &'static [&'static str];
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

    pub fn script_ops(&self) -> impl Iterator<Item = &Op> {
        self.tagged_ops.iter().map(|op| &op.op)
    }
}

impl<I: InputScript> Ops for TaggedScript<I> {
    fn ops(&self) -> Cow<[TaggedOp]> {
        self.tagged_ops.as_slice().into()
    }
}

impl<I: InputScript> Clone for TaggedScript<I> {
    fn clone(&self) -> Self {
        TaggedScript::new(self.tagged_ops.clone())
    }
}

impl<I: InputScript> From<TaggedScript<I>> for Script {
    fn from(script: TaggedScript<I>) -> Self {
        Script::new(script.tagged_ops)
    }
}
