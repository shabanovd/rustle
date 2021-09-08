use crate::eval::{Object, Type, eval_expr};
use crate::eval::Environment;
use crate::serialization::object_to_string;

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

pub fn fn_concat<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {
    let str = arguments.iter()
        .map(|item| object_to_string(item))
        .collect();

    (env, Object::Atomic(Type::String(str)))
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

pub fn fn_string_to_codepoints<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {

    let mut current_env = env;

    let result = match arguments.as_slice() {
        [Object::Empty] => Object::Empty,
        [Object::Atomic(Type::String(str))] => {
            let mut codes = Vec::with_capacity(str.len());
            for char in str.chars() {
                // let code = char as u32;
                codes.push(Object::Atomic(Type::Integer(char as i128)));
            }

            Object::Sequence(codes)
        },
        _ => panic!("error")
    };

    (current_env, result)
}


pub fn object_to_array(object: Object) -> Vec<Object> {
    match object {
        Object::Array(array) => array,
        _ => panic!("TODO object_to_array {:?}", object)
    }
}
