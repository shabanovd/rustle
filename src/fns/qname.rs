use crate::eval::{Object, Type, EvalResult};
use crate::eval::Environment;

use crate::serialization::object_to_string;
use crate::parser::errors::ErrorCode;

pub fn fn_resolve_qname<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context_item: &Object) -> EvalResult<'a> {
    todo!()
}

pub fn fn_qname<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context_item: &Object) -> EvalResult<'a> {
    let url = arguments.get(0).unwrap();
    let qname = arguments.get(1).unwrap();

    let url = match url {
        Object::Empty => None,
        Object::Atomic(..) => {
            Some(object_to_string(url))
        },
        _ => {
            return Err((ErrorCode::FOCA0002, String::from("TODO")));
        }
    };
    let qname = object_to_string(qname);

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