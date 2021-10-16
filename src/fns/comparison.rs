use crate::eval::{Object, Type, EvalResult, comparison, DynamicContext};
use crate::eval::Environment;

pub(crate) fn fn_deep_equal<'a>(env: Box<Environment<'a>>, mut arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    let o1 = arguments.remove(0);
    let o2 = arguments.remove(0);
    match comparison::deep_eq((&env, &o1), (&env, &o2)) {
        Ok(v) => Ok((env, Object::Atomic(Type::Boolean(v)))),
        Err(e) => Err(e)
    }
}