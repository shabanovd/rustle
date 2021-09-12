use crate::eval::{Environment, Object, Type, NumberCase, EvalResult};
use rust_decimal::{Decimal, prelude::FromPrimitive};

pub fn fn_count<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> EvalResult<'a> {
    match arguments.as_slice() {
        [Object::Empty] => {
            Ok((env, Object::Atomic(Type::Integer(0))))
        },
        [Object::Sequence(items)] => {
            Ok((env, Object::Atomic(Type::Integer(items.len() as i128))))
        }
        _ => panic!("error")
    }
}

pub fn fn_avg<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> EvalResult<'a> {

    match arguments.as_slice() {
        [Object::Empty] => {
            Ok((env, Object::Empty))
        },
        [Object::Sequence(items)] => {
            // xs:untypedAtomic => xs:double
            // xs:double, xs:float, xs:decimal, xs:yearMonthDuration, xs:dayTimeDuration
            let mut sum = 0;
            let mut count = 0;
            for item in items {
                match item {
                    Object::Atomic(Type::Integer(num)) => {
                        sum += num;
                        count += 1;
                    },
                    _ => panic!("error")
                }
            }
            let result = sum as f32 / count as f32;
            if let Some(number) = Decimal::from_f32(result) {
                Ok((env, Object::Atomic(Type::Decimal { number: Some(number), case: NumberCase::Normal })))
            } else {
                panic!("error")
            }
        },
        _ => panic!("error")
    }
}