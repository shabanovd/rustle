use chrono::Datelike;
use crate::eval::{Environment, Object, DynamicContext, EvalResult};
use crate::values::{Date, DayTimeDuration, Duration, Integer, Time, YearMonthDuration};

pub(crate) fn fn_day_from_date(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Empty] => {
            Ok((env, Object::Empty))
        }
        [Object::Atomic(Date(date))] => {
            Ok((env, Object::Atomic(Integer::boxed(date.day() as i128))))
        },
        _ => panic!("error")
    }
}

pub(crate) fn fn_year_from_date(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Empty] => {
            Ok((env, Object::Empty))
        }
        [Object::Atomic(Date(date))] => {
            Ok((env, Object::Atomic(Integer::boxed(date.year() as i128))))
        },
        _ => panic!("error")
    }
}

pub(crate) fn fn_month_from_date(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Empty] => {
            Ok((env, Object::Empty))
        }
        [Object::Atomic(Date(date))] => {
            Ok((env, Object::Atomic(Integer::boxed(date.month() as i128))))
        },
        _ => panic!("error")
    }
}

pub(crate) fn fn_days_from_duration(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Empty] => {
            Ok((env, Object::Empty))
        }
        [Object::Atomic(Duration { positive, days, .. })] => {
            let sign = if *positive { 1 } else { -1 };
            Ok((env, Object::Atomic(Integer::boxed(*days as i128 * sign))))
        },
        [Object::Atomic(YearMonthDuration { .. })] => {
            Ok((env, Object::Atomic(Integer::boxed(0))))
        },
        [Object::Atomic(DayTimeDuration { positive, days, .. })] => {
            let sign = if *positive { 1 } else { -1 };
            Ok((env, Object::Atomic(Integer::boxed(*days as i128 * sign))))
        },
        _ => panic!("error")
    }
}

pub(crate) fn fn_current_date(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    // TODO  deterministic
    Ok((env, Object::Atomic(Date::now())))
}

pub(crate) fn fn_current_time(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    // TODO  deterministic
    Ok((env, Object::Atomic(Time::now())))
}

pub(crate) fn fn_timezone_from_time(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Empty] => Ok((env, Object::Empty)),
        [Object::Atomic(Time(time))] => {
            let seconds = time.offset.local_minus_utc();

            let (seconds, positive) = if seconds < 0 {
                (-seconds as u32, false)
            } else {
                (seconds as u32, true)
            };

            let (seconds, minutes) = norm(seconds, 60);
            let (minutes, hours) = norm(minutes, 60);
            let (hours, days) = norm(hours, 24);

            Ok((env, Object::atomic(DayTimeDuration { positive, days, hours, minutes, seconds, microseconds: 0 })))
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