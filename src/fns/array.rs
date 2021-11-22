use crate::eval::{Object, Type, EvalResult, DynamicContext};
use crate::eval::Environment;
use crate::eval::sequence_type::*;
use crate::fns::FUNCTION;

// array:size($array as array(*)) as xs:integer
pub(crate) fn FN_ARRAY_SIZE() -> FUNCTION {
    (
        (
            [SequenceType::exactly_one(ItemType::Array(None))].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_INTEGER.into()))
        ),
        array_size
    )
}

pub(crate) fn array_size(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Array(array)] => {
            let size = array.len();
            Ok((env, Object::Atomic(Type::Integer(size as i128))))
        }
        _ => panic!("error")
    }
}

// array:get($array as array(*), $position as xs:integer) as item()*
pub(crate) fn FN_ARRAY_GET() -> FUNCTION {
    (
        (
            [
                SequenceType::exactly_one(ItemType::Array(None)),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_INTEGER.into())),
            ].to_vec(),
            SequenceType::zero_or_more(ItemType::Item)
        ),
        array_get
    )
}

pub(crate) fn array_get(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

// array:put($array as array(*), $position as xs:integer, $member as item()*) as array(*)
pub(crate) fn FN_ARRAY_PUT() -> FUNCTION {
    (
        (
            [
                SequenceType::exactly_one(ItemType::Array(None)),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_INTEGER.into())),
                SequenceType::zero_or_more(ItemType::Item)
            ].to_vec(),
            SequenceType::exactly_one(ItemType::Array(None))
        ),
        array_put
    )
}

pub(crate) fn array_put(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

// array:append($array as array(*), $appendage as item()*) as array(*)
pub(crate) fn FN_ARRAY_APPEND() -> FUNCTION {
    (
        (
            [
                SequenceType::exactly_one(ItemType::Array(None)),
                SequenceType::zero_or_more(ItemType::Item)
            ].to_vec(),
            SequenceType::exactly_one(ItemType::Array(None))
        ),
        array_append
    )
}

pub(crate) fn array_append(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            let mut result = array.clone();
            result.push(item.clone());

            Ok((env, Object::Array(result)))
        }

        _ => panic!("error")
    }
}

// array:subarray($array as array(*), $start as xs:integer) as array(*)
pub(crate) fn FN_ARRAY_SUBARRAY_2() -> FUNCTION {
    (
        (
            [
                SequenceType::exactly_one(ItemType::Array(None)),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_INTEGER.into()))
            ].to_vec(),
            SequenceType::exactly_one(ItemType::Array(None))
        ),
        array_subarray
    )
}

// array:subarray($array as array(*), $start as xs:integer, $length as xs:integer) as array(*)
pub(crate) fn FN_ARRAY_SUBARRAY_3() -> FUNCTION {
    (
        (
            [
                SequenceType::exactly_one(ItemType::Array(None)),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_INTEGER.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_INTEGER.into()))
            ].to_vec(),
            SequenceType::exactly_one(ItemType::Array(None))
        ),
        array_subarray
    )
}

pub(crate) fn array_subarray(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

// array:remove($array as array(*), $positions as xs:integer*) as array(*)
pub(crate) fn FN_ARRAY_REMOVE() -> FUNCTION {
    (
        (
            [
                SequenceType::exactly_one(ItemType::Array(None)),
                SequenceType::zero_or_more(ItemType::AtomicOrUnionType(XS_INTEGER.into()))
            ].to_vec(),
            SequenceType::exactly_one(ItemType::Array(None))
        ),
        array_remove
    )
}

pub(crate) fn array_remove(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

// array:insert-before($array as array(*), $position as xs:integer, $member as item()*) as array(*)
pub(crate) fn FN_ARRAY_INSERT_BEFORE() -> FUNCTION {
    (
        (
            [
                SequenceType::exactly_one(ItemType::Array(None)),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_INTEGER.into())),
                SequenceType::zero_or_more(ItemType::Item)
            ].to_vec(),
            SequenceType::exactly_one(ItemType::Array(None))
        ),
        array_insert_before
    )
}

pub(crate) fn array_insert_before(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

// array:head($array as array(*)) as item()*
pub(crate) fn FN_ARRAY_HEAD() -> FUNCTION {
    (
        (
            [SequenceType::exactly_one(ItemType::Array(None))].to_vec(),
            SequenceType::zero_or_more(ItemType::Item)
        ),
        array_head
    )
}

pub(crate) fn array_head(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

// array:tail($array as array(*)) as array(*)
pub(crate) fn FN_ARRAY_TAIL() -> FUNCTION {
    (
        (
            [SequenceType::exactly_one(ItemType::Array(None))].to_vec(),
            SequenceType::exactly_one(ItemType::Array(None))
        ),
        array_tail
    )
}

pub(crate) fn array_tail(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

// array:reverse($array as array(*)) as array(*)
pub(crate) fn FN_ARRAY_REVERSE() -> FUNCTION {
    (
        (
            [SequenceType::exactly_one(ItemType::Array(None))].to_vec(),
            SequenceType::exactly_one(ItemType::Array(None))
        ),
        array_reverse
    )
}

pub(crate) fn array_reverse(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

// array:join($arrays as array(*)*) as array(*)
pub(crate) fn FN_ARRAY_JOIN() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_more(ItemType::Array(None))].to_vec(),
            SequenceType::exactly_one(ItemType::Array(None))
        ),
        array_join
    )
}

