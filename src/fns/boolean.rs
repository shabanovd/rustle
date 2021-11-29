use crate::eval::{Environment, Object, Type, DynamicContext, EvalResult, ErrorInfo};
use crate::eval::sequence_type::*;
use crate::fns::FUNCTION;
use crate::parser::errors::ErrorCode;

use bigdecimal::Zero;
use crate::values::Types;

// fn:true() as xs:boolean
pub(crate) fn FN_TRUE() -> FUNCTION {
    (
        (
            [].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_BOOLEAN.into()))
        ),
        fn_true
    )
}

pub(crate) fn fn_true(env: Box<Environment>, _arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    Ok((env, Object::Atomic(Type::Boolean(true))))
}

// fn:false() as xs:boolean
pub(crate) fn FN_FALSE() -> FUNCTION {
    (
        (
            [].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_BOOLEAN.into()))
        ),
        fn_false
    )
}

pub(crate) fn fn_false(env: Box<Environment>, _arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    Ok((env, Object::Atomic(Type::Boolean(false))))
}

// fn:boolean($arg as item()*) as xs:boolean
pub(crate) fn FN_BOOLEAN() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_more(ItemType::Item)].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_BOOLEAN.into()))
        ),
        fn_boolean
    )
}

pub(crate) fn fn_boolean(env: Box<Environment>, mut arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    let item = arguments.remove(0);

    let v = effective_boolean_value(item)?;
    Ok((env, Object::Atomic(Type::Boolean(v))))
}

// fn:not($arg as item()*) as xs:boolean
pub(crate) fn FN_NOT() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_more(ItemType::Item)].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_BOOLEAN.into()))
        ),
        fn_not
    )
}

pub(crate) fn fn_not(env: Box<Environment>, mut arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    let item = arguments.remove(0);

    let v = effective_boolean_value(item)?;
    Ok((env, Object::Atomic(Type::Boolean(!v))))
}

pub fn effective_boolean_value(object: Object) ->  Result<bool, ErrorInfo> {
    match object {
        Object::Empty => Ok(false),
        Object::Atomic(t) => {
            match t {
                Type::Boolean(v) => Ok(v),

                Type::String(str) |
                Type::NormalizedString(str) |
                Type::AnyURI(str) |
                Type::Untyped(str) => Ok(str.len() != 0),

                Type::Integer(number) => Ok(number != 0),
                Type::Decimal(number) => Ok(!number.is_zero()),
                Type::Float(number) => Ok(!number.is_zero() && !number.is_nan()),
                Type::Double(number) => Ok(!number.is_zero() && !number.is_nan()),

                _ => Err((ErrorCode::FORG0006, String::from("TODO")))
            }
        },
        Object::Sequence(items) => {
            match items[0] {
                Object::Node(_) => Ok(true),
                _ => Err((ErrorCode::FORG0006, String::from("TODO")))
            }

        }
        _ => Err((ErrorCode::FORG0006, String::from("TODO")))
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
        _ => Err((ErrorCode::FORG0006, format!("The {:?} cannot be cast to a boolean", object))) // FORG0001 or FORG0006 ?
    }
}