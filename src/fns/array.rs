use crate::eval::{Object, Type};
use crate::eval::Environment;
use crate::eval::Object::Atomic;

pub fn size<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {
    match arguments.as_slice() {
        [Object::Array(array)] => {
            let size = array.len();
            (env, Object::Atomic(Type::Integer(size as i128)))
        }
        _ => panic!("error")
    }
}

pub fn get<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub fn put<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub fn append<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            let mut result = array.clone();
            result.push(item.clone());

            (env, Object::Array(result))
        }

        _ => panic!("error")
    }
}

pub fn subarray<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub fn remove<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub fn insert_before<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub fn head<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub fn tail<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub fn reverse<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub fn join<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub fn for_each<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub fn filter<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub fn fold_left<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub fn fold_right<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub fn for_each_pair<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub fn sort<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub fn flatten<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}