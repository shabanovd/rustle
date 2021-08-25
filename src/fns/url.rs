use crate::eval::Type;
use crate::eval::Object;
use crate::eval::Environment;

pub fn xs_anyuri_eval<'a>(env: &'a Environment<'a>, arguments: Vec<Object>) -> (&'a Environment<'a>, Object) {
    match arguments.as_slice() {

        [Object::Atomic(Type::String(string))] => (env, Object::Atomic(Type::AnyURI(String::from(string)))),

        _ => (env, Object::Empty), // TODO: raise error?
    }
}