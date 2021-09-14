//use crate::eval::Type;
use crate::eval::{Object, Type, EvalResult, typed_value_of_node};
use crate::eval::Environment;

use std::collections::HashMap;
use crate::value::{resolve_function_qname, resolve_element_qname};
use crate::eval::Object::Atomic;

pub fn fn_data<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> EvalResult<'a> {

    let item = if arguments.len() == 0 {
        context_item
    } else {
        arguments.get(0).unwrap()
    };

    println!("{:?}", arguments);

    let mut result = vec![];
    data(item.clone(), &mut result);

    if result.len() == 0 {
        Ok((env, Object::Empty))
    } else if result.len() == 1 {
        let item = result.remove(0);
        Ok((env, item))
    } else {
        Ok((env, Object::Sequence(result)))
    }
}

fn data(obj: Object, result: &mut Vec<Object>) {
    match obj {
        Object::Atomic(..) => result.push(obj),
        Object::Node(node) => {
            let mut data = vec![];
            typed_value_of_node(node, &mut data);
            let item = Object::Atomic(Type::Untyped(data.join("")));
            result.push(item);
        },
        Object::Array(items) => data_of_vec(items, result),
        Object::Sequence(items) => data_of_vec(items, result),
        _ => todo!()
    }
}

fn data_of_vec(items: Vec<Object>, result: &mut Vec<Object>) {
    for item in items {
        data(item, result);
    }
}

pub fn fn_empty<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> EvalResult<'a> {

    let mut current_env = env;

    println!("arguments {:?}", arguments);

    let result = match arguments.as_slice() {
        [Object::Empty] => true,
        [Object::Range { min, max}] => {
            min == max
        },
        _ => false
    };

    Ok((current_env, Object::Atomic(Type::Boolean(result))))
}

pub fn fn_reverse<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> EvalResult<'a> {

    let mut current_env = env;

    match arguments.as_slice() {
        [Object::Range { min, max}] => {
            Ok((current_env, Object::Range { min: *max, max: *min } ))
        },
        _ => panic!("error")
    }
}

pub fn sort_and_dedup(seq: &mut Vec<Object>) {
    seq.sort();
    seq.dedup();
}