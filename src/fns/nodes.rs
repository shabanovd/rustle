use crate::eval::{Environment, Object, DynamicContext, EvalResult, Type, Node};
use crate::parser::errors::ErrorCode;

pub(crate) fn fn_name<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    println!("arguments {:?}", arguments);
    todo!()
}

pub(crate) fn fn_local_name<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult<'a> {
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
        Object::Node(node) => {
            match node {
                Node::Element { name, .. } |
                Node::Attribute { name, .. } => {
                    Ok((env, Object::Atomic(Type::String(name.local_part.clone()))))
                },
                _ => Ok((env, Object::Atomic(Type::String(String::new())))),
            }

        }
        _ => Err((ErrorCode::XPTY0004, "TODO".to_string()))
    }
}

pub(crate) fn fn_namespace_uri<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    println!("arguments {:?}", arguments);
    todo!()
}

pub(crate) fn fn_lang<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    println!("arguments {:?}", arguments);
    todo!()
}

pub(crate) fn fn_root<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    println!("arguments {:?}", arguments);
    todo!()
}

pub(crate) fn fn_path<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    println!("arguments {:?}", arguments);
    todo!()
}

pub(crate) fn fn_has_children<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    println!("arguments {:?}", arguments);
    todo!()
}

pub(crate) fn fn_innermost<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    println!("arguments {:?}", arguments);
    todo!()
}

pub(crate) fn fn_outermost<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    println!("arguments {:?}", arguments);
    todo!()
}