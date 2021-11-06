use std::fmt::Debug;
use dyn_clone::DynClone;

use crate::eval::{Environment, DynamicContext, EvalResult, Object};
use crate::tree::Reference;

pub trait Expression: DynClone + Debug {
    fn eval(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult;

    fn predicate(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult;
}

dyn_clone::clone_trait_object!(Expression);

pub trait NodeTest: DynClone + Debug {
    fn test_node(&self, rf: &Reference) -> bool;
}

dyn_clone::clone_trait_object!(NodeTest);