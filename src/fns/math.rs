use crate::eval::{Environment, Object, Type, EvalResult, DynamicContext};
use crate::eval::sequence_type::*;
use crate::fns::FUNCTION;

use crate::parser::errors::ErrorCode;
use crate::values::Types;
use math::round::half_to_even;
use bigdecimal::num_traits::float::FloatCore;
use bigdecimal::{BigDecimal, FromPrimitive, Signed, ToPrimitive};
use bigdecimal::num_bigint::BigInt;
use ordered_float::OrderedFloat;

// fn:abs($arg as xs:numeric?) as xs:numeric?
pub(crate) fn FN_MATH_ABS() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_NUMERIC.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_NUMERIC.into()))
        ),
        fn_abs
    )
}

pub(crate) fn fn_abs(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {

    match arguments.as_slice() {
        [Object::Atomic(Type::Integer(number))] => {
            Ok((env, Object::Atomic(Type::Integer(number.abs()))))
        },
        [Object::Atomic(Type::Decimal(number))] => {
            Ok((env, Object::Atomic(Type::Decimal(number.abs()))))
        },
        [Object::Atomic(Type::Float(number))] => {
            Ok((env, Object::Atomic(Type::Float(number.abs().into()))))
        },
        [Object::Atomic(Type::Double(number))] => {
            Ok((env, Object::Atomic(Type::Double(number.abs().into()))))
        },
        _ => panic!("error")
    }
}

// fn:ceiling($arg as xs:numeric?) as xs:numeric?
pub(crate) fn FN_MATH_CEILING() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_NUMERIC.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_NUMERIC.into()))
        ),
        fn_ceiling
    )
}

pub(crate) fn fn_ceiling(env: Box<Environment>, mut arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    let number = arguments.remove(0);
    let t = match number {
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

                Type::Integer(_) => t,

                Type::Decimal(number) => {
                    if let Some(number) = number.to_f64() {
                        if let Some(number) = BigDecimal::from_f64(number.ceil()) {
                            Type::Decimal(number.normalized())
                        } else {
                            todo!("raise error")
                        }
                    } else {
                        todo!("raise error")
                    }
                },
                Type::Float(number) => Type::Float(number.ceil()),
                Type::Double(number) => Type::Double(number.ceil()),

                _ => panic!("raise error")
            }
        }
        _ => panic!("raise error")
    };
    Ok((env, Object::Atomic(t)))
}

// fn:floor($arg as xs:numeric?) as xs:numeric?
pub(crate) fn FN_MATH_FLOOR() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_NUMERIC.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_NUMERIC.into()))
        ),
        fn_floor
    )
}

pub(crate) fn fn_floor(env: Box<Environment>, mut arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    let number = arguments.remove(0);
    let t = match number {
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

                Type::Integer(_) => t,

                Type::Decimal(number) => Type::Decimal(number.round(0)), // TODO: fix it
                Type::Float(number) => Type::Float(number.floor()),
                Type::Double(number) => Type::Double(number.floor()),

                _ => panic!("raise error")
            }
        }
        _ => panic!("raise error")
    };
    Ok((env, Object::Atomic(t)))
}

// fn:round($arg as xs:numeric?) as xs:numeric?
pub(crate) fn FN_MATH_ROUND_1() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_NUMERIC.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_NUMERIC.into()))
        ),
        fn_round
    )
}

// fn:round($arg as xs:numeric?, $precision as xs:integer) as xs:numeric?
pub(crate) fn FN_MATH_ROUND_2() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_NUMERIC.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_INTEGER.into()))
            ].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_NUMERIC.into()))
        ),
        fn_round
    )
}

pub(crate) fn fn_round(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Atomic(Type::Integer(number))] => {
            Ok((env, Object::Atomic(Type::Integer(*number))))
        },
        [Object::Atomic(Type::Decimal(number))] => {
            Ok((env, Object::Atomic(Type::Decimal(number.round(0)))))
        },
        [Object::Atomic(Type::Float(number))] => {
            Ok((env, Object::Atomic(Type::Float(number.round().into()))))
        },
        [Object::Atomic(Type::Double(number))] => {
            Ok((env, Object::Atomic(Type::Double(number.round().into()))))
        },
        [Object::Atomic(Type::Integer(number)), Object::Atomic(Type::Integer(precision))] => {
            // TODO check precision range
            let factor = (10 as i128).pow(precision.abs() as u32);
            let number = (*number / factor) * factor;
            Ok((env, Object::Atomic(Type::Integer(number))))
        },
        [Object::Atomic(Type::Decimal(number)), Object::Atomic(Type::Integer(precision))] => {
            // TODO check precision range
            Ok((env, Object::Atomic(Type::Decimal(number.round(*precision as i64)))))
        },
        [Object::Atomic(Type::Float(number)), Object::Atomic(Type::Integer(precision))] => {
            // TODO do proper round with precision
            Ok((env, Object::Atomic(Type::Float(number.round().into()))))
        },
        [Object::Atomic(Type::Double(number)), Object::Atomic(Type::Integer(precision))] => {
            // TODO check precision range
            Ok((env, Object::Atomic(Type::Double(number.round().into()))))
        },
        _ => panic!("error")
    }
}

