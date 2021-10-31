use crate::eval::{Object, Type, DynamicContext, EvalResult, ErrorInfo};
use crate::eval::Environment;

use bigdecimal::Zero;
use crate::parser::errors::ErrorCode;

pub(crate) fn fn_true(env: Box<Environment>, _arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    Ok((env, Object::Atomic(Type::Boolean(true))))
}

pub(crate) fn fn_false(env: Box<Environment>, _arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    Ok((env, Object::Atomic(Type::Boolean(false))))
}

pub(crate) fn fn_not(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    let result = match arguments.as_slice() {
        [object] => {
            object_to_bool(object)
        },
        _ => panic!("error")
    };

    match result {
        Ok(v) => Ok((env, Object::Atomic(Type::Boolean(!v)))),
        Err(e) => Err(e)
    }
}

pub(crate) fn fn_boolean(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    let item = arguments.get(0).unwrap();

    match object_to_bool(item) {
        Ok(v) => Ok((env, Object::Atomic(Type::Boolean(v)))),
        Err(e) => Err(e)
    }
}

pub fn object_to_bool(object: &Object) ->  Result<bool, ErrorInfo> {
    object_casting_bool(object, false)
}

pub fn object_casting_bool(object: &Object, is_casting: bool) -> Result<bool, ErrorInfo> {
    match object {
        Object::Atomic(Type::Boolean(v)) => Ok(*v),
        Object::Empty => Ok(false),
        Object::Atomic(Type::String(str)) => {
            if is_casting {
                if str == "false" || str == "0" {
                    Ok(false)
                } else if str == "true" || str == "1" {
                    Ok(true)
                } else {
                    Err((ErrorCode::FORG0001, format!("The string {} cannot be cast to a boolean", str)))
                }
            } else {
                Ok(str.len() != 0)
            }
        },
        Object::Atomic(Type::Integer(number)) => Ok(!number.is_zero()),
        Object::Atomic(Type::Decimal(number)) => Ok(!number.is_zero()),
        Object::Atomic(Type::Float(number)) => {
            let v = if number.is_nan() {
                false
            } else if number.is_infinite() && !number.is_zero() {
                true
            } else {
                false
            };
            Ok(v)
        },
        Object::Atomic(Type::Double(number)) => {
            let v = if number.is_nan() {
                false
            } else if number.is_infinite() && !number.is_zero() {
                true
            } else {
                false
            };
            Ok(v)
        },
        Object::Node{..} |
        Object::Atomic(..) => Ok(!is_casting),
        _ => Err((ErrorCode::FORG0001, format!("The {:?} cannot be cast to a boolean", object)))
    }
}