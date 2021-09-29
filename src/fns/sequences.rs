use crate::eval::{Object, Type, typed_value_of_node, DynamicContext, EvalResult};
use crate::eval::Environment;

use crate::eval::helpers::relax;
use crate::parser::errors::ErrorCode;

pub(crate) fn fn_data<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult<'a> {

    let item = if arguments.len() == 0 {
        &context.item
    } else {
        arguments.get(0).unwrap()
    };

    let mut result = vec![];
    data(item.clone(), &mut result);

    relax(env, result)
}

fn data(obj: Object, result: &mut Vec<Object>) {
    match obj {
        Object::Atomic(..) => result.push(obj),
        Object::Node(node) => {
            let mut data = vec![];
            typed_value_of_node(node, &mut data);
            let item = Object::Atomic(Type::Untyped(data.join("")));
            result.push(item);
        },
        Object::Array(items) => data_of_vec(items, result),
        Object::Sequence(items) => data_of_vec(items, result),
        _ => todo!()
    }
}

fn data_of_vec(items: Vec<Object>, result: &mut Vec<Object>) {
    for item in items {
        data(item, result);
    }
}

pub(crate) fn fn_empty<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    let result = match arguments.as_slice() {
        [Object::Empty] => true,
        [Object::Range { min, max}] => {
            min == max
        },
        _ => false
    };

    Ok((env, Object::Atomic(Type::Boolean(result))))
}

pub(crate) fn fn_remove<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    match arguments.as_slice() {
        [Object::Empty, ..] => Ok((env, Object::Empty)),
        [Object::Sequence(items), Object::Atomic(Type::Integer(pos))] => {
            let position = *pos - 1;
            let mut result = items.clone();

            if position >= 0 && position < items.len() as i128 {
                result.remove(position as usize);
            }

            Ok((env, Object::Sequence(result)))
        },
        _ => panic!("error")
    }
}

pub(crate) fn fn_reverse<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {

    match arguments.as_slice() {
        [Object::Range { min, max}] => {
            Ok((env, Object::Range { min: *max, max: *min } ))
        },
        _ => panic!("error")
    }
}

pub(crate) fn fn_subsequence<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    match arguments.as_slice() {
        [Object::Empty, ..] => Ok((env, Object::Empty)),
        [Object::Atomic(t), Object::Atomic(Type::Integer(start)), Object::Atomic(Type::Integer(length))] => {
            if *start == 1 && *length >= 1 {
                Ok((env, Object::Atomic(t.clone())))
            } else {
                Ok((env, Object::Empty))
            }
        },
        [Object::Sequence(items), Object::Atomic(Type::Integer(start)), Object::Atomic(Type::Integer(length))] => {
            let mut result = Vec::with_capacity(*length as usize);

            let from = *start as usize;
            let till = (*start + *length) as usize;

            for position in from..till as usize {
                if let Some(item) = items.get((position - 1) as usize) {
                    result.push(item.clone());
                } else {
                    break
                }
            }
            Ok((env, Object::Sequence(result)))
        },
        _ => panic!("error")
    }
}

pub(crate) fn fn_position<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult<'a> {
    if let Some(position) = context.position {
        Ok((env, Object::Atomic(Type::Integer(position as i128))))
    } else {
        Err((ErrorCode::XPDY0002, String::from("context position unknown")))
    }
}

pub(crate) fn fn_last<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult<'a> {
    if let Some(last) = context.last {
        Ok((env, Object::Atomic(Type::Integer(last as i128))))
    } else {
        Err((ErrorCode::XPDY0002, String::from("context size unknown")))
    }
}