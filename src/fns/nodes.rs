use crate::eval::{Environment, Object, DynamicContext, EvalResult, Type};
use crate::parser::errors::ErrorCode;

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

pub(crate) fn fn_lang(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    println!("arguments {:?}", arguments);
    todo!()
}

pub(crate) fn fn_root(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    println!("arguments {:?}", arguments);
    todo!()
}

pub(crate) fn fn_path(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    println!("arguments {:?}", arguments);
    todo!()
}

pub(crate) fn fn_has_children(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    println!("arguments {:?}", arguments);
    todo!()
}

pub(crate) fn fn_innermost(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    println!("arguments {:?}", arguments);
    todo!()
}

pub(crate) fn fn_outermost(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    println!("arguments {:?}", arguments);
    todo!()
}