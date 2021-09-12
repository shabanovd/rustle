use crate::eval::{Object, Type, NumberCase, EvalResult};
use crate::eval::Environment;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;

pub fn fn_abs<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> EvalResult<'a> {

    match arguments.as_slice() {
        [Object::Atomic(Type::Integer(num))] => {
            Ok((env, Object::Atomic(Type::Integer((*num).abs()))))
        },
        _ => panic!("error")
    }
}