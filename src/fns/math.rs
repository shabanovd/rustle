use crate::eval::{Object, Type, EvalResult, DynamicContext};
use crate::eval::Environment;
use math::round::half_to_even;
use bigdecimal::num_traits::float::FloatCore;
use bigdecimal::{BigDecimal, Signed};
use bigdecimal::num_bigint::BigInt;

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

pub(crate) fn fn_floor(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Atomic(Type::Integer(number))] => {
            Ok((env, Object::Atomic(Type::Integer(*number))))
        },
        [Object::Atomic(Type::Decimal(number))] => {
            Ok((env, Object::Atomic(Type::Decimal(number.round(0))))) // TODO: fix it
        },
        [Object::Atomic(Type::Float(number))] => {
            Ok((env, Object::Atomic(Type::Float(number.floor().into()))))
        },
        [Object::Atomic(Type::Double(number))] => {
            Ok((env, Object::Atomic(Type::Double(number.floor().into()))))
        },
        _ => panic!("error")
    }
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