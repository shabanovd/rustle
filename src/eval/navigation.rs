use std::fmt;
use crate::eval::expression::Expression;
use crate::eval::{Environment, DynamicContext, EvalResult, Object};
use crate::eval::helpers::relax;
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
                if let Some(node) = rf.parent() {
                    Ok((env, Object::Node(node)))
                } else {
                    Ok((env, Object::Empty))
                }
            },
            Object::Sequence(items) => {
                let mut result = Vec::with_capacity(items.len());
                for item in items {
                    match item {
                        Object::Empty => {},
                        Object::Node(rf) => {
                            if let Some(node) = rf.parent() {
                                result.push(Object::Node(node));
                            }
                        },
                        _ => return Err((ErrorCode::TODO, String::from("TODO")))
                    }
                }
                relax(env, result)
            }
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