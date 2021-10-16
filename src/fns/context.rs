use crate::eval::{Environment, Object, DynamicContext, EvalResult, Type};

pub(crate) fn fn_default_collation<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult<'a> {
    todo!()
}

pub(crate) fn fn_default_language<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult<'a> {
    todo!()
}

pub(crate) fn fn_static_base_uri<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult<'a> {
    let result = if let Some(uri) = &env.static_base_uri {
        Object::Atomic(Type::AnyURI(uri.clone()))
    } else {
        Object::Empty
    };
    Ok((env, result))
}