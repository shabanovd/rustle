use crate::eval::{Object, Type, DynamicContext, EvalResult};
use crate::eval::Environment;
use crate::eval::sequence_type::*;
use crate::fns::FUNCTION;

use crate::serialization::object_to_string;
use crate::parser::errors::ErrorCode;

// fn:resolve-QName($qname as xs:string?, $element as element()) as xs:QName?
pub(crate) fn FN_RESOLVE_QNAME() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::zero_or_one(ItemType::element())
            ].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_QNAME.into()))
        ),
        fn_resolve_qname
    )
}

pub(crate) fn fn_resolve_qname(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// fn:QName($paramURI as xs:string?, $paramQName as xs:string) as xs:QName
pub(crate) fn FN_QNAME() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
            ].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_QNAME.into()))
        ),
        fn_qname
    )
}

pub(crate) fn fn_qname(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    let url = arguments.get(0).unwrap();
    let qname = arguments.get(1).unwrap();

    let url = match url {
        Object::Empty => None,
        Object::Atomic(..) => {
            Some(object_to_string(&env, url))
        },
        _ => {
            return Err((ErrorCode::FOCA0002, String::from("TODO")));
        }
    };
    let qname = object_to_string(&env, qname);

    let mut parts = qname.split(":");
    let (prefix, local_part) = if let Some(p1) = parts.next() {
        if let Some(p2) = parts.next() {
            if let Some(..) = parts.next() {
                return Err((ErrorCode::FOCA0002, String::from("TODO")));
            }
            (Some(String::from(p1)), String::from(p2))
        } else {
            (None, String::from(p1))
        }
    } else {
        return Err((ErrorCode::FOCA0002, String::from("TODO")));
    };

    Ok((env, Object::Atomic( Type::QName { url, prefix, local_part } )))
}

// TODO op:QName-equal($arg1 as xs:QName, $arg2 as xs:QName) as xs:boolean

// fn:prefix-from-QName($arg as xs:QName?) as xs:NCName?
pub(crate) fn FN_PREFIX_FROM_QNAME() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_QNAME.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_NCNAME.into()))
        ),
        fn_prefix_from_qname
    )
}

pub(crate) fn fn_prefix_from_qname(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    println!("arguments {:?}", arguments);
    todo!()
}

// fn:local-name-from-QName($arg as xs:QName?) as xs:NCName?
pub(crate) fn FN_LOCAL_NAME_FROM_QNAME() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_QNAME.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_NCNAME.into()))
        ),
        fn_local_name_from_qname
    )
}

pub(crate) fn fn_local_name_from_qname(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Empty] => Ok((env, Object::Empty)),
        [Object::Atomic(Type::QName { local_part, .. })] => {
            Ok((env, Object::Atomic(Type::NCName(local_part.clone()))))
        },
        _ => panic!()
    }
}

// fn:namespace-uri-from-QName($arg as xs:QName?) as xs:anyURI?
pub(crate) fn FN_NAMESPACE_URI_FROM_QNAME() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_QNAME.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_ANY_URI.into()))
        ),
        fn_namespace_uri_from_qname
    )
}

pub(crate) fn fn_namespace_uri_from_qname(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Empty] => Ok((env, Object::Empty)),
        [Object::Atomic(Type::QName { url, .. })] => {
            if let Some(uri) = url {
                Ok((env, Object::Atomic(Type::AnyURI(uri.clone()))))
            } else {
                Ok((env, Object::Atomic(Type::AnyURI(String::new()))))
            }
        },
        _ => panic!("{:?}", arguments)
    }
}

// fn:namespace-uri-for-prefix($prefix as xs:string?, $element as element()) as xs:anyURI?
pub(crate) fn FN_NAMESPACE_URI_FOR_PREFIX() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::zero_or_one(ItemType::element())
            ].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_ANY_URI.into()))
        ),
        fn_namespace_uri_for_prefix
    )
}

pub(crate) fn fn_namespace_uri_for_prefix(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    println!("arguments {:?}", arguments);
    todo!()
}

// fn:in-scope-prefixes($element as element()) as xs:string*
pub(crate) fn FN_IN_SCOPE_PREFIXES() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::element())].to_vec(),
            SequenceType::zero_or_more(ItemType::AtomicOrUnionType(XS_STRING.into()))
        ),
        fn_in_scope_prefixes
    )
}

pub(crate) fn fn_in_scope_prefixes(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    println!("arguments {:?}", arguments);
    todo!()
}

// fn:node-name() as xs:QName?
pub(crate) fn FN_NODE_NAME_0() -> FUNCTION {
    (
        (
            [].to_vec(),
            SequenceType::zero_or_more(ItemType::AtomicOrUnionType(XS_QNAME.into()))
        ),
        fn_node_name
    )
}

// fn:node-name($arg as node()?) as xs:QName?
pub(crate) fn FN_NODE_NAME_1() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::node())].to_vec(),
            SequenceType::zero_or_more(ItemType::AtomicOrUnionType(XS_QNAME.into()))
        ),
        fn_node_name
    )
}

pub(crate) fn fn_node_name(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    println!("arguments {:?}", arguments);
    let item = if arguments.len() == 0 {
        &context.item
    } else {
        arguments.get(0).unwrap()
    };

    match item {
        Object::Empty => Ok((env, Object::Empty)),
        Object::Node(rf) => {
            if let Some(name) = rf.name() {
                Ok((env, Object::Atomic(Type::QName {
                    url: name.url.clone(),
                    prefix: name.prefix.clone(),
                    local_part: name.local_part.clone()
                })))
            } else {
                Ok((env, Object::Empty))
            }
        },
        _ => panic!("TODO")
    }
}