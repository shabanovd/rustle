use crate::eval::{Environment, DynamicContext, EvalResult, Object, Node};
use dyn_clone::DynClone;

pub trait Expression: DynClone {
    fn eval<'a>(&self, env: Box<Environment<'a>>, context: &DynamicContext) -> EvalResult<'a>;

    fn predicate<'a>(&self, env: Box<Environment<'a>>, context: &DynamicContext, value: Object) -> EvalResult<'a>;

    fn debug(&self) -> String;
}

dyn_clone::clone_trait_object!(Expression);

pub(crate) trait NodeTest: DynClone {
    fn test_node(&self, node: &Node) -> bool;
}

dyn_clone::clone_trait_object!(NodeTest);