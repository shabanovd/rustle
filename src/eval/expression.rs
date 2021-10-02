use crate::eval::{Environment, DynamicContext, EvalResult, Object};
use dyn_clone::DynClone;

pub trait Expression: DynClone {
    fn eval<'a>(&self, env: Box<Environment<'a>>, context: &DynamicContext) -> EvalResult<'a>;

    fn predicate<'a>(&self, env: Box<Environment<'a>>, context: &DynamicContext, value: Object) -> EvalResult<'a>;

    fn debug(&self) -> String;
}

dyn_clone::clone_trait_object!(Expression);