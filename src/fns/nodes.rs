use crate::eval::{Environment, Object, DynamicContext, EvalResult, Type};
use crate::eval::sequence_type::*;
use crate::fns::FUNCTION;
use crate::parser::errors::ErrorCode;

// fn:name() as xs:string
pub(crate) fn FN_NAME_0() -> FUNCTION {
    (
        (
            [].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
        ),
        fn_name
    )
}

// fn:name($arg as node()?) as xs:string
pub(crate) fn FN_NAME_1() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::node())].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
        ),
        fn_name
    )
}

pub(crate) fn fn_name(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    let item = if arguments.len() == 0 {
        if context.item == Object::Nothing {
            return Err((ErrorCode::XPDY0002, "context item is absent".to_string()))
        }
        &context.item
    } else {
        arguments.get(0).unwrap()
    };

    match item {
        Object::Empty => Ok((env, Object::Atomic(Type::String(String::new())))),
        Object::Node(rf) => {
            let data = if let Some(name) = rf.name() {
                name.string()
            } else {
                String::new()
            };

            Ok((env, Object::Atomic(Type::String(data))))
        },
        _ => Err((ErrorCode::XPTY0004, String::from("TODO")))
    }
}

// fn:local-name() as xs:string
pub(crate) fn FN_LOCAL_NAME_0() -> FUNCTION {
    (
        (
            [].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
        ),
        fn_local_name
    )
}

// fn:local-name($arg as node()?) as xs:string
pub(crate) fn FN_LOCAL_NAME_1() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::node())].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
        ),
        fn_local_name
    )
}

pub(crate) fn fn_local_name(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    let item = if arguments.len() == 0 {
        if context.item == Object::Nothing {
            return Err((ErrorCode::XPDY0002, "context item is absent".to_string()))
        }
        &context.item
    } else {
        arguments.get(0).unwrap()
    };

    match item {
        Object::Empty => Ok((env, Object::Atomic(Type::String(String::new())))),
        Object::Node(rf) => {
            if let Some(name) = rf.name() {
                Ok((env, Object::Atomic(Type::String(name.local_part))))
            } else {
                Ok((env, Object::Atomic(Type::String(String::new()))))
            }

        }
        _ => Err((ErrorCode::XPTY0004, "TODO".to_string()))
    }
}

// fn:namespace-uri() as xs:anyURI
pub(crate) fn FN_NAMESPACE_URI_0() -> FUNCTION {
    (
        (
            [].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_ANY_URI.into()))
        ),
        fn_namespace_uri
    )
}

// fn:namespace-uri($arg as node()?) as xs:anyURI
pub(crate) fn FN_NAMESPACE_URI_1() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::node())].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_ANY_URI.into()))
        ),
        fn_namespace_uri
    )
}

pub(crate) fn fn_namespace_uri(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    let item = if arguments.len() == 0 {
        if context.item == Object::Nothing {
            return Err((ErrorCode::XPDY0002, "context item is absent".to_string()))
        }
        &context.item
    } else {
        arguments.get(0).unwrap()
    };

    match item {
        Object::Node(rf) => {
            let url = if let Some(name) = rf.name() {
                if let Some(url) = name.url {
                    url
                } else {
                    String::from("")
                }
            } else {
                String::from("")
            };
            Ok((env, Object::Atomic(Type::String(url))))
        }
        _ => Err((ErrorCode::XPTY0004, "context item is not a node".to_string()))
    }
}

// fn:lang($testlang as xs:string?) as xs:boolean
pub(crate) fn FN_LANG_1() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into()))].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_BOOLEAN.into()))
        ),
        fn_lang
    )
}

// fn:lang($testlang as xs:string?, $node as node()) as xs:boolean
pub(crate) fn FN_LANG_2() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::exactly_one(ItemType::node()),
            ].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_BOOLEAN.into()))
        ),
        fn_lang
    )
}

pub(crate) fn fn_lang(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    println!("arguments {:?}", arguments);
    todo!()
}

// fn:root() as node()
pub(crate) fn FN_ROOT_0() -> FUNCTION {
    (
        (
            [].to_vec(),
            SequenceType::exactly_one(ItemType::node())
        ),
        fn_root
    )
}

// fn:root($arg as node()?) as node()?
pub(crate) fn FN_ROOT_1() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::node())].to_vec(),
            SequenceType::zero_or_one(ItemType::node())
        ),
        fn_root
    )
}

pub(crate) fn fn_root(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    println!("arguments {:?}", arguments);
    todo!()
}

// fn:path() as xs:string?
pub(crate) fn FN_PATH_0() -> FUNCTION {
    (
        (
            [].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
        ),
        fn_path
    )
}

// fn:path($arg as node()?) as xs:string?
pub(crate) fn FN_PATH_1() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::node())].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
        ),
        fn_path
    )
}

pub(crate) fn fn_path(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    println!("arguments {:?}", arguments);
    todo!()
}

// fn:has-children() as xs:boolean
pub(crate) fn FN_HAS_CHILDREN_0() -> FUNCTION {
    (
        (
            [].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_BOOLEAN.into()))
        ),
        fn_has_children
    )
}

// fn:has-children($node as node()?) as xs:boolean
pub(crate) fn FN_HAS_CHILDREN_1() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::node())].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_BOOLEAN.into()))
        ),
        fn_has_children
    )
}

pub(crate) fn fn_has_children(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    println!("arguments {:?}", arguments);
    todo!()
}

// fn:innermost($nodes as node()*) as node()*
pub(crate) fn FN_INNERMOST() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_more(ItemType::node())].to_vec(),
            SequenceType::zero_or_more(ItemType::node())
        ),
        fn_innermost
    )
}

pub(crate) fn fn_innermost(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    println!("arguments {:?}", arguments);
    todo!()
}

// fn:outermost($nodes as node()*) as node()*
pub(crate) fn FN_OUTERMOST() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_more(ItemType::node())].to_vec(),
            SequenceType::zero_or_more(ItemType::node())
        ),
        fn_outermost
    )
}

pub(crate) fn fn_outermost(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    println!("arguments {:?}", arguments);
    todo!()
}