// fn:round-half-to-even($arg as xs:numeric?) as xs:numeric?
pub(crate) fn FN_MATH_ROUND_HALF_TO_EVEN_1() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_NUMERIC.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_NUMERIC.into()))
        ),
        fn_round_half_to_even
    )
}

// fn:round-half-to-even($arg as xs:numeric?, $precision as xs:integer) as xs:numeric?
pub(crate) fn FN_MATH_ROUND_HALF_TO_EVEN_2() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_NUMERIC.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_INTEGER.into()))
            ].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_NUMERIC.into()))
        ),
        fn_round_half_to_even
    )
}

pub(crate) fn fn_round_half_to_even(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {

    println!("arguments: {:?}", arguments);
    // TODO precision parameter
    match arguments.as_slice() {
        [Object::Atomic(Type::Integer(number))] => {
            Ok((env, Object::Atomic(Type::Integer(*number))))
        },
        [Object::Atomic(Type::Decimal(number))] => {
            Ok((env, Object::Atomic(Type::Decimal(number.round(0)))))
        },
        [Object::Atomic(Type::Float(number))] => {
            Ok((env, Object::Atomic(Type::Float(number.round().into()))))
        },
        [Object::Atomic(Type::Double(number))] => {
            Ok((env, Object::Atomic(Type::Double(number.round().into()))))
        },
        [Object::Atomic(Type::Integer(number)), Object::Atomic(Type::Integer(..))] => {
            Ok((env, Object::Atomic(Type::Integer(*number))))
        },
        [Object::Atomic(Type::Decimal(number)), Object::Atomic(Type::Integer(precision))] => {
            Ok((env, Object::Atomic(Type::Decimal(round(number,*precision as i64)))))
        },
        [Object::Atomic(Type::Float(number)), Object::Atomic(Type::Integer(precision))] => {
            Ok((env, Object::Atomic(Type::Float(
                (half_to_even(number.into_inner() as f64, *precision as i8) as f32).into()
            ))))
        },
        [Object::Atomic(Type::Double(number)), Object::Atomic(Type::Integer(precision))] => {
            Ok((env, Object::Atomic(Type::Double(
                half_to_even(number.into_inner() as f64, *precision as i8).into()
            ))))
        },
        _ => panic!("error")
    }
}

pub fn round(this: &BigDecimal, round_digits: i64) -> BigDecimal {
    let (bigint, decimal_part_digits) = this.as_bigint_and_exponent();
    let need_to_round_digits = decimal_part_digits - round_digits;
    if round_digits >= 0 && need_to_round_digits <= 0 {
        return this.clone();
    }

    let mut number = bigint.clone(); //.to_i128().unwrap();
    if number < BigInt::from(0) {
        number = -number;
    }
    for _ in 0..(need_to_round_digits - 1) {
        number /= 10;
    }
    let digit = number % 10;

    if digit <= BigInt::from(4) {
        this.with_scale(round_digits)
    } else if bigint.is_negative() {
        this.with_scale(round_digits) - BigDecimal::new(BigInt::from(1), round_digits)
    } else {
        this.with_scale(round_digits) + BigDecimal::new(BigInt::from(1), round_digits)
    }
}

// fn:number() as xs:double
pub(crate) fn FN_MATH_NUMBER_0() -> FUNCTION {
    (
        (
            [].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DOUBLE.into()))
        ),
        fn_number
    )
}

// fn:number($arg as xs:anyAtomicType?) as xs:double
pub(crate) fn FN_MATH_NUMBER_1() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DOUBLE.into()))
        ),
        fn_number
    )
}

pub(crate) fn fn_number(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    let item = if arguments.len() == 0 {
        &context.item
    } else {
        arguments.get(0).unwrap()
    };

    match item {
        Object::Nothing => Err((ErrorCode::XPDY0002, String::from("TODO"))),
        Object::Empty => Ok((env, Object::Empty)),
        Object::Atomic(t) => {
            match t.convert(Types::Double) {
                Ok(v) => Ok((env, Object::Atomic(v))),
                Err(_) => Ok((env, Object::Atomic(Type::Double(OrderedFloat::from(f64::NAN))))),
            }
        }
        Object::Node(rf) => {
            match rf.to_typed_value() {
                Ok(str) => {
                    let value = crate::values::string_to::double(&str, false)?;
                    Ok((env, Object::Atomic(value)))
                },
                Err(msg) => Err((ErrorCode::FORG0001, msg))
            }
        }
        _ => Ok((env, Object::Atomic(Type::Double(OrderedFloat::from(f64::NAN)))))
    }
}

