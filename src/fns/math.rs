//use crate::eval::Type;
use crate::eval::{Object, eval_statements, Type};
use crate::eval::Environment;

use std::collections::HashMap;
use crate::value::{resolve_function_qname, resolve_element_qname};

pub fn fn_abs<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {

    let mut current_env = env;

    match arguments.as_slice() {
        [Object::Atomic(Type::Integer(num))] => {
            (current_env, Object::Atomic(Type::Integer((*num).abs())))
        },
        _ => panic!("error")
    }
}