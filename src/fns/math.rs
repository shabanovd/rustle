use crate::eval::{Object, Type, NumberCase};
use crate::eval::Environment;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;

pub fn fn_avg<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {

    match arguments.as_slice() {
        [Object::Empty] => {
            (env, Object::Empty)
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
                (env, Object::Atomic(Type::Decimal { number: Some(number), case: NumberCase::Normal }))
            } else {
                panic!("error")
            }
        },
        _ => panic!("error")
    }
}

pub fn fn_abs<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {

    match arguments.as_slice() {
        [Object::Atomic(Type::Integer(num))] => {
            (env, Object::Atomic(Type::Integer((*num).abs())))
        },
        _ => panic!("error")
    }
}