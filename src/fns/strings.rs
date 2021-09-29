use crate::eval::{Object, Type, DynamicContext, EvalResult};
use crate::eval::Environment;
use crate::serialization::object_to_string;

pub(crate) fn fn_string<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult<'a> {

    let item = if arguments.len() == 0 {
        &context.item
    } else {
        arguments.get(0).unwrap()
    };

    let str = object_to_string(item);

    Ok((env, Object::Atomic(Type::String(str))))
}

pub(crate) fn fn_upper_case<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {

    // TODO empty sequence return empty string
    let item = arguments.get(0).unwrap();

    let str = object_to_string(item);

    Ok((env, Object::Atomic(Type::String(str.to_uppercase()))))
}

pub(crate) fn fn_lower_case<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {

    // TODO empty sequence return empty string
    let item = arguments.get(0).unwrap();

    let str = object_to_string(item);

    Ok((env, Object::Atomic(Type::String(str.to_lowercase()))))
}

pub(crate) fn fn_concat<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    let str = arguments.iter()
        .map(|item| object_to_string(item))
        .collect();

    Ok((env, Object::Atomic(Type::String(str))))
}

pub(crate) fn fn_string_join<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {

    if arguments.len() != 1 {
        panic!("got {:?} arguments, but expected 1", arguments.len(), )
    }
    let item = arguments.get(0).unwrap();

    let str = object_to_string(item);

    Ok((env, Object::Atomic(Type::String(str))))
}

pub(crate) fn fn_string_length<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult<'a> {

    let item = if arguments.len() == 0 {
        &context.item
    } else {
        arguments.get(0).unwrap()
    };

    let str = object_to_string(item);

    println!("{:?} {:?}", str, str.len());

    Ok((env, Object::Atomic(Type::Integer(str.len() as i128))))
}

pub(crate) fn fn_string_to_codepoints<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {

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

    Ok((env, result))
}


pub(crate) fn object_to_array(object: Object) -> Vec<Object> {
    match object {
        Object::Array(array) => array,
        _ => panic!("TODO object_to_array {:?}", object)
    }
}
