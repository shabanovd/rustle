//use crate::eval::Type;
use crate::eval::Object;
use crate::eval::Environment;

use std::collections::HashMap;

pub fn map_get<'a>(env: &'a mut Environment<'a>, arguments: Vec<Object>) -> (&'a mut Environment<'a>, Object) {
    match arguments.as_slice() {
        [Object::Map(map), Object::Atomic(k)] => {

            println!("map_get {:?} {:?}", k, map);

            if let Some(value) = map.get(k) {
                (env, value.clone()) //TODO: understand, is it possible to avoid clone? for example by using reference all around
            } else {
                (env, Object::Empty)
            }
        }

        _ => (env, Object::Empty), // TODO: raise error?
    }
}

pub fn map_merge<'a>(env: &'a mut Environment<'a>, arguments: Vec<Object>) -> (&'a mut Environment<'a>, Object) {
    (env, Object::Empty) // TODO: raise error?
}

pub fn map_size<'a>(env: &'a mut Environment<'a>, arguments: Vec<Object>) -> (&'a mut Environment<'a>, Object) {
    (env, Object::Empty) // TODO: raise error?
}

pub fn map_keys<'a>(env: &'a mut Environment<'a>, arguments: Vec<Object>) -> (&'a mut Environment<'a>, Object) {
    (env, Object::Empty) // TODO: raise error?
}

pub fn map_contains<'a>(env: &'a mut Environment<'a>, arguments: Vec<Object>) -> (&'a mut Environment<'a>, Object) {
    (env, Object::Empty) // TODO: raise error?
}

pub fn map_find<'a>(env: &'a mut Environment<'a>, arguments: Vec<Object>) -> (&'a mut Environment<'a>, Object) {
    (env, Object::Empty) // TODO: raise error?
}

pub fn map_put<'a>(env: &'a mut Environment<'a>, arguments: Vec<Object>) -> (&'a mut Environment<'a>, Object) {
    (env, Object::Empty) // TODO: raise error?
}

pub fn map_entry<'a>(env: &'a mut Environment<'a>, arguments: Vec<Object>) -> (&'a mut Environment<'a>, Object) {
    match arguments.as_slice() {
        [Object::Atomic(k), Object::Atomic(v)] => {

            let mut map = HashMap::new();

            println!("map_entry {:?} {:?}", k, v);

            map.insert(k.clone(), Object::Atomic(v.clone())); //TODO: understand, is it possible to avoid clone?

            (env, Object::Map(map))
        }

        _ => (env, Object::Empty), // TODO: raise error?
    }
}

pub fn map_remove<'a>(env: &'a mut Environment<'a>, arguments: Vec<Object>) -> (&'a mut Environment<'a>, Object) {
    (env, Object::Empty) // TODO: raise error?
}

pub fn map_for_each<'a>(env: &'a mut Environment<'a>, arguments: Vec<Object>) -> (&'a mut Environment<'a>, Object) {
    (env, Object::Empty) // TODO: raise error?
}