use crate::eval::{Object, Type, EvalResult, DynamicContext};
use crate::eval::Environment;

pub(crate) fn size<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    match arguments.as_slice() {
        [Object::Array(array)] => {
            let size = array.len();
            Ok((env, Object::Atomic(Type::Integer(size as i128))))
        }
        _ => panic!("error")
    }
}

pub(crate) fn get<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub(crate) fn put<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub(crate) fn append<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            let mut result = array.clone();
            result.push(item.clone());

            Ok((env, Object::Array(result)))
        }

        _ => panic!("error")
    }
}

pub(crate) fn subarray<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub(crate) fn remove<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub(crate) fn insert_before<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub(crate) fn head<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub(crate) fn tail<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub(crate) fn reverse<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub(crate) fn join<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub(crate) fn for_each<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub(crate) fn filter<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub(crate) fn fold_left<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub(crate) fn fold_right<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub(crate) fn for_each_pair<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub(crate) fn sort<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

pub(crate) fn flatten<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}