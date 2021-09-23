use crate::eval::{Environment, Object, Type, EvalResult, DynamicContext};
use bigdecimal::{BigDecimal, FromPrimitive};

pub(crate) fn fn_count<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {
    match arguments.as_slice() {
        [Object::Empty] => {
            Ok((env, Object::Atomic(Type::Integer(0))))
        },
        [Object::Range { min, max}] => {
            let min = *min;
            let max = *max;
            let count = if min <= max {
                max - min + 1
            } else {
                min - max + 1
            };
            Ok((env, Object::Atomic(Type::Integer(count))))
        },
        [Object::Sequence(items)] => {
            Ok((env, Object::Atomic(Type::Integer(items.len() as i128))))
        }
        _ => panic!("error {:?}", arguments)
    }
}

pub(crate) fn fn_avg<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult<'a> {

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