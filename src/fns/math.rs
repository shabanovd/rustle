use crate::eval::{Object, EvalResult, DynamicContext};
use crate::eval::Environment;
use math::round::half_to_even;
use bigdecimal::num_traits::float::FloatCore;
use bigdecimal::{BigDecimal, Signed};
use bigdecimal::num_bigint::BigInt;
use crate::values::{Decimal, Double, Float, Integer};

pub(crate) fn fn_abs(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {

    match arguments.as_slice() {
        [Object::Atomic(Integer(number))] => {
            Ok((env, Object::Atomic(Integer::boxed(number.abs()))))
        },
        [Object::Atomic(Decimal(number))] => {
            Ok((env, Object::Atomic(Decimal::boxed(number.abs()))))
        },
        [Object::Atomic(Float(number))] => {
            Ok((env, Object::Atomic(Float::boxed(number.abs().into()))))
        },
        [Object::Atomic(Double(number))] => {
            Ok((env, Object::Atomic(Double::boxed(number.abs().into()))))
        },
        _ => panic!("error")
    }
}

pub(crate) fn fn_floor(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Atomic(Integer(number))] => {
            Ok((env, Object::Atomic(Integer::boxed(*number))))
        },
        [Object::Atomic(Decimal(number))] => {
            Ok((env, Object::Atomic(Decimal::boxed(number.round(0))))) // TODO: fix it
        },
        [Object::Atomic(Float(number))] => {
            Ok((env, Object::Atomic(Float::boxed(number.floor().into()))))
        },
        [Object::Atomic(Double(number))] => {
            Ok((env, Object::Atomic(Double::boxed(number.floor().into()))))
        },
        _ => panic!("error")
    }
}

pub(crate) fn fn_round(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Atomic(Integer(number))] => {
            Ok((env, Object::Atomic(Integer::boxed(*number))))
        },
        [Object::Atomic(Decimal(number))] => {
            Ok((env, Object::Atomic(Decimal::boxed(number.round(0)))))
        },
        [Object::Atomic(Float(number))] => {
            Ok((env, Object::Atomic(Float::boxed(number.round().into()))))
        },
        [Object::Atomic(Double(number))] => {
            Ok((env, Object::Atomic(Double::boxed(number.round().into()))))
        },
        [Object::Atomic(Integer(number)), Object::Atomic(Integer(precision))] => {
            // TODO check precision range
            let factor = (10 as i128).pow(precision.abs() as u32);
            let number = (*number / factor) * factor;
            Ok((env, Object::Atomic(Integer::boxed(number))))
        },
        [Object::Atomic(Decimal(number)), Object::Atomic(Integer(precision))] => {
            // TODO check precision range
            Ok((env, Object::Atomic(Decimal::boxed(number.round(*precision as i64)))))
        },
        [Object::Atomic(Float(number)), Object::Atomic(Integer(precision))] => {
            // TODO do proper round with precision
            Ok((env, Object::Atomic(Float::boxed(number.round().into()))))
        },
        [Object::Atomic(Double(number)), Object::Atomic(Integer(precision))] => {
            // TODO check precision range
            Ok((env, Object::Atomic(Double::boxed(number.round().into()))))
        },
        _ => panic!("error")
    }
}

pub(crate) fn fn_round_half_to_even(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {

    println!("arguments: {:?}", arguments);
    // TODO precision parameter
    match arguments.as_slice() {
        [Object::Atomic(Integer(number))] => {
            Ok((env, Object::Atomic(Integer::boxed(*number))))
        },
        [Object::Atomic(Decimal(number))] => {
            Ok((env, Object::Atomic(Decimal::boxed(number.round(0)))))
        },
        [Object::Atomic(Float(number))] => {
            Ok((env, Object::Atomic(Float::boxed(number.round().into()))))
        },
        [Object::Atomic(Double(number))] => {
            Ok((env, Object::Atomic(Double::boxed(number.round().into()))))
        },
        [Object::Atomic(Integer(number)), Object::Atomic(Integer(..))] => {
            Ok((env, Object::Atomic(Integer::boxed(*number))))
        },
        [Object::Atomic(Decimal(number)), Object::Atomic(Integer(precision))] => {
            Ok((env, Object::Atomic(Decimal::boxed(round(number,*precision as i64)))))
        },
        [Object::Atomic(Float(number)), Object::Atomic(Integer(precision))] => {
            Ok((env, Object::Atomic(Float::boxed(
                (half_to_even(number.into_inner() as f64, *precision as i8) as f32).into()
            ))))
        },
        [Object::Atomic(Double(number)), Object::Atomic(Integer(precision))] => {
            Ok((env, Object::Atomic(Double::boxed(
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