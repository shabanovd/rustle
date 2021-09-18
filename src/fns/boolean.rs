use crate::eval::{Object, Type, EvalResult};
use crate::eval::Environment;

use bigdecimal::Zero;

pub fn fn_true<'a>(env: Box<Environment<'a>>, _arguments: Vec<Object>, _context_item: &Object) -> EvalResult<'a> {
    Ok((env, Object::Atomic(Type::Boolean(true))))
}

pub fn fn_false<'a>(env: Box<Environment<'a>>, _arguments: Vec<Object>, _context_item: &Object) -> EvalResult<'a> {
    Ok((env, Object::Atomic(Type::Boolean(false))))
}

pub fn fn_not<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context_item: &Object) -> EvalResult<'a> {
    let result = match arguments.as_slice() {
        [object] => {
            !object_to_bool(object)
        },
        _ => panic!("error")
    };

    Ok((env, Object::Atomic(Type::Boolean(result))))
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
        Object::Atomic(..) => true,
        _ => panic!("TODO object_to_bool {:?}", object)
    }
}