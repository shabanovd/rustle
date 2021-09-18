use crate::eval::{Object, Type, EvalResult, comparison};
use crate::eval::Environment;

pub fn fn_deep_equal<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context_item: &Object) -> EvalResult<'a> {

    let result = match arguments.as_slice() {
        [o1, o2] => {
            comparison::deep_eq(o1, o2)
        },
        _ => panic!("error")
    };

    match result {
        Ok(v) => Ok((env, Object::Atomic(Type::Boolean(v)))),
        Err(code) => Err((code, String::from("TODO")))
    }
}