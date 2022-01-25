use crate::eval::{Environment, Object, DynamicContext, EvalResult, Type};
use crate::eval::sequence_type::*;
use crate::fns::FUNCTION;
use crate::values::string_to_any_uri;

// fn:default-collation() as xs:string
pub(crate) fn FN_DEFAULT_COLLATION() -> FUNCTION {
    (
        (
            [].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
        ),
        fn_default_collation
    )
}

pub(crate) fn fn_default_collation(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    todo!()
}

// fn:default-language() as xs:language
pub(crate) fn FN_DEFAULT_LANGUAGE() -> FUNCTION {
    (
        (
            [].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_LANGUAGE.into())),
        ),
        fn_default_language
    )
}

pub(crate) fn fn_default_language(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    todo!()
}

// fn:static-base-uri() as xs:anyURI?
pub(crate) fn FN_STATIC_BASE_URI() -> FUNCTION {
    (
        (
            [].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_ANY_URI.into())),
        ),
        fn_static_base_uri
    )
}

pub(crate) fn fn_static_base_uri(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    let result = if let Some(uri) = &env.static_base_uri {
        let uri = string_to_any_uri(uri)?;
        Object::Atomic(Type::AnyURI(uri))
    } else {
        Object::Empty
    };
    Ok((env, result))
}