use crate::eval::{Object, eval_statements, Type};
use crate::eval::Environment;

use std::collections::HashMap;
use crate::value::{resolve_function_qname, resolve_element_qname};

pub fn fn_string<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {

    let mut current_env = env;

    let item = if arguments.len() == 0 {
        context_item
    } else {
        arguments.get(0).unwrap()
    };

    let str = object_to_string(item);

    (current_env, Object::Atomic(Type::String(str)))
}

pub fn fn_string_join<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {

    if arguments.len() != 1 {
        panic!("got {:?} arguments, but expected 1", arguments.len(), )
    }
    let item = arguments.get(0).unwrap();

    let mut current_env = env;

    let str = object_to_string(item);

    (current_env, Object::Atomic(Type::String(str)))
}


pub fn object_to_string(object: &Object) -> String {
    match object {
        Object::Atomic(Type::String(str)) => str.clone(),
        Object::Atomic(Type::Integer(num)) => num.to_string(),
        Object::Sequence(items) => {
            let mut result = String::new();
            for item in items {
                let str = object_to_string(item);
                result.push_str(str.as_str());
            }
            result
        },
        _ => panic!("TODO object_to_string {:?}", object)
    }
}