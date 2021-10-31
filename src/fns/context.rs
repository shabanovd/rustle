use crate::eval::{Environment, Object, DynamicContext, EvalResult, Type};

pub(crate) fn fn_default_collation(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    todo!()
}

pub(crate) fn fn_default_language(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    todo!()
}

pub(crate) fn fn_static_base_uri(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    let result = if let Some(uri) = &env.static_base_uri {
        Object::Atomic(Type::AnyURI(uri.clone()))
    } else {
        Object::Empty
    };
    Ok((env, result))
}