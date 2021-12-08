use crate::eval::{Environment, Object, Type, DynamicContext, EvalResult, object_to_integer};
use crate::eval::sequence_type::*;
use crate::fns::FUNCTION;

use crate::eval::helpers::relax;
use crate::parser::errors::ErrorCode;

// fn:data() as xs:anyAtomicType*
pub(crate) fn FN_DATA_0() -> FUNCTION {
    (
        (
            [].to_vec(),
            SequenceType::zero_or_more(ItemType::AnyAtomicType)
        ),
        fn_data
    )
}

// fn:data($arg as item()*) as xs:anyAtomicType*
pub(crate) fn FN_DATA_1() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_more(ItemType::Item)].to_vec(),
            SequenceType::zero_or_more(ItemType::AnyAtomicType)
        ),
        fn_data
    )
}

pub(crate) fn fn_data(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {

    let item = if arguments.len() == 0 {
        &context.item
    } else {
        arguments.get(0).unwrap()
    };

    let mut result = vec![];
    match data(env, item.clone(), &mut result) {
        Ok(env) => relax(env, result),
        Err(msg) => Err((ErrorCode::TODO, msg))
    }
}

fn data(env: Box<Environment>, obj: Object, result: &mut Vec<Object>) -> Result<Box<Environment>, String> {
    match obj {
        Object::Atomic(..) => {
            result.push(obj);
            Ok(env)
        },
        Object::Node(rf) => {
            match rf.to_typed_value() {
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

fn data_of_vec(env: Box<Environment>, items: Vec<Object>, result: &mut Vec<Object>) -> Result<Box<Environment>, String> {
    let mut current_env = env;
    for item in items {
        match data(current_env, item, result) {
            Ok(env) => current_env = env,
            Err(msg) => return Err(msg),
        }
    }
    Ok(current_env)
}

// fn:empty($arg as item()*) as xs:boolean
pub(crate) fn FN_EMPTY() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_more(ItemType::Item)].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_BOOLEAN.into()))
        ),
        fn_empty
    )
}

pub(crate) fn fn_empty(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    let result = match arguments.as_slice() {
        [Object::Empty] => true,
        [Object::Range { min, max}] => {
            min == max
        },
        _ => false
    };

    Ok((env, Object::Atomic(Type::Boolean(result))))
}

// fn:exists($arg as item()*) as xs:boolean
pub(crate) fn FN_EXISTS() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_more(ItemType::Item)].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_BOOLEAN.into()))
        ),
        fn_exists
    )
}

pub(crate) fn fn_exists(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    let result = match arguments.as_slice() {
        [Object::Empty] => false,
        _ => true
    };

    Ok((env, Object::Atomic(Type::Boolean(result))))
}

// fn:head($arg as item()*) as item()?
pub(crate) fn FN_HEAD() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_more(ItemType::Item)].to_vec(),
            SequenceType::exactly_one(ItemType::Item)
        ),
        fn_head
    )
}

pub(crate) fn fn_head(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// fn:tail($arg as item()*) as item()*
pub(crate) fn FN_TAIL() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_more(ItemType::Item)].to_vec(),
            SequenceType::zero_or_more(ItemType::Item)
        ),
        fn_tail
    )
}

pub(crate) fn fn_tail(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// fn:insert-before($target as item()*, $position as xs:integer, $inserts as item()*) as item()*
pub(crate) fn FN_INSERT_BEFORE() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_more(ItemType::Item),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_INTEGER.into())),
                SequenceType::zero_or_more(ItemType::Item),
            ].to_vec(),
            SequenceType::zero_or_more(ItemType::Item)
        ),
        fn_insert_before
    )
}

pub(crate) fn fn_insert_before(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// fn:remove($target as item()*, $position as xs:integer) as item()*
pub(crate) fn FN_REMOVE() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_more(ItemType::Item),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_INTEGER.into()))
            ].to_vec(),
            SequenceType::zero_or_more(ItemType::Item)
        ),
        fn_remove
    )
}

pub(crate) fn fn_remove(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
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

// fn:reverse($arg as item()*) as item()*
pub(crate) fn FN_REVERSE() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_more(ItemType::Item)].to_vec(),
            SequenceType::zero_or_more(ItemType::Item)
        ),
        fn_reverse
    )
}

pub(crate) fn fn_reverse(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Empty] => Ok((env, Object::Empty)),
        [Object::Range { min, max}] => {
            Ok((env, Object::Range { min: *max, max: *min } ))
        },
        _ => panic!("error {:?}", arguments)
    }
}

