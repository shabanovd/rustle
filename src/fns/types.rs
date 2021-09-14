use crate::eval::{Type, NumberCase, EvalResult, string_to_number};
use crate::eval::Object;
use crate::eval::Environment;
use rust_decimal::Decimal;
use crate::parser::parse_duration::{parse_day_time_duration, string_to_dt_duration, string_to_ym_duration, string_to_duration, string_to_date};
use crate::eval::string_to_double;

pub fn xs_untyped_atomic_eval<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> EvalResult<'a> {
    match arguments.as_slice() {
        [Object::Atomic(Type::String(string))] => {
            Ok((env, Object::Atomic(Type::Untyped(string.clone()))))
        },
        [Object::Atomic(Type::Integer(num))] => {
            Ok((env, Object::Atomic(Type::Untyped(num.to_string()))))
        },
        _ => todo!()
    }
}

pub fn xs_ncname_eval<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> EvalResult<'a> {
    match arguments.as_slice() {

        [Object::Atomic(Type::String(string))] =>
            Ok((env, Object::Atomic(Type::NCName(string.clone())))),

        _ => todo!()
    }
}

pub fn xs_anyuri_eval<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> EvalResult<'a> {
    match arguments.as_slice() {

        [Object::Atomic(Type::String(string))] =>
            Ok((env, Object::Atomic(Type::AnyURI(string.clone())))),

        _ => todo!()
    }
}

pub fn xs_date_eval<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> EvalResult<'a> {
    match arguments.as_slice() {
        [Object::Atomic(Type::String(string))] => {
            match string_to_date(string) {
                Ok(dt) => Ok((env, Object::Atomic(dt))),
                Err(e) => {
                    println!("{}", e);
                    todo!()
                }
            }
        },
        _ => todo!()
    }
}

pub fn xs_duration_eval<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> EvalResult<'a> {
    match arguments.as_slice() {
        [Object::Atomic(Type::String(string))] => {
            match string_to_duration(string) {
                Ok(dt) => Ok((env, Object::Atomic(dt))),
                Err(e) => {
                    println!("{}", e);
                    todo!()
                }
            }
        },
        _ => todo!()
    }
}

pub fn xs_day_time_duration_eval<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> EvalResult<'a> {
    match arguments.as_slice() {
        [Object::Atomic(Type::String(string))] => {
            match string_to_dt_duration(string) {
                Ok(dt) => Ok((env, Object::Atomic(dt))),
                Err(e) => {
                    println!("{}", e);
                    todo!()
                }
            }
        },
        _ => todo!()
    }
}

pub fn xs_year_month_duration_eval<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> EvalResult<'a> {
    match arguments.as_slice() {
        [Object::Atomic(Type::String(string))] => {
            match string_to_ym_duration(string) {
                Ok(dt) => Ok((env, Object::Atomic(dt))),
                Err(e) => todo!()
            }
        },
        _ => todo!()
    }
}

pub fn xs_integer_eval<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> EvalResult<'a> {
    match arguments.as_slice() {

        [Object::Atomic(Type::String(string))] =>
            Ok((env, Object::Atomic(Type::Integer(string.parse::<i128>().unwrap())))),

        [Object::Atomic(Type::Integer(integer))] =>
            Ok((env, Object::Atomic(Type::Integer(*integer)))),

        _ => todo!()
    }
}

pub fn xs_decimal_eval<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> EvalResult<'a> {
    match arguments.as_slice() {
        [Object::Atomic(Type::String(string))] => {
            match string_to_number(string) {
                Ok((number, case)) => {
                    Ok((env, Object::Atomic(Type::Decimal { number, case })))
                },
                Err(code) => Err((code, String::from("TODO")))
            }
        },
        [Object::Atomic(Type::Integer(number))] => {
            Ok((env, Object::Atomic(Type::Decimal { number: Some(Decimal::from(*number)), case: NumberCase::Normal })))
        },
        [Object::Atomic(Type::Decimal { number, case })] => {
            Ok((env, Object::Atomic(Type::Decimal { number: *number, case: case.clone() })))
        },
        [Object::Atomic(Type::Float { number, case })] => {
            Ok((env, Object::Atomic(Type::Decimal { number: *number, case: case.clone() })))
        },
        [Object::Atomic(Type::Double { number, case })] => {
            Ok((env, Object::Atomic(Type::Decimal { number: *number, case: case.clone() })))
        },

        _ => todo!()
    }
}

pub fn xs_float_eval<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> EvalResult<'a> {
    match arguments.as_slice() {
        [Object::Atomic(Type::String(string))] => {
            match string_to_number(string) {
                Ok((number, case)) => {
                    Ok((env, Object::Atomic(Type::Float { number, case })))
                },
                Err(code) => Err((code, String::from("TODO")))
            }
        }
        [Object::Atomic(Type::Integer(number))] => {
            Ok((env, Object::Atomic(Type::Float { number: Some(Decimal::from(*number)), case: NumberCase::Normal })))
        },
        [Object::Atomic(Type::Decimal { number, case })] => {
            Ok((env, Object::Atomic(Type::Float { number: *number, case: case.clone() })))
        },
        [Object::Atomic(Type::Float { number, case })] => {
            Ok((env, Object::Atomic(Type::Float { number: *number, case: case.clone() })))
        },
        [Object::Atomic(Type::Double { number, case })] => {
            Ok((env, Object::Atomic(Type::Float { number: *number, case: case.clone() })))
        },

        _ => todo!()
    }
}

pub fn xs_double_eval<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> EvalResult<'a> {
    println!("arguments {:?}", arguments);
    match arguments.as_slice() {

        [Object::Atomic(Type::String(string))] => {
            match string_to_number(string) {
                Ok((number, case)) => {
                    Ok((env, Object::Atomic(Type::Double { number, case })))
                },
                Err(code) => Err((code, String::from("TODO")))
            }
        },
        [Object::Atomic(Type::Integer(number))] => {
            Ok((env, Object::Atomic(Type::Double { number: Some(Decimal::from(*number)), case: NumberCase::Normal })))
        },
        [Object::Atomic(Type::Decimal { number, case })] => {
            Ok((env, Object::Atomic(Type::Double { number: *number, case: case.clone() })))
        },
        [Object::Atomic(Type::Float { number, case })] => {
            Ok((env, Object::Atomic(Type::Double { number: *number, case: case.clone() })))
        },
        [Object::Atomic(Type::Double { number, case })] => {
            Ok((env, Object::Atomic(Type::Double { number: *number, case: case.clone() })))
        },
        _ => todo!()
    }
}