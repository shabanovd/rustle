use crate::eval::Type;
use crate::eval::Object;
use crate::eval::Environment;
use crate::parser::Expr;

pub fn xs_decimal_eval<'a>(env: &'a Environment<'a>, arguments: Vec<Object>) -> (&'a Environment<'a>, Object) {
    match arguments.as_slice() {

        [Object::Atomic(Type::String(string))] => (env, Object::Atomic(Type::Integer(string.parse::<i128>().unwrap()))),

        [Object::Atomic(Type::Integer(integer))] => (env, Object::Atomic(Type::Integer(*integer))),

        _ => (env, Object::Empty), // TODO: raise error?
    }
}