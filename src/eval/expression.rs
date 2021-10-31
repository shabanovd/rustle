use core::fmt;
use crate::eval::{Environment, DynamicContext, EvalResult, Object};
use dyn_clone::DynClone;
use crate::eval::expression::debugging::DynDebug;
use crate::tree::Reference;

mod debugging {
    use std::fmt::Debug;

    pub trait DynDebug {}
    impl<T: Debug> DynDebug for T {}
    pub struct Private;
}

pub trait Expression: DynClone + DynDebug {
    fn eval(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult;

    fn predicate(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult;

    fn dump(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

impl fmt::Debug for dyn Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.dump(f)
    }
}

dyn_clone::clone_trait_object!(Expression);

pub trait NodeTest: DynClone + DynDebug {
    fn test_node(&self, rf: &Reference) -> bool;
}

impl fmt::Debug for dyn NodeTest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeTest {{ .. }}")
    }
}

dyn_clone::clone_trait_object!(NodeTest);