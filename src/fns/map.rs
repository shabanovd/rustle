use crate::eval::{Object, EvalResult, DynamicContext};
use crate::eval::Environment;

use std::collections::HashMap;
use crate::eval::sequence_type::*;
use crate::fns::FUNCTION;

// op:same-key($k1 as xs:anyAtomicType, $k2 as xs:anyAtomicType) as xs:boolean

// map:merge($maps as map(*)*) as map(*)
pub(crate) fn FN_MAP_MERGE_1() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_more(ItemType::Map)].to_vec(),
            SequenceType::exactly_one(ItemType::Map)
        ),
        fn_map_merge
    )
}

// map:merge($maps as map(*)*, $options as map(*)) as map(*)
pub(crate) fn FN_MAP_MERGE_2() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_more(ItemType::Map),
                SequenceType::exactly_one(ItemType::Map),
            ].to_vec(),
            SequenceType::exactly_one(ItemType::Map)
        ),
        fn_map_merge
    )
}

pub(crate) fn fn_map_merge(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// map:size($map as map(*)) as xs:integer
pub(crate) fn FN_MAP_SIZE() -> FUNCTION {
    (
        (
            [SequenceType::exactly_one(ItemType::Map)].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_INTEGER.into()))
        ),
        map_size
    )
}

pub(crate) fn map_size(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// map:keys($map as map(*)) as xs:anyAtomicType*
pub(crate) fn FN_MAP_KEYS() -> FUNCTION {
    (
        (
            [SequenceType::exactly_one(ItemType::Map)].to_vec(),
            SequenceType::zero_or_more(ItemType::AnyAtomicType)
        ),
        map_keys
    )
}

pub(crate) fn map_keys(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// map:contains($map as map(*), $key as xs:anyAtomicType) as xs:boolean
pub(crate) fn FN_MAP_CONTAINS() -> FUNCTION {
    (
        (
            [
                SequenceType::exactly_one(ItemType::Map),
                SequenceType::exactly_one(ItemType::AnyAtomicType),
            ].to_vec(),
            SequenceType::zero_or_more(ItemType::AtomicOrUnionType(XS_BOOLEAN.into()))
        ),
        map_contains
    )
}

pub(crate) fn map_contains(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// map:get($map as map(*), $key as xs:anyAtomicType) as item()*
pub(crate) fn FN_MAP_GET() -> FUNCTION {
    (
        (
            [
                SequenceType::exactly_one(ItemType::Map),
                SequenceType::exactly_one(ItemType::AnyAtomicType),
            ].to_vec(),
            SequenceType::zero_or_more(ItemType::Item)
        ),
        map_get
    )
}

pub(crate) fn map_get(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Map(map), Object::Atomic(k)] => {

            // println!("map_get {:?} {:?}", k, map);

            if let Some(value) = map.get(k) {
                Ok((env, value.clone()))
            } else {
                Ok((env, Object::Empty))
            }
        }

        _ => panic!("error")
    }
}

// map:find($input as item()*, $key as xs:anyAtomicType) as array(*)
pub(crate) fn FN_MAP_FIND() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_more(ItemType::Item),
                SequenceType::exactly_one(ItemType::AnyAtomicType),
            ].to_vec(),
            SequenceType::exactly_one(ItemType::Array(None))
        ),
        map_find
    )
}

pub(crate) fn map_find(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// map:put($map as map(*), $key as xs:anyAtomicType, $value as item()*) as map(*)
pub(crate) fn FN_MAP_PUT() -> FUNCTION {
    (
        (
            [
                SequenceType::exactly_one(ItemType::Map),
                SequenceType::exactly_one(ItemType::AnyAtomicType),
                SequenceType::zero_or_more(ItemType::Item),
            ].to_vec(),
            SequenceType::exactly_one(ItemType::Map)
        ),
        map_put
    )
}

pub(crate) fn map_put(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// map:entry($key as xs:anyAtomicType, $value as item()*) as map(*)
pub(crate) fn FN_MAP_ENTRY() -> FUNCTION {
    (
        (
            [
                SequenceType::exactly_one(ItemType::AnyAtomicType),
                SequenceType::zero_or_more(ItemType::Item),
            ].to_vec(),
            SequenceType::exactly_one(ItemType::Map)
        ),
        map_entry
    )
}

pub(crate) fn map_entry(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Atomic(k), Object::Atomic(v)] => {

            let mut map = HashMap::new();

            map.insert(k.clone(), Object::Atomic(v.clone())); //TODO: understand, is it possible to avoid clone?

            Ok((env, Object::Map(map)))
        }

        _ => panic!("error")
    }
}

// map:remove($map as map(*), $keys as xs:anyAtomicType*) as map(*)
pub(crate) fn FN_MAP_REMOVE() -> FUNCTION {
    (
        (
            [
                SequenceType::exactly_one(ItemType::Map),
                SequenceType::zero_or_more(ItemType::AnyAtomicType),
            ].to_vec(),
            SequenceType::exactly_one(ItemType::Map)
        ),
        map_remove
    )
}

pub(crate) fn map_remove(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// map:for-each($map as map(*), $action as function(xs:anyAtomicType, item()*) as item()*) as item()*
pub(crate) fn FN_MAP_FOR_EACH() -> FUNCTION {
    (
        (
            [
                SequenceType::exactly_one(ItemType::Map),
                SequenceType::exactly_one(ItemType::Function {
                    args: Some([
                        SequenceType::exactly_one(ItemType::AnyAtomicType),
                        SequenceType::zero_or_more(ItemType::Item)
                    ].to_vec()),
                    st: Some(Box::new(SequenceType::zero_or_more(ItemType::Item)))
                }),
                SequenceType::zero_or_more(ItemType::AnyAtomicType),
            ].to_vec(),
            SequenceType::exactly_one(ItemType::Map)
        ),
        map_for_each
    )
}

pub(crate) fn map_for_each(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}