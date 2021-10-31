use crate::eval::{Environment, Object, Type, Time, DynamicContext, EvalResult};
use chrono::{Datelike, Date, Local, TimeZone};

pub(crate) fn fn_day_from_date(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Empty] => {
            Ok((env, Object::Empty))
        }
        [Object::Atomic(Type::Date(date))] => {
            Ok((env, Object::Atomic(Type::Integer(date.day() as i128))))
        },
        _ => panic!("error")
    }
}

pub(crate) fn fn_year_from_date(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Empty] => {
            Ok((env, Object::Empty))
        }
        [Object::Atomic(Type::Date(date))] => {
            Ok((env, Object::Atomic(Type::Integer(date.year() as i128))))
        },
        _ => panic!("error")
    }
}

pub(crate) fn fn_month_from_date(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Empty] => {
            Ok((env, Object::Empty))
        }
        [Object::Atomic(Type::Date(date))] => {
            Ok((env, Object::Atomic(Type::Integer(date.month() as i128))))
        },
        _ => panic!("error")
    }
}

pub(crate) fn fn_days_from_duration(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
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

pub(crate) fn fn_current_date(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    // TODO  deterministic
    let now = Local::now();
    let date = Date::from_utc(now.date().naive_utc(), TimeZone::from_offset(now.offset()));

    Ok((env, Object::Atomic(Type::Date(date))))
}

pub(crate) fn fn_current_time(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    // TODO  deterministic
    Ok((env, Object::Atomic(Type::Time(Time::now()))))
}

pub(crate) fn fn_timezone_from_time(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Empty] => Ok((env, Object::Empty)),
        [Object::Atomic(Type::Time(time))] => {
            let seconds = time.offset.local_minus_utc();

            let (seconds, positive) = if seconds < 0 {
                (-seconds as u32, false)
            } else {
                (seconds as u32, true)
            };

            let (seconds, minutes) = norm(seconds, 60);
            let (minutes, hours) = norm(minutes, 60);
            let (hours, days) = norm(hours, 24);

            Ok((env, Object::Atomic(Type::DayTimeDuration { positive, days, hours, minutes, seconds, microseconds: 0 })))
        },
        _ => panic!("error")
    }
}

pub(crate) fn norm(value: u32, max: u32) -> (u32, u32) {
    let mut v = value;
    let mut count = 0;
    while v > max {
        v = v - max;
        count += 1;
    }
    (v, count)
}