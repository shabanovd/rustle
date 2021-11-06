use crate::eval::{Object, DynamicContext, EvalResult};
use crate::eval::Environment;

use crate::serialization::object_to_string;
use crate::parser::errors::ErrorCode;
use crate::values::{AnyURI, QName};

pub(crate) fn fn_resolve_qname(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
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

    Ok((env, Object::Atomic( QName { url, prefix, local_part } )))
}

pub(crate) fn fn_prefix_from_qname(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    println!("arguments {:?}", arguments);
    todo!()
}

pub(crate) fn fn_local_name_from_qname(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    println!("arguments {:?}", arguments);
    todo!()
}

pub(crate) fn fn_namespace_uri_from_qname(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    println!("arguments {:?}", arguments);
    match arguments.as_slice() {
        [Object::Empty] => Ok((env, Object::Empty)),
        [Object::Atomic(QName { url, .. })] => {
            if let Some(uri) = url {
                Ok((env, Object::Atomic(AnyURI::boxed(uri.clone()))))
            } else {
                Ok((env, Object::Atomic(AnyURI::boxed(String::new()))))
            }
        },
        _ => panic!()
    }
}

pub(crate) fn fn_namespace_uri_for_prefix(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    println!("arguments {:?}", arguments);
    todo!()
}

pub(crate) fn fn_in_scope_prefixes(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    println!("arguments {:?}", arguments);
    todo!()
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
                Ok((env, Object::Atomic(Box::new(QName {
                    url: name.url.clone(),
                    prefix: name.prefix.clone(),
                    local_part: name.local_part.clone()
                }))))
            } else {
                Ok((env, Object::Empty))
            }
        },
        _ => panic!("TODO")
    }
}