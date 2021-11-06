use crate::eval::{Environment, Object, EvalResult, DynamicContext, comparison};
use bigdecimal::{BigDecimal, FromPrimitive};
use crate::values::{Decimal, Integer};

pub(crate) fn fn_count(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Empty] => {
            Ok((env, Object::Atomic(Integer::boxed(0))))
        },
        [Object::Atomic(..)] => {
            Ok((env, Object::Atomic(Integer::boxed(1))))
        }
        [Object::Range { min, max}] => {
            let min = *min;
            let max = *max;
            let count = if min <= max {
                max - min + 1
            } else {
                min - max + 1
            };
            Ok((env, Object::Atomic(Integer::boxed(count))))
        },
        [Object::Sequence(items)] => {
            let mut count = 0;
            for item in items {
                match item {
                    Object::Range { min, max } => {
                        if min <= max {
                            count += (max - min) + 1;
                        } else {
                            count += (min - max) + 1;
                        }
                    },
                    _ => count += 1
                }
            }
            Ok((env, Object::Atomic(Integer::boxed(count))))
        },
        [Object::Node(..)] => {
            Ok((env, Object::Atomic(Integer::boxed(1))))
        }
        _ => panic!("error {:?}", arguments)
    }
}

pub(crate) fn fn_avg(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {

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
                    Object::Atomic(Integer(num)) => {
                        sum += num;
                        count += 1;
                    },
                    _ => panic!("error")
                }
            }

            let sum = BigDecimal::from_i128(sum).unwrap(); // TODO code it
            let count = BigDecimal::from_usize(count).unwrap(); // TODO code it

            let number = sum / count;

            Ok((env, Object::Atomic(Decimal::boxed(number))))
        },
        _ => panic!("error")
    }
}

pub(crate) fn fn_max(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Empty] => {
            Ok((env, Object::Empty))
        },
        [Object::Range { min, max}] => {
            Ok((env, Object::Atomic(Integer::boxed(*max))))
        },
        [Object::Sequence(items)] => {
            let mut obj = &Object::Empty;
            for item in items {
                if obj == &Object::Empty {
                    obj = item
                } else {
                    match comparison::gr((&env, item), (&env, obj)) {
                        Ok(v) => {
                            if v {
                                obj = item
                            }
                        },
                        Err(e) => return Err(e)
                    }
                }
            }
            Ok((env, obj.clone()))
        }
        _ => panic!("error {:?}", arguments)
    }
}

pub(crate) fn fn_min(env: Box<Environment>, mut arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    let arg = arguments.remove(0);
    match arg {
        Object::Empty => {
            Ok((env, Object::Empty))
        },
        Object::Range{..} => {
            Ok((env, arg))
        },
        Object::Sequence(items) => {
            let mut obj = Object::Empty;
            for item in items {
                if obj == Object::Empty {
                    obj = item
                } else {
                    match comparison::ls((&env, &item), (&env, &obj)) {
                        Ok(v) => if v { obj = item },
                        Err(e) => return Err(e)
                    }
                }
            }
            Ok((env, obj))
        }
        _ => panic!("error {:?}", arguments)
    }
}