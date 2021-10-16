use std::fmt;
use crate::eval::expression::Expression;
use crate::eval::{Environment, DynamicContext, EvalResult, Object};
use crate::parser::errors::ErrorCode;

#[derive(Clone, Debug)]
pub(crate) struct NodeParent {
}

impl NodeParent {
    pub(crate) fn boxed() -> Box<dyn Expression> {
        Box::new(NodeParent {})
    }
}

impl Expression for NodeParent {
    fn eval<'a>(&self, env: Box<Environment<'a>>, context: &DynamicContext) -> EvalResult<'a> {
        match &context.item {
            Object::Empty => Ok((env, Object::Empty)),
            Object::Node(rf) => {
                todo!()
            },
            _ => Err((ErrorCode::TODO, String::from("TODO")))
        }
    }

    fn predicate<'a>(&self, env: Box<Environment<'a>>, context: &DynamicContext, value: Object) -> EvalResult<'a> {
        todo!()
    }

    fn dump(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}