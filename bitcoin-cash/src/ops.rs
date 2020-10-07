use crate::{Op, Script, TaggedOp};
use std::borrow::Cow;

pub trait Ops {
    fn ops(&self) -> Cow<[TaggedOp]>;
}

pub struct TaggedScript<O: Ops> {
    tagged_ops: Vec<TaggedOp>,
    input_params: std::marker::PhantomData<O>,
}

impl<O: Ops> TaggedScript<O> {
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

impl<O: Ops> Ops for TaggedScript<O> {
    fn ops(&self) -> Cow<[TaggedOp]> {
        self.tagged_ops.as_slice().into()
    }
}

impl<O: Ops> Clone for TaggedScript<O> {
    fn clone(&self) -> Self {
        TaggedScript::new(self.tagged_ops.clone())
    }
}

impl<O: Ops> From<TaggedScript<O>> for Script {
    fn from(script: TaggedScript<O>) -> Self {
        Script::new(script.tagged_ops)
    }
}
