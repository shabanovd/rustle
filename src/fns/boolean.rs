use crate::eval::{Object, Type, DynamicContext, EvalResult};
use crate::eval::Environment;

use bigdecimal::Zero;

pub(crate) fn fn_true<'a>(env: Box<Environment<'a>>, _arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    Ok((env, Object::Atomic(Type::Boolean(true))))
}

pub(crate) fn fn_false<'a>(env: Box<Environment<'a>>, _arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    Ok((env, Object::Atomic(Type::Boolean(false))))
}

pub(crate) fn fn_not<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    let result = match arguments.as_slice() {
        [object] => {
            !object_to_bool(object)
        },
        _ => panic!("error")
    };

    Ok((env, Object::Atomic(Type::Boolean(result))))
}

pub(crate) fn fn_boolean<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    let item = arguments.get(0).unwrap();

    let flag = object_to_bool(item);

    Ok((env, Object::Atomic(Type::Boolean(flag))))
}

pub fn object_to_bool(object: &Object) -> bool {
    match object {
        Object::Empty => false,
        Object::Atomic(Type::Boolean(v)) => *v,
        Object::Atomic(Type::String(str)) => str.len() != 0,
        Object::Atomic(Type::Integer(number)) => !number.is_zero(),
        Object::Atomic(Type::Decimal(number)) => !number.is_zero(),
        Object::Atomic(Type::Float(number)) => {
            if number.is_nan() {
                false
            } else if number.is_infinite() && !number.is_zero() {
                true
            } else {
                false
            }
        },
        Object::Atomic(Type::Double(number)) => {
            if number.is_nan() {
                false
            } else if number.is_infinite() && !number.is_zero() {
                true
            } else {
                false
            }
        },
        Object::Node(..) |
            Object::Atomic(..) => true,
        _ => panic!("TODO object_to_bool {:?}", object)
    }
}