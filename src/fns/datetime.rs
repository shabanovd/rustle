use crate::eval::{Environment, Object, Type, EvalResult};

pub fn fn_day_from_date<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> EvalResult<'a> {
    match arguments.as_slice() {
        [Object::Empty] => {
            Ok((env, Object::Empty))
        }
        [Object::Atomic(Type::Date { d, .. })] => {
            Ok((env, Object::Atomic(Type::Integer(*d as i128))))
        },
        _ => panic!("error")
    }
}

pub fn fn_month_from_date<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> EvalResult<'a> {
    match arguments.as_slice() {
        [Object::Empty] => {
            Ok((env, Object::Empty))
        }
        [Object::Atomic(Type::Date { m, .. })] => {
            Ok((env, Object::Atomic(Type::Integer(*m as i128))))
        },
        _ => panic!("error")
    }
}

pub fn fn_days_from_duration<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> EvalResult<'a> {
    match arguments.as_slice() {
        [Object::Empty] => {
            Ok((env, Object::Empty))
        }
        [Object::Atomic(Type::Duration { positive, days, .. })] => {
            let sign = if *positive { 1 } else { -1 };
            Ok((env, Object::Atomic(Type::Integer(*days as i128 * sign))))
        },
        [Object::Atomic(Type::YearMonthDuration { .. })] => {
            Ok((env, Object::Atomic(Type::Integer(0))))
        },
        [Object::Atomic(Type::DayTimeDuration { positive, days, .. })] => {
            let sign = if *positive { 1 } else { -1 };
            Ok((env, Object::Atomic(Type::Integer(*days as i128 * sign))))
        },
        _ => panic!("error")
    }
}
