use crate::eval::{Object, EvalResult};
use crate::eval::Environment;

use std::collections::HashMap;

pub fn map_get<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context_item: &Object) -> EvalResult<'a> {
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

pub fn map_merge<'a>(env: Box<Environment>, arguments: Vec<Object>, _context_item: &Object) -> EvalResult<'a> {
    todo!()
}

pub fn map_size<'a>(env: Box<Environment>, arguments: Vec<Object>, _context_item: &Object) -> EvalResult<'a> {
    todo!()
}

pub fn map_keys<'a>(env: Box<Environment>, arguments: Vec<Object>, _context_item: &Object) -> EvalResult<'a> {
    todo!()
}

pub fn map_contains<'a>(env: Box<Environment>, arguments: Vec<Object>, _context_item: &Object) -> EvalResult<'a> {
    todo!()
}

pub fn map_find<'a>(env: Box<Environment>, arguments: Vec<Object>, _context_item: &Object) -> EvalResult<'a> {
    todo!()
}

pub fn map_put<'a>(env: Box<Environment>, arguments: Vec<Object>, _context_item: &Object) -> EvalResult<'a> {
    todo!()
}

pub fn map_entry<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context_item: &Object) -> EvalResult<'a> {
    match arguments.as_slice() {
        [Object::Atomic(k), Object::Atomic(v)] => {

            let mut map = HashMap::new();

            map.insert(k.clone(), Object::Atomic(v.clone())); //TODO: understand, is it possible to avoid clone?

            Ok((env, Object::Map(map)))
        }

        _ => panic!("error")
    }
}

pub fn map_remove<'a>(env: Box<Environment>, arguments: Vec<Object>, _context_item: &Object) -> EvalResult<'a> {
    todo!()
}

pub fn map_for_each<'a>(env: Box<Environment>, arguments: Vec<Object>, _context_item: &Object) -> EvalResult<'a> {
    todo!()
}