// fn:subsequence($sourceSeq as item()*, $startingLoc as xs:double) as item()*
pub(crate) fn FN_SUBSEQUENCE_2() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_more(ItemType::Item),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DOUBLE.into()))
            ].to_vec(),
            SequenceType::zero_or_more(ItemType::Item)
        ),
        fn_subsequence
    )
}

// fn:subsequence($sourceSeq as item()*, $startingLoc as xs:double, $length as xs:double) as item()*
pub(crate) fn FN_SUBSEQUENCE_3() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_more(ItemType::Item),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DOUBLE.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DOUBLE.into())),
            ].to_vec(),
            SequenceType::zero_or_more(ItemType::Item)
        ),
        fn_subsequence
    )
}

pub(crate) fn fn_subsequence(env: Box<Environment>, mut arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    println!("arguments {:?}", arguments);

    let source = arguments.remove(0);
    let start = arguments.remove(0).to_integer()?;
    let mut length_opt = if arguments.len() == 0 {
        None
    } else {
        Some(arguments.remove(0).to_integer()?)
    };

    match source {
        Object::Empty => Ok((env, Object::Empty)),
        Object::Range { min: mi, max: ma } => {
            let min = mi.min(ma);
            let max = mi.max(ma);

            let length = if let Some(length) = length_opt { length } else { min - max + 1 };

            if start <= 0 || length <= 0 {
                Ok((env, Object::Empty))
            } else {
                let new_min = min + (start.max(1) - 1);
                if new_min > max {
                    Ok((env, Object::Empty))
                } else {
                    let new_max = (new_min + (length - 1)).min(max);

                    if new_min == new_max {
                        Ok((env, Object::Atomic(Type::Integer(new_min))))
                    } else {
                        Ok((env, Object::Range { min: new_min, max: new_max }))
                    }
                }
            }
        },
        Object::Atomic(t) => {
            let length = if let Some(length) = length_opt { length } else { 1 };

            if start == 1 && length >= 1 {
                Ok((env, Object::Atomic(t.clone())))
            } else {
                Ok((env, Object::Empty))
            }
        },
        Object::Sequence(items) => {
            let length = if let Some(length) = length_opt { length } else { items.len() as i128 };

            let mut result = Vec::with_capacity(length as usize);

            let from = start as usize;
            let till = (start + length) as usize;

            for position in from..till as usize {
                if let Some(item) = items.get((position - 1) as usize) {
                    result.push(item.clone());
                } else {
                    break
                }
            }
            relax(env, result)
        },
        _ => panic!("error {:?}", arguments)
    }
}

// fn:unordered($sourceSeq as item()*) as item()*
pub(crate) fn FN_UNORDERED() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_more(ItemType::Item)].to_vec(),
            SequenceType::zero_or_more(ItemType::Item)
        ),
        fn_unordered
    )
}

pub(crate) fn fn_unordered(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// fn:position() as xs:integer
pub(crate) fn FN_POSITION() -> FUNCTION {
    (
        (
            [].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_INTEGER.into()))
        ),
        fn_position
    )
}

pub(crate) fn fn_position(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    if let Some(position) = context.position {
        Ok((env, Object::Atomic(Type::Integer(position as i128))))
    } else {
        Err((ErrorCode::XPDY0002, String::from("context position unknown")))
    }
}

// fn:last() as xs:integer
pub(crate) fn FN_LAST() -> FUNCTION {
    (
        (
            [].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_INTEGER.into()))
        ),
        fn_last
    )
}

pub(crate) fn fn_last(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    if let Some(last) = context.last {
        Ok((env, Object::Atomic(Type::Integer(last as i128))))
    } else {
        Err((ErrorCode::XPDY0002, String::from("context size unknown")))
    }
}

// fn:zero-or-one($arg as item()*) as item()?
pub(crate) fn FN_ZERO_OR_ONE() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_more(ItemType::Item)].to_vec(),
            SequenceType::zero_or_one(ItemType::Item)
        ),
        fn_zero_or_one
    )
}

pub(crate) fn fn_zero_or_one(env: Box<Environment>, mut arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
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


// fn:one-or-more($arg as item()*) as item()+
pub(crate) fn FN_ONE_OR_MORE() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_more(ItemType::Item)].to_vec(),
            SequenceType::one_or_more(ItemType::Item)
        ),
        fn_one_or_more
    )
}

pub(crate) fn fn_one_or_more(env: Box<Environment>, mut arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
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

// fn:exactly-one($arg as item()*) as item()
pub(crate) fn FN_EXACTLY_ONE() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_more(ItemType::Item)].to_vec(),
            SequenceType::exactly_one(ItemType::Item)
        ),
        fn_exactly_one
    )
}

pub(crate) fn fn_exactly_one(env: Box<Environment>, mut arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
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