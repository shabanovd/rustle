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

    let v = item.effective_boolean_value()?;
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

    let v = item.effective_boolean_value()?;
    Ok((env, Object::Atomic(Type::Boolean(!v))))
}