pub(crate) fn array_join(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

// array:for-each($array as array(*), $action as function(item()*) as item()*) as array(*)
pub(crate) fn FN_ARRAY_FOR_EACH() -> FUNCTION {
    (
        (
            [
                SequenceType::exactly_one(ItemType::Array(None)),
                SequenceType::exactly_one(ItemType::Function {
                    args: Some([SequenceType::zero_or_more(ItemType::Item)].to_vec()),
                    st: Some(Box::new(SequenceType::zero_or_more(ItemType::Item)))
                })
            ].to_vec(),
            SequenceType::exactly_one(ItemType::Array(None))
        ),
        array_for_each
    )
}

pub(crate) fn array_for_each(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

// array:filter($array as array(*), $function as function(item()*) as xs:boolean) as array(*)
pub(crate) fn FN_ARRAY_FILTER() -> FUNCTION {
    (
        (
            [
                SequenceType::exactly_one(ItemType::Array(None)),
                SequenceType::exactly_one(ItemType::Function {
                    args: Some([SequenceType::zero_or_more(ItemType::Item)].to_vec()),
                    st: Some(Box::new(SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_BOOLEAN.into()))))
                })
            ].to_vec(),
            SequenceType::exactly_one(ItemType::Array(None))
        ),
        array_filter
    )
}

pub(crate) fn array_filter(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

// array:fold-left($array as array(*), $zero as item()*, $function as function(item()*, item()*) as item()*) as item()*
pub(crate) fn FN_ARRAY_FOLD_LEFT() -> FUNCTION {
    (
        (
            [
                SequenceType::exactly_one(ItemType::Array(None)),
                SequenceType::zero_or_more(ItemType::Item),
                SequenceType::exactly_one(ItemType::Function {
                    args: Some(
                        [
                            SequenceType::zero_or_more(ItemType::Item),
                            SequenceType::zero_or_more(ItemType::Item)
                        ].to_vec()
                    ),
                    st: Some(Box::new(SequenceType::zero_or_more(ItemType::Item)))
                })
            ].to_vec(),
            SequenceType::zero_or_more(ItemType::Item)
        ),
        array_fold_left
    )
}

pub(crate) fn array_fold_left(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

// array:fold-right($array as array(*), $zero as item()*, $function as function(item()*, item()*) as item()*) as item()*
pub(crate) fn FN_ARRAY_FOLD_RIGHT() -> FUNCTION {
    (
        (
            [
                SequenceType::exactly_one(ItemType::Array(None)),
                SequenceType::zero_or_more(ItemType::Item),
                SequenceType::exactly_one(ItemType::Function {
                    args: Some(
                        [
                            SequenceType::zero_or_more(ItemType::Item),
                            SequenceType::zero_or_more(ItemType::Item)
                        ].to_vec()
                    ),
                    st: Some(Box::new(SequenceType::zero_or_more(ItemType::Item)))
                })
            ].to_vec(),
            SequenceType::zero_or_more(ItemType::Item)
        ),
        array_fold_right
    )
}

pub(crate) fn array_fold_right(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

// array:for-each-pair($array1 as array(*), $array2 as array(*), $function as function(item()*, item()*) as item()*) as array(*)
pub(crate) fn FN_ARRAY_FOR_EACH_PAIR() -> FUNCTION {
    (
        (
            [
                SequenceType::exactly_one(ItemType::Array(None)),
                SequenceType::exactly_one(ItemType::Array(None)),
                SequenceType::exactly_one(ItemType::Function {
                    args: Some(
                        [
                            SequenceType::zero_or_more(ItemType::Item),
                            SequenceType::zero_or_more(ItemType::Item)
                        ].to_vec()
                    ),
                    st: Some(Box::new(SequenceType::zero_or_more(ItemType::Item)))
                })
            ].to_vec(),
            SequenceType::exactly_one(ItemType::Array(None)),
        ),
        array_for_each_pair
    )
}

pub(crate) fn array_for_each_pair(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

// array:sort($array as array(*)) as array(*)
pub(crate) fn FN_ARRAY_SORT_1() -> FUNCTION {
    (
        (
            [SequenceType::exactly_one(ItemType::Array(None))].to_vec(),
            SequenceType::exactly_one(ItemType::Array(None)),
        ),
        array_sort
    )
}

// array:sort($array as array(*), $collation as xs:string?) as array(*)
pub(crate) fn FN_ARRAY_SORT_2() -> FUNCTION {
    (
        (
            [
                SequenceType::exactly_one(ItemType::Array(None)),
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
            ].to_vec(),
            SequenceType::exactly_one(ItemType::Array(None)),
        ),
        array_sort
    )
}

// array:sort($array as array(*), $collation as xs:string?, $key as function(item()*) as xs:anyAtomicType*) as array(*)
pub(crate) fn FN_ARRAY_SORT_3() -> FUNCTION {
    (
        (
            [
                SequenceType::exactly_one(ItemType::Array(None)),
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::exactly_one(ItemType::Function {
                    args: Some([SequenceType::zero_or_more(ItemType::Item)].to_vec()),
                    st: Some(Box::new(SequenceType::zero_or_more(ItemType::AnyAtomicType)))
                })
            ].to_vec(),
            SequenceType::exactly_one(ItemType::Array(None)),
        ),
        array_sort
    )
}

pub(crate) fn array_sort(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}

// array:flatten($input as item()*) as item()*
pub(crate) fn FN_ARRAY_FLATTEN() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::Item)].to_vec(),
            SequenceType::zero_or_one(ItemType::Item),
        ),
        array_flatten
    )
}

pub(crate) fn array_flatten(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Array(array), item] => {
            todo!()
        }

        _ => panic!("error")
    }
}