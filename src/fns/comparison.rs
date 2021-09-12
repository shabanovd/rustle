use crate::eval::{Object, Type, EvalResult};
use crate::eval::Environment;

use std::collections::HashMap;
use crate::value::{resolve_function_qname, resolve_element_qname};

pub fn fn_deep_equal<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> EvalResult<'a> {

    let result = match arguments.as_slice() {
        [o1, o2] => {
            crate::eval::comparison::deep_eq(o1, o2)
        },
        _ => panic!("error")
    };

    Ok((env, Object::Atomic(Type::Boolean(result))))
}