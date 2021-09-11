use crate::eval::{Environment, Object, Type};

pub fn fn_days_from_duration<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> (Box<Environment<'a>>, Object) {
    match arguments.as_slice() {
        [Object::Empty] => {
            (env, Object::Empty)
        }
        [Object::Atomic(Type::Duration { positive, days, .. })] => {
            let sign = if *positive { 1 } else { -1 };
            (env, Object::Atomic(Type::Integer(*days as i128 * sign)))
        },
        [Object::Atomic(Type::YearMonthDuration { .. })] => {
            (env, Object::Atomic(Type::Integer(0)))
        },
        [Object::Atomic(Type::DayTimeDuration { positive, days, .. })] => {
            let sign = if *positive { 1 } else { -1 };
            (env, Object::Atomic(Type::Integer(*days as i128 * sign)))
        },
        _ => panic!("error")
    }
}
