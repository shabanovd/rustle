//use crate::eval::Type;
use crate::eval::{Object, Type};
use crate::eval::Environment;

use std::collections::HashMap;
use crate::value::{resolve_function_qname, resolve_element_qname};

pub fn fn_true<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {
    // TODO: check arity?
    (env, Object::Atomic(Type::Boolean(true)))
}

pub fn fn_false<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {
    // TODO: check arity?
    (env, Object::Atomic(Type::Boolean(false)))
}