// fn:format-integer($value as xs:integer?, $picture as xs:string) as xs:string
pub(crate) fn FN_MATH_FORMAT_INTEGER_2() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_INTEGER.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
            ].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
        ),
        fn_format_integer
    )
}

// fn:format-integer($value as xs:integer?, $picture as xs:string, $lang as xs:string?) as xs:string
pub(crate) fn FN_MATH_FORMAT_INTEGER_3() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_INTEGER.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
            ].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
        ),
        fn_format_integer
    )
}

pub(crate) fn fn_format_integer(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// fn:format-number($value as xs:numeric?, $picture as xs:string) as xs:string
pub(crate) fn FN_MATH_FORMAT_NUMBER_2() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_NUMERIC.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
            ].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
        ),
        fn_format_number
    )
}

// fn:format-number($value as xs:numeric?, $picture as xs:string, $decimal-format-name as xs:string?) as xs:string
pub(crate) fn FN_MATH_FORMAT_NUMBER_3() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_NUMERIC.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
            ].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
        ),
        fn_format_number
    )
}

pub(crate) fn fn_format_number(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// math:pi() as xs:double
pub(crate) fn FN_MATH_PI() -> FUNCTION {
    (
        (
            [].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DOUBLE.into()))
        ),
        fn_pi
    )
}

pub(crate) fn fn_pi(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    Ok((env, Object::Atomic(Type::Double(OrderedFloat::from(std::f64::consts::PI)))))
}

// math:exp($arg as xs:double?) as xs:double?
pub(crate) fn FN_MATH_EXP() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DOUBLE.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DOUBLE.into()))
        ),
        fn_exp
    )
}

pub(crate) fn fn_exp(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    todo!()
}

// math:exp10($arg as xs:double?) as xs:double?
pub(crate) fn FN_MATH_EXP10() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DOUBLE.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DOUBLE.into()))
        ),
        fn_exp10
    )
}

pub(crate) fn fn_exp10(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    todo!()
}

// math:log($arg as xs:double?) as xs:double?
pub(crate) fn FN_MATH_LOG() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DOUBLE.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DOUBLE.into()))
        ),
        fn_log
    )
}

pub(crate) fn fn_log(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    todo!()
}

// math:log10($arg as xs:double?) as xs:double?
pub(crate) fn FN_MATH_LOG10() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DOUBLE.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DOUBLE.into()))
        ),
        fn_log10
    )
}

pub(crate) fn fn_log10(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    todo!()
}

// math:pow($x as xs:double?, $y as xs:numeric) as xs:double?
pub(crate) fn FN_MATH_POW() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DOUBLE.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_NUMERIC.into())),
            ].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DOUBLE.into()))
        ),
        fn_pow
    )
}

pub(crate) fn fn_pow(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    todo!()
}

// math:sqrt($arg as xs:double?) as xs:double?
pub(crate) fn FN_MATH_SQRT() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DOUBLE.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DOUBLE.into()))
        ),
        fn_sqrt
    )
}

pub(crate) fn fn_sqrt(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    todo!()
}

// math:sin($θ as xs:double?) as xs:double?
pub(crate) fn FN_MATH_SIN() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DOUBLE.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DOUBLE.into()))
        ),
        fn_sin
    )
}

pub(crate) fn fn_sin(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    todo!()
}

// math:cos($θ as xs:double?) as xs:double?
pub(crate) fn FN_MATH_COS() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DOUBLE.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DOUBLE.into()))
        ),
        fn_cos
    )
}

pub(crate) fn fn_cos(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    todo!()
}

// math:tan($θ as xs:double?) as xs:double?
pub(crate) fn FN_MATH_TAN() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DOUBLE.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DOUBLE.into()))
        ),
        fn_tan
    )
}

pub(crate) fn fn_tan(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    todo!()
}

// math:asin($arg as xs:double?) as xs:double?
pub(crate) fn FN_MATH_ASIN() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DOUBLE.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DOUBLE.into()))
        ),
        fn_asin
    )
}

pub(crate) fn fn_asin(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    todo!()
}

// math:acos($arg as xs:double?) as xs:double?
pub(crate) fn FN_MATH_ACOS() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DOUBLE.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DOUBLE.into()))
        ),
        fn_acos
    )
}

pub(crate) fn fn_acos(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    todo!()
}

// math:atan($arg as xs:double?) as xs:double?
pub(crate) fn FN_MATH_ATAN() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DOUBLE.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DOUBLE.into()))
        ),
        fn_atan
    )
}

pub(crate) fn fn_atan(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    todo!()
}

// math:atan2($y as xs:double, $x as xs:double) as xs:double
pub(crate) fn FN_MATH_ATAN2() -> FUNCTION {
    (
        (
            [
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DOUBLE.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DOUBLE.into())),
            ].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DOUBLE.into()))
        ),
        fn_atan2
    )
}

pub(crate) fn fn_atan2(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    todo!()
}