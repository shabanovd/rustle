//use crate::eval::Type;
use crate::eval::{Object, eval_statements, Type};
use crate::eval::Environment;

use std::collections::HashMap;
use crate::value::{resolve_function_qname, resolve_element_qname};

pub fn fn_empty<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {

    let mut current_env = env;

    println!("arguments {:?}", arguments);

    let result = match arguments.as_slice() {
        [Object::Empty] => true,
        [Object::Range { min, max}] => {
            min == max
        },
        _ => false
    };

    ( current_env, Object::Atomic(Type::Boolean(result)) )
}

pub fn fn_reverse<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {

    let mut current_env = env;

    match arguments.as_slice() {
        [Object::Range { min, max}] => {
            (current_env, Object::Range { min: *max, max: *min } )
        },
        _ => panic!("error")
    }
}