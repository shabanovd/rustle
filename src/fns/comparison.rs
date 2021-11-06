use crate::eval::{Object, EvalResult, comparison, DynamicContext};
use crate::eval::Environment;
use crate::values::Boolean;

pub(crate) fn fn_deep_equal(env: Box<Environment>, mut arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    let o1 = arguments.remove(0);
    let o2 = arguments.remove(0);
    match comparison::deep_eq((&env, &o1), (&env, &o2)) {
        Ok(v) => Ok((env, Object::Atomic(Boolean::boxed(v)))),
        Err(e) => Err(e)
    }
}