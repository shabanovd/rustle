use crate::eval::{Environment, Object, Type, EvalResult, DynamicContext, comparison, ErrorInfo};
use crate::eval::sequence_type::*;
use crate::fns::FUNCTION;

use bigdecimal::{BigDecimal, FromPrimitive};
use crate::parser::errors::ErrorCode;
use crate::values::Types;

// fn:count($arg as item()*) as xs:integer
pub(crate) fn FN_COUNT() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_more(ItemType::Item)].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_INTEGER.into()))
        ),
        fn_count
    )
}

pub(crate) fn fn_count(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Empty] => {
            Ok((env, Object::Atomic(Type::Integer(0))))
        },
        [Object::Atomic(..)] => {
            Ok((env, Object::Atomic(Type::Integer(1))))
        }
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
            Ok((env, Object::Atomic(Type::Integer(count))))
        },
        [Object::Node(..)] => {
            Ok((env, Object::Atomic(Type::Integer(1))))
        }
        _ => panic!("error {:?}", arguments)
    }
}

// fn:avg($arg as xs:anyAtomicType*) as xs:anyAtomicType?
pub(crate) fn FN_AVG() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_more(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AnyAtomicType)
        ),
        fn_avg
    )
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

// fn:max($arg as xs:anyAtomicType*) as xs:anyAtomicType?
pub(crate) fn FN_MAX_1() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_more(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AnyAtomicType)
        ),
        fn_max
    )
}

// fn:max($arg as xs:anyAtomicType*, $collation as xs:string) as xs:anyAtomicType?
pub(crate) fn FN_MAX_2() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_more(ItemType::AnyAtomicType),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
            ].to_vec(),
            SequenceType::zero_or_one(ItemType::AnyAtomicType)
        ),
        fn_max
    )
}

pub(crate) fn fn_max(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Empty] => {
            Ok((env, Object::Empty))
        },
        [Object::Range { min, max}] => {
            Ok((env, Object::Atomic(Type::Integer(*max))))
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

// fn:min($arg as xs:anyAtomicType*) as xs:anyAtomicType?
pub(crate) fn FN_MIN_1() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_more(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AnyAtomicType)
        ),
        fn_min
    )
}

// fn:min($arg as xs:anyAtomicType*, $collation as xs:string) as xs:anyAtomicType?
pub(crate) fn FN_MIN_2() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_more(ItemType::AnyAtomicType),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
            ].to_vec(),
            SequenceType::zero_or_one(ItemType::AnyAtomicType)
        ),
        fn_min
    )
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

// fn:sum($arg as xs:anyAtomicType*) as xs:anyAtomicType
pub(crate) fn FN_SUM_1() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_more(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AnyAtomicType)
        ),
        fn_sum
    )
}

// fn:sum($arg as xs:anyAtomicType*, $zero as xs:anyAtomicType?) as xs:anyAtomicType?
pub(crate) fn FN_SUM_2() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_more(ItemType::AnyAtomicType),
                SequenceType::zero_or_one(ItemType::AnyAtomicType)
            ].to_vec(),
            SequenceType::zero_or_one(ItemType::AnyAtomicType)
        ),
        fn_sum
    )
}

fn extract_number_or_duration(obj: Object) -> Result<Type, ErrorInfo> {
    match obj {
        Object::Atomic(t) => {
            match t {
                Type::UnsignedByte(_) |
                Type::UnsignedShort(_) |
                Type::UnsignedInt(_) |
                Type::UnsignedLong(_) |

                Type::Byte(_) |
                Type::Short(_) |
                Type::Int(_) |
                Type::Long(_) |

                Type::PositiveInteger(_) |
                Type::NonNegativeInteger(_) |
                Type::NonPositiveInteger(_) |
                Type::NegativeInteger(_) |

                Type::Integer(_) |
                Type::Decimal(_) |
                Type::Float(_) |
                Type::Double(_) |

                Type::YearMonthDuration  { .. } |
                Type::DayTimeDuration { .. }  => Ok(t),

                _ => Err(ErrorCode::forg0006(format!("{:?}", t)))
            }
        }
        _ => Err(ErrorCode::forg0006(obj.to_short_string()))
    }
}

pub(crate) fn fn_sum(env: Box<Environment>, mut arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    let arg = arguments.remove(0);
    // let on_empty = if arguments.len() == 1 {
    //     extract_number_or_duration(arguments.remove(0))?
    // } else {
    //     Type::Integer(0)
    // };

    let mut it = arg.into_iter();
    let result = if let Some(obj) = it.next() {

        let mut sum = extract_number_or_duration(obj)?;
        loop {
            if let Some(operand) = it.next() {
                sum = match sum.to_type() {
                    Types::YearMonthDuration => todo!(),
                    Types::DayTimeDuration  => todo!(),

                    Types::UnsignedByte |
                    Types::UnsignedShort |
                    Types::UnsignedInt |
                    Types::UnsignedLong |

                    Types::Byte |
                    Types::Short |
                    Types::Int |
                    Types::Long |

                    Types::PositiveInteger |
                    Types::NonNegativeInteger |
                    Types::NonPositiveInteger |
                    Types::NegativeInteger |

                    Types::Integer |
                    Types::Decimal |
                    Types::Float |
                    Types::Double => {
                        let n1 = crate::eval::arithmetic::type_to_number(sum)?;
                        let n2 = crate::eval::arithmetic::object_to_number(operand)?;

                        match n1.add(&*n2) {
                            Ok(number) => number.to_atomic(),
                            Err(code) => return Err((code, String::from("TODO")))
                        }
                    }
                    _ => return Err(ErrorCode::forg0006(operand.to_short_string()))
                };
            } else {
                break;
            }
        }
        sum
    } else {
        // when empty use "zero" argument
        if arguments.len() == 1 {
            extract_number_or_duration(arguments.remove(0))?
        } else {
            Type::Integer(0)
        }
    };

    Ok((env, Object::Atomic(result)))
}