use crate::eval::{Object, Type, NumberCase, EvalResult};
use crate::eval::Environment;

use std::collections::HashMap;
use crate::value::{resolve_function_qname, resolve_element_qname};

pub fn fn_true<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> EvalResult<'a> {
    Ok((env, Object::Atomic(Type::Boolean(true))))
}

pub fn fn_false<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> EvalResult<'a> {
    Ok((env, Object::Atomic(Type::Boolean(false))))
}

pub fn fn_not<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> EvalResult<'a> {
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
        Object::Atomic(Type::Integer(num)) => *num != 0,
        Object::Atomic(Type::Decimal { number, case }) |
        Object::Atomic(Type::Float { number, case }) |
        Object::Atomic(Type::Double { number, case }) => {
            match case {
                NumberCase::Normal => {
                    if number.is_some() {
                        !number.unwrap().is_zero()
                    } else {
                        panic!("internal error")
                    }
                }
                NumberCase::NaN => false,
                NumberCase::PlusInfinity => true,
                NumberCase::MinusInfinity => true,
            }
        },
        Object::Atomic(..) => true,
        _ => panic!("TODO object_to_bool {:?}", object)
    }
}