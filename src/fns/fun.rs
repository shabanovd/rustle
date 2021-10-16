use crate::eval::{Object, object_to_iterator, DynamicContext, EvalResult};
use crate::eval::Environment;

use crate::values::resolve_element_qname;
use crate::fns::call;
use crate::fns::strings::object_to_array;
use crate::parser::errors::ErrorCode;

pub(crate) fn for_each<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    match arguments.as_slice() {
        [Object::Function { parameters, body }, Object::Array(arguments)] => {
            todo!()
        },
        _ => panic!("error")
    }
}

pub(crate) fn filter<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    match arguments.as_slice() {
        [Object::Function { parameters, body }, Object::Array(arguments)] => {
            todo!()
        },
        _ => panic!("error")
    }
}

pub(crate) fn fold_left<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult<'a> {
    match arguments.as_slice() {
        [seq, Object::Array(array), Object::FunctionRef { name, arity }] => {
            let mut result = array.clone();

            let mut current_env = env;

            let it = object_to_iterator(seq);
            for item in it {
                let arguments = vec![Object::Array(result), item.clone()];
                let (new_env, obj) = call(current_env, name.clone(), arguments, context)?;
                current_env = new_env;

                result = object_to_array(obj);
            }

            if result.is_empty() {
                Ok((current_env, Object::Empty))
            } else {
                Ok((current_env, Object::Array(result)))
            }
        },
        _ => panic!("error")
    }
}

pub(crate) fn fold_right<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    match arguments.as_slice() {
        [Object::Function { parameters, body }, Object::Array(arguments)] => {
            todo!()
        },
        _ => panic!("error")
    }
}

pub(crate) fn for_each_pair<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    match arguments.as_slice() {
        [Object::Function { parameters, body }, Object::Array(arguments)] => {
            todo!()
        },
        _ => panic!("error")
    }
}

pub(crate) fn sort<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    match arguments.as_slice() {
        [Object::Function { parameters, body }, Object::Array(arguments)] => {
            todo!()
        },
        _ => panic!("error")
    }
}

pub(crate) fn apply<'a>(env: Box<Environment<'a>>, mut arguments: Vec<Object>, context: &DynamicContext) -> EvalResult<'a> {
    let mut current_env = env;

    let arg1 = arguments.remove(0);
    let arg2 = arguments.remove(0);

    match (arg1, arg2) {
        (Object::Function { parameters, body }, Object::Array( arguments )) => {

            assert_eq!(parameters.len(), arguments.len(), "wrong number of parameters");

            let mut fn_env = current_env.next();
            for (parameter, argument) in parameters.into_iter().zip(arguments.into_iter()) {

                let name = resolve_element_qname(&parameter.name, &fn_env);

                fn_env.set(name, argument);
            }

            let (new_env, result) = body.eval(fn_env, &DynamicContext::nothing())?;
            current_env = new_env.prev();

            Ok((current_env, result))
        },
        (Object::FunctionRef { name, arity }, Object::Array( arguments )) => {
            let fun = current_env.functions.get(&name, arity);

            return if fun.is_some() {
                fun.unwrap()(current_env, arguments, context)
            } else {
                panic!("no function {:?}#{:?}", name, arity)
            }
        },
        _ => panic!("error")
    }
}

pub(crate) fn error<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    match arguments.as_slice() {
        [] => {
            Err((ErrorCode::FOER0000, String::new()))
        },
        _ => panic!("error")
    }
}
