use crate::eval::{Object, EvalResult, DynamicContext};
use crate::eval::Environment;

use std::collections::HashMap;

pub(crate) fn map_get<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
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

pub(crate) fn map_merge<'a>(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    todo!()
}

pub(crate) fn map_size<'a>(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    todo!()
}

pub(crate) fn map_keys<'a>(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    todo!()
}

pub(crate) fn map_contains<'a>(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    todo!()
}

pub(crate) fn map_find<'a>(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    todo!()
}

pub(crate) fn map_put<'a>(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    todo!()
}

pub(crate) fn map_entry<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    match arguments.as_slice() {
        [Object::Atomic(k), Object::Atomic(v)] => {

            let mut map = HashMap::new();

            map.insert(k.clone(), Object::Atomic(v.clone())); //TODO: understand, is it possible to avoid clone?

            Ok((env, Object::Map(map)))
        }

        _ => panic!("error")
    }
}

pub(crate) fn map_remove<'a>(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    todo!()
}

pub(crate) fn map_for_each<'a>(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    todo!()
}