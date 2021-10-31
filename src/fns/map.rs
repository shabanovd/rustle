use crate::eval::{Object, EvalResult, DynamicContext};
use crate::eval::Environment;

use std::collections::HashMap;

pub(crate) fn map_get(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Map(map), Object::Atomic(k)] => {

            // println!("map_get {:?} {:?}", k, map);

            if let Some(value) = map.get(k) {
                Ok((env, value.clone()))
            } else {
                Ok((env, Object::Empty))
            }
        }

        _ => panic!("error")
    }
}

pub(crate) fn map_merge(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

pub(crate) fn map_size(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

pub(crate) fn map_keys(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

pub(crate) fn map_contains(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

pub(crate) fn map_find(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

pub(crate) fn map_put(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

pub(crate) fn map_entry(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Atomic(k), Object::Atomic(v)] => {

            let mut map = HashMap::new();

            map.insert(k.clone(), Object::Atomic(v.clone())); //TODO: understand, is it possible to avoid clone?

            Ok((env, Object::Map(map)))
        }

        _ => panic!("error")
    }
}

pub(crate) fn map_remove(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

pub(crate) fn map_for_each(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}