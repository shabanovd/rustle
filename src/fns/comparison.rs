use crate::eval::{Object, Type};
use crate::eval::Environment;

use std::collections::HashMap;
use crate::value::{resolve_function_qname, resolve_element_qname};

pub fn fn_deep_equal<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {

    let result = match arguments.as_slice() {
        [o1, o2] => {
            crate::eval::comparison::deep_eq(o1, o2)
        },
        _ => panic!("error")
    };

    ( env, Object::Atomic(Type::Boolean(result)) )
}