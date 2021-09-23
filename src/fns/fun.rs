use crate::eval::{Object, eval_expr, object_to_iterator, EvalResult, DynamicContext};
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

pub(crate) fn apply<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult<'a> {

    let mut current_env = env;

    match arguments.as_slice() {
        [Object::Function { parameters, body }, Object::Array( arguments )] => {

            assert_eq!(parameters.len(), arguments.len(), "wrong number of parameters");

            let mut function_environment = Environment::new();
            for (parameter, argument) in parameters.into_iter().zip(arguments.into_iter()) {

                let name = resolve_element_qname(&parameter.name, &current_env);

                function_environment.set(name, argument.clone());
            }

            let (_, result) = eval_expr(*body.clone(), Box::new(function_environment), &DynamicContext::nothing())?;

            Ok((current_env, result))
        },
        [Object::FunctionRef { name, arity }, Object::Array( arguments )] => {
            let fun = current_env.functions.get(&name, *arity);

            return if fun.is_some() {
                fun.unwrap()(current_env, arguments.clone(), context)
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
