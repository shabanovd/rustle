use crate::eval::Object;
use crate::eval::Environment;

pub fn xs_decimal_eval<'a>(env: &'a mut Environment<'a>, arguments: Vec<Object>) -> (&'a mut Environment<'a>, Object) {
    match arguments.as_slice() {
        [Object::String(string)] => (env, Object::Integer(string.parse::<i128>().unwrap())),
        [Object::Integer(integer)] => (env, Object::Integer(*integer)),
        _ => (env, Object::Empty),
    }
}