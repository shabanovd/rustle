use crate::eval::{Environment, Object, Type, EvalResult};
use bigdecimal::{BigDecimal, FromPrimitive};

pub fn fn_count<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context_item: &Object) -> EvalResult<'a> {
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

pub fn fn_avg<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context_item: &Object) -> EvalResult<'a> {

    match arguments.as_slice() {
        [Object::Empty] => {
            Ok((env, Object::Empty))
        },
        [Object::Sequence(items)] => {
            // xs:untypedAtomic => xs:double
            // xs:double, xs:float, xs:decimal, xs:yearMonthDuration, xs:dayTimeDuration
            let mut sum = 0;
            let mut count: usize = 0;
            for item in items {
                match item {
                    Object::Atomic(Type::Integer(num)) => {
                        sum += num;
                        count += 1;
                    },
                    _ => panic!("error")
                }
            }

            let sum = BigDecimal::from_i128(sum).unwrap(); // TODO code it
            let count = BigDecimal::from_usize(count).unwrap(); // TODO code it

            let number = sum / count;

            Ok((env, Object::Atomic(Type::Decimal(number))))
        },
        _ => panic!("error")
    }
}