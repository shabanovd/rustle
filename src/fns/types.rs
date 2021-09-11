use crate::eval::{Type, NumberCase};
use crate::eval::Object;
use crate::eval::Environment;
use rust_decimal::Decimal;
use crate::parser::parse_duration::{parse_day_time_duration, string_to_dt_duration, string_to_ym_duration, string_to_duration};

pub fn xs_untyped_atomic_eval<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {
    match arguments.as_slice() {

        [Object::Atomic(Type::String(string))] => {
            (env, Object::Atomic(Type::Untyped(string.clone())))
        },
        _ => todo!()
    }
}

pub fn xs_ncname_eval<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {
    match arguments.as_slice() {

        [Object::Atomic(Type::String(string))] =>
            (env, Object::Atomic(Type::NCName(string.clone()))),

        _ => todo!()
    }
}

pub fn xs_anyuri_eval<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {
    match arguments.as_slice() {

        [Object::Atomic(Type::String(string))] =>
            (env, Object::Atomic(Type::AnyURI(string.clone()))),

        _ => todo!()
    }
}

pub fn xs_duration_eval<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {
    match arguments.as_slice() {
        [Object::Atomic(Type::String(string))] => {
            match string_to_duration(string) {
                Ok(dt) => (env, Object::Atomic(dt)),
                Err(e) => {
                    println!("{}", e);
                    todo!()
                }
            }
        },
        _ => todo!()
    }
}

pub fn xs_day_time_duration_eval<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {
    match arguments.as_slice() {
        [Object::Atomic(Type::String(string))] => {
            match string_to_dt_duration(string) {
                Ok(dt) => (env, Object::Atomic(dt)),
                Err(e) => {
                    println!("{}", e);
                    todo!()
                }
            }
        },
        _ => todo!()
    }
}

pub fn xs_year_month_duration_eval<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {
    match arguments.as_slice() {
        [Object::Atomic(Type::String(string))] => {
            match string_to_ym_duration(string) {
                Ok(dt) => (env, Object::Atomic(dt)),
                Err(e) => todo!()
            }
        },
        _ => todo!()
    }
}

pub fn xs_decimal_eval<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {
    match arguments.as_slice() {

        [Object::Atomic(Type::String(string))] => (env, Object::Atomic(Type::Integer(string.parse::<i128>().unwrap()))),

        [Object::Atomic(Type::Integer(integer))] => (env, Object::Atomic(Type::Integer(*integer))),

        _ => todo!()
    }
}

pub fn xs_float_eval<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {
    match arguments.as_slice() {

        [Object::Atomic(Type::String(string))] => {
            let t = match string.as_str() {
                "INF" => Type::Float { number: None, case: NumberCase::PlusInfinity },
                "-INF" => Type::Float { number: None, case: NumberCase::MinusInfinity },
                "NaN" => Type::Float { number: None, case: NumberCase::NaN },
                _ => {
                    if let Ok(num) = Decimal::from_scientific(string) {
                        Type::Float { number: Some(num), case: NumberCase::Normal }
                    } else {
                        panic!("error")
                    }
                },
            };

            (env, Object::Atomic(t))
        }

        [Object::Atomic(Type::Float { number, case })] => {
            (env, Object::Atomic(Type::Float { number: *number, case: case.clone() }))
        },

        _ => todo!()
    }
}

pub fn xs_double_eval<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {
    match arguments.as_slice() {

        [Object::Atomic(Type::String(string))] => {
            let t = match string.as_str() {
                "INF" => Type::Double { number: None, case: NumberCase::PlusInfinity },
                "-INF" => Type::Double { number: None, case: NumberCase::MinusInfinity },
                "NaN" => Type::Double { number: None, case: NumberCase::NaN },
                _ => {
                    if let Ok(num) = Decimal::from_scientific(string) {
                        Type::Double { number: Some(num), case: NumberCase::Normal }
                    } else {
                        panic!("error")
                    }
                },
            };

            (env, Object::Atomic(t))
        }

        [Object::Atomic(Type::Double { number, case })] => {
            (env, Object::Atomic(Type::Double { number: *number, case: case.clone() }))
        },

        _ => todo!()
    }
}