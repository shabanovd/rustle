use crate::eval::{Object, EvalResult, DynamicContext};
use crate::eval::Environment;
use crate::values::Integer;

pub(crate) fn size(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Array(array)] => {
            let size = array.len();
            Ok((env, Object::Atomic(Integer::boxed(size as i128))))
        }
        _ => panic!("error")
    }
}

pub(crate) fn get(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub(crate) fn put(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub(crate) fn append(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            let mut result = array.clone();
            result.push(item.clone());

            Ok((env, Object::Array(result)))
        }

        _ => panic!("error")
    }
}

pub(crate) fn subarray(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub(crate) fn remove(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub(crate) fn insert_before(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub(crate) fn head(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub(crate) fn tail(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub(crate) fn reverse(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub(crate) fn join(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub(crate) fn for_each(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub(crate) fn filter(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub(crate) fn fold_left(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub(crate) fn fold_right(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub(crate) fn for_each_pair(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub(crate) fn sort(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub(crate) fn flatten(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}