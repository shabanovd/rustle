use crate::eval::{Object, Type};
use crate::eval::Environment;

pub fn fn_abs<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {

    match arguments.as_slice() {
        [Object::Atomic(Type::Integer(num))] => {
            (env, Object::Atomic(Type::Integer((*num).abs())))
        },
        _ => panic!("error")
    }
}