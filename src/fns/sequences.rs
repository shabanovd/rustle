use crate::eval::{Object, Type, DynamicContext, EvalResult};
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
    match data(env, item.clone(), &mut result) {
        Ok(env) => relax(env, result),
        Err(msg) => Err((ErrorCode::TODO, String::from("TODO")))
    }
}

fn data<'a>(env: Box<Environment<'a>>, obj: Object, result: &mut Vec<Object>) -> Result<Box<Environment<'a>>, String> {
    match obj {
        Object::Atomic(..) => {
            result.push(obj);
            Ok(env)
        },
        Object::Node(rf) => {
            match rf.to_typed_value(&env) {
                Ok(data) => {
                    let item = Object::Atomic(Type::Untyped(data));
                    result.push(item);
                },
                Err(msg) => return Err(msg)
            }
            Ok(env)
        },
        Object::Array(items) |
        Object::Sequence(items) => {
            data_of_vec(env, items, result)
        },
        _ => todo!()
    }
}

fn data_of_vec<'a>(env: Box<Environment<'a>>, items: Vec<Object>, result: &mut Vec<Object>) -> Result<Box<Environment<'a>>, String> {
    let mut current_env = env;
    for item in items {
        match data(current_env, item, result) {
            Ok(env) => current_env = env,
            Err(msg) => return Err(msg),
        }
    }
    Ok(current_env)
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
        [Object::Empty] => Ok((env, Object::Empty)),
        [Object::Range { min, max}] => {
            Ok((env, Object::Range { min: *max, max: *min } ))
        },
        _ => panic!("error {:?}", arguments)
    }
}

pub(crate) fn fn_subsequence<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    println!("arguments {:?}", arguments);
    match arguments.as_slice() {
        [Object::Empty, ..] => Ok((env, Object::Empty)),
        [Object::Range { min, max }, Object::Atomic(Type::Integer(start)), Object::Atomic(Type::Integer(length))] => {
            if *start <= 0 || *length <= 0 {
                Ok((env, Object::Empty))
            } else {
                if min < max {
                    let new_min = min + (start.max(&1) - 1);
                    if new_min > *max {
                        Ok((env, Object::Empty))
                    } else {
                        let new_max = (new_min + (length - 1)).min(*max);

                        if new_min == new_max {
                            Ok((env, Object::Atomic(Type::Integer(new_min))))
                        } else {
                            Ok((env, Object::Range { min: new_min, max: new_max }))
                        }
                    }
                } else {
                    let new_min = min - (start.max(&1) - 1);
                    if new_min < *max {
                        Ok((env, Object::Empty))
                    } else {
                        let new_max = (new_min - (length - 1)).max(*max);

                        if new_min == new_max {
                            Ok((env, Object::Atomic(Type::Integer(new_min))))
                        } else {
                            Ok((env, Object::Range { min: new_min, max: new_max }))
                        }
                    }
                }
            }
        },
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
        _ => panic!("error {:?}", arguments)
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

pub(crate) fn fn_zero_or_one<'a>(env: Box<Environment<'a>>, mut arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    let arg = arguments.remove(0);
    match arg {
        Object::Empty => Ok((env, Object::Empty)),
        Object::Range{..} => Err((ErrorCode::FORG0003, String::from("TODO"))),
        Object::Atomic(..) |
        Object::Node{..} => Ok((env, arg)),
        Object::Sequence(items) => {
            if items.len() > 1 {
                Err((ErrorCode::FORG0003, String::from("TODO")))
            } else {
                Ok((env, Object::Sequence(items)))
            }
        }
        _ => Err((ErrorCode::FORG0003, String::from("TODO")))
    }
}

pub(crate) fn fn_one_or_more<'a>(env: Box<Environment<'a>>, mut arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    let arg = arguments.remove(0);
    match arg {
        Object::Empty => Err((ErrorCode::FORG0004, String::from("TODO"))),
        Object::Range{..} |
        Object::Atomic(..) |
        Object::Node{..} => Ok((env, arg)),
        Object::Sequence(items) => {
            if items.len() == 0 {
                Err((ErrorCode::FORG0004, String::from("TODO")))
            } else {
                Ok((env, Object::Sequence(items)))
            }
        }
        _ => Err((ErrorCode::FORG0004, String::from("TODO")))
    }
}

pub(crate) fn fn_exactly_one<'a>(env: Box<Environment<'a>>, mut arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    let arg = arguments.remove(0);
    match arg {
        Object::Empty => Err((ErrorCode::FORG0005, String::from("TODO"))),
        Object::Range{..} |
        Object::Atomic(..) |
        Object::Node{..} => Ok((env, arg)),
        Object::Sequence(items) => {
            if items.len() != 1 {
                Err((ErrorCode::FORG0005, String::from("TODO")))
            } else {
                Ok((env, Object::Sequence(items)))
            }
        }
        _ => Err((ErrorCode::FORG0005, String::from("TODO")))
    }
}