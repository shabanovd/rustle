use crate::eval::{Object, Type, DynamicContext, EvalResult};
use crate::eval::Environment;
use crate::serialization::object_to_string;
use crate::serialization::to_string::_object_to_string;
use crate::parser::errors::ErrorCode;

pub(crate) fn fn_string<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult<'a> {

    let item = if arguments.len() == 0 {
        &context.item
    } else {
        arguments.get(0).unwrap()
    };

    let str = object_to_string(&env, item);

    Ok((env, Object::Atomic(Type::String(str))))
}

pub(crate) fn fn_upper_case<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {

    // TODO empty sequence return empty string
    let item = arguments.get(0).unwrap();

    let str = object_to_string(&env, item);

    Ok((env, Object::Atomic(Type::String(str.to_uppercase()))))
}

pub(crate) fn fn_lower_case<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {

    // TODO empty sequence return empty string
    let item = arguments.get(0).unwrap();

    let str = object_to_string(&env, item);

    Ok((env, Object::Atomic(Type::String(str.to_lowercase()))))
}

pub(crate) fn fn_concat<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    let str = arguments.iter()
        .map(|item| object_to_string(&env, item))
        .collect();

    Ok((env, Object::Atomic(Type::String(str))))
}

pub(crate) fn fn_string_join<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    let str = if let Some(item) = arguments.get(0) {
        if let Some(sep) = arguments.get(1) {
            let sep = object_to_string(&env, sep);
            _object_to_string(&env, item, true, sep.as_str())
        } else {
            _object_to_string(&env, item, true, " ")
        }
    } else {
        return Err((ErrorCode::TODO, format!("got {:?} arguments, but expected 1 or 2", arguments.len())));
    };

    Ok((env, Object::Atomic(Type::String(str))))
}

pub(crate) fn fn_string_length<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult<'a> {

    let item = if arguments.len() == 0 {
        &context.item
    } else {
        arguments.get(0).unwrap()
    };

    let str = object_to_string(&env, item);

    Ok((env, Object::Atomic(Type::Integer(str.len() as i128))))
}

pub(crate) fn fn_normalize_space<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult<'a> {

    let item = if arguments.len() == 0 {
        &context.item
    } else {
        arguments.get(0).unwrap()
    };

    let mut str = object_to_string(&env, item);
    str = str.trim().to_string();

    // TODO replacing sequences of one or more adjacent whitespace characters with a single space

    Ok((env, Object::Atomic(Type::String(str))))
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

pub(crate) fn fn_starts_with<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult<'a> {

    let string = arguments.get(0).unwrap();
    let pattern = arguments.get(1).unwrap();

    let string = object_to_string(&env, string);
    let pattern = object_to_string(&env, pattern);

    let result = string.starts_with(&pattern);

    Ok((env, Object::Atomic(Type::Boolean(result))))
}

pub(crate) fn fn_ends_with<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult<'a> {

    let string = arguments.get(0).unwrap();
    let pattern = arguments.get(1).unwrap();

    let string = object_to_string(&env, string);
    let pattern = object_to_string(&env, pattern);

    let result = string.ends_with(&pattern);

    Ok((env, Object::Atomic(Type::Boolean(result))))
}

pub(crate) fn object_to_array(object: Object) -> Vec<Object> {
    match object {
        Object::Array(array) => array,
        _ => panic!("TODO object_to_array {:?}", object)
    }
}
