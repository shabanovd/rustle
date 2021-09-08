use crate::eval::{Object, Type, NumberCase};
use crate::eval::Environment;
use rust_decimal::Decimal;

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

        _ => (env, Object::Empty), // TODO: raise error?
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

        _ => (env, Object::Empty), // TODO: raise error?
    }
}

pub fn fn_abs<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {

    match arguments.as_slice() {
        [Object::Atomic(Type::Integer(num))] => {
            (env, Object::Atomic(Type::Integer((*num).abs())))
        },
        _ => panic!("error")
    }
}