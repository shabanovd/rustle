use crate::eval::{Object, Type, EvalResult};
use crate::eval::Environment;
use math::round::half_to_even;
use bigdecimal::num_traits::float::FloatCore;

pub fn fn_abs<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context_item: &Object) -> EvalResult<'a> {

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

pub fn fn_floor<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context_item: &Object) -> EvalResult<'a> {
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

pub fn fn_round<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context_item: &Object) -> EvalResult<'a> {
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
        _ => panic!("error")
    }
}

pub fn fn_round_half_to_even<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context_item: &Object) -> EvalResult<'a> {

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
            println!("{:?} {:?}", precision, number.round(*precision as i64));
            Ok((env, Object::Atomic(Type::Decimal(number.round(*precision as i64)))))
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