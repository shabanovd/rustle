//use crate::eval::Type;
use crate::eval::{Object, eval_statements};
use crate::eval::Environment;

use std::collections::HashMap;
use crate::value::{resolve_function_qname, resolve_element_qname};

pub fn apply<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {

    let mut current_env = env;

    match arguments.as_slice() {
        [Object::Function { parameters, body }, Object::Array( arguments )] => {

            assert_eq!(parameters.len(), arguments.len(), "wrong number of parameters");

            let mut function_environment = Environment::new();
            for (parameter, argument) in parameters.into_iter().zip(arguments.into_iter()) {

                let name = resolve_element_qname(parameter.name.clone(), &current_env);

                function_environment.set(name, argument.clone());
            }

            let (_, result) = eval_statements(body.clone(), Box::new(function_environment), context_item);

            (current_env, result)
        }
        _ => panic!("error")
    }
}