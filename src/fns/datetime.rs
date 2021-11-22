use chrono::Datelike;
use crate::eval::{Environment, Object, Type, DynamicContext, EvalResult};
use crate::eval::sequence_type::*;
use crate::fns::FUNCTION;

// // op:yearMonthDuration-less-than($arg1 as xs:yearMonthDuration, $arg2 as xs:yearMonthDuration) as xs:boolean
// pub(crate) static OP_YEAR_MONTH_DURATION_LESS_THAN() -> FUNCTION {
//     (
//     (
//         [
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_YEAR_MONTH_DURATION)),
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_YEAR_MONTH_DURATION))
//         ].to_vec(),
//         SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_BOOLEAN)),
//     ),
//     op_year_month_duration_less_than
// );
//
// pub(crate) fn op_year_month_duration_less_than(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
//     todo!()
// }
//
// // op:yearMonthDuration-greater-than($arg1 as xs:yearMonthDuration, $arg2 as xs:yearMonthDuration) as xs:boolean
// pub(crate) static OP_YEAR_MONTH_DURATION_GREATER_THAN() -> FUNCTION {
//     (
//     (
//         [
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_YEAR_MONTH_DURATION)),
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_YEAR_MONTH_DURATION))
//         ].to_vec(),
//         SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_BOOLEAN)),
//     ),
//     op_year_month_duration_greater_than
// );
//
// pub(crate) fn op_year_month_duration_greater_than(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
//     todo!()
// }
//
// // op:dayTimeDuration-less-than($arg1 as xs:dayTimeDuration, $arg2 as xs:dayTimeDuration) as xs:boolean
// pub(crate) static OP_DAY_TIME_DURATION_LESS_THAN() -> FUNCTION {
//     (
//     (
//         [
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DAY_TIME_DURATION.into())),
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DAY_TIME_DURATION.into()))
//         ].to_vec(),
//         SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_BOOLEAN)),
//     ),
//     op_day_time_duration_less_than
// );
//
// pub(crate) fn op_day_time_duration_less_than(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
//     todo!()
// }
//
// // op:dayTimeDuration-greater-than($arg1 as xs:dayTimeDuration, $arg2 as xs:dayTimeDuration) as xs:boolean
// pub(crate) static OP_DAY_TIME_DURATION_GREATER_THAN() -> FUNCTION {
//     (
//     (
//         [
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DAY_TIME_DURATION.into())),
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DAY_TIME_DURATION.into()))
//         ].to_vec(),
//         SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_BOOLEAN)),
//     ),
//     op_day_time_duration_greater_than
// );
//
// pub(crate) fn op_day_time_duration_greater_than(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
//     todo!()
// }
//
// // op:duration-equal($arg1 as xs:duration, $arg2 as xs:duration) as xs:boolean
// pub(crate) static OP_DURATION_EQUAL() -> FUNCTION {
//     (
//     (
//         [
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DURATION)),
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DURATION))
//         ].to_vec(),
//         SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_BOOLEAN)),
//     ),
//     op_duration_equal
// );
//
// pub(crate) fn op_duration_equal(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
//     todo!()
// }

// fn:years-from-duration($arg as xs:duration?) as xs:integer?
pub(crate) fn FN_YEARS_FROM_DURATION() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DURATION.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_INTEGER.into())),
        ),
        fn_years_from_duration
    )
}

pub(crate) fn fn_years_from_duration(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// fn:months-from-duration($arg as xs:duration?) as xs:integer?
pub(crate) fn FN_MONTHS_FROM_DURATION() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DURATION.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_INTEGER.into())),
        ),
        fn_months_from_duration
    )
}

pub(crate) fn fn_months_from_duration(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// fn:days-from-duration($arg as xs:duration?) as xs:integer?
pub(crate) fn FN_DAYS_FROM_DURATION() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DURATION.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_INTEGER.into())),
        ),
        fn_days_from_duration
    )
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

// fn:hours-from-duration($arg as xs:duration?) as xs:integer?
pub(crate) fn FN_HOURS_FROM_DURATION() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DURATION.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_INTEGER.into())),
        ),
        fn_hours_from_duration
    )
}

pub(crate) fn fn_hours_from_duration(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// fn:minutes-from-duration($arg as xs:duration?) as xs:integer?
pub(crate) fn FN_MINUTES_FROM_DURATION() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DURATION.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_INTEGER.into())),
        ),
        fn_minutes_from_duration
    )
}

pub(crate) fn fn_minutes_from_duration(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// fn:seconds-from-duration($arg as xs:duration?) as xs:decimal?
pub(crate) fn FN_SECONDS_FROM_DURATION() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DURATION.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DECIMAL.into())),
        ),
        fn_seconds_from_duration
    )
}

pub(crate) fn fn_seconds_from_duration(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// // op:add-yearMonthDurations($arg1 as xs:yearMonthDuration, $arg2 as xs:yearMonthDuration) as xs:yearMonthDuration
// pub(crate) static OP_ADD_YEAR_MONTH_DURATIONS() -> FUNCTION {
//     (
//     (
//         [
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_YEAR_MONTH_DURATION)),
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_YEAR_MONTH_DURATION))
//         ].to_vec(),
//         SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_YEAR_MONTH_DURATION)),
//     ),
//     op_add_year_month_durations
// );
//
// pub(crate) fn op_add_year_month_durations(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
//     todo!()
// }
//
// // op:subtract-yearMonthDurations($arg1 as xs:yearMonthDuration, $arg2 as xs:yearMonthDuration) as xs:yearMonthDuration
// pub(crate) static OP_SUBTRACT_YEAR_MONTH_DURATIONS() -> FUNCTION {
//     (
//     (
//         [
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_YEAR_MONTH_DURATION)),
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_YEAR_MONTH_DURATION))
//         ].to_vec(),
//         SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_YEAR_MONTH_DURATION)),
//     ),
//     op_subtract_year_month_durations
// );
//
// pub(crate) fn op_subtract_year_month_durations(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
//     todo!()
// }
//
// // op:multiply-yearMonthDuration($arg1 as xs:yearMonthDuration, $arg2 as xs:double) as xs:yearMonthDuration
// pub(crate) static OP_MULTIPLY_YEAR_MONTH_DURATIONS() -> FUNCTION {
//     (
//     (
//         [
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_YEAR_MONTH_DURATION)),
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_YEAR_MONTH_DURATION))
//         ].to_vec(),
//         SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_YEAR_MONTH_DURATION)),
//     ),
//     op_multiply_year_month_durations
// );
//
// pub(crate) fn op_multiply_year_month_durations(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
//     todo!()
// }
//
// // op:divide-yearMonthDuration($arg1 as xs:yearMonthDuration, $arg2 as xs:double) as xs:yearMonthDuration
// pub(crate) static OP_DIVIDE_YEAR_MONTH_DURATION() -> FUNCTION {
//     (
//     (
//         [
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_YEAR_MONTH_DURATION)),
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DOUBLE))
//         ].to_vec(),
//         SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_YEAR_MONTH_DURATION)),
//     ),
//     op_divide_year_month_duration
// );
//
// pub(crate) fn op_divide_year_month_duration(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
//     todo!()
// }
//
// // op:divide-yearMonthDuration-by-yearMonthDuration($arg1 as xs:yearMonthDuration, $arg2 as xs:yearMonthDuration) as xs:decimal
// pub(crate) static OP_DIVIDE_YEAR_MONTH_DURATION_BY_YEAR_MONTH_DURATION() -> FUNCTION {
//     (
//     (
//         [
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_YEAR_MONTH_DURATION)),
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_YEAR_MONTH_DURATION))
//         ].to_vec(),
//         SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DECIMAL.into())),
//     ),
//     op_divide_year_month_duration_by_year_month_duration
// );
//
// pub(crate) fn op_divide_year_month_duration_by_year_month_duration(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
//     todo!()
// }
//
// // op:add-dayTimeDurations($arg1 as xs:dayTimeDuration, $arg2 as xs:dayTimeDuration) as xs:dayTimeDuration
// pub(crate) static OP_ADD_DAY_TIME_DURATIONS() -> FUNCTION {
//     (
//     (
//         [
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DAY_TIME_DURATION.into())),
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DAY_TIME_DURATION.into()))
//         ].to_vec(),
//         SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DAY_TIME_DURATION.into())),
//     ),
//     op_add_day_time_durations
// );
//
// pub(crate) fn op_add_day_time_durations(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
//     todo!()
// }
//
// // op:subtract-dayTimeDurations($arg1 as xs:dayTimeDuration, $arg2 as xs:dayTimeDuration) as xs:dayTimeDuration
// pub(crate) static OP_SUBTRACT_DAY_TIME_DURATIONS() -> FUNCTION {
//     (
//     (
//         [
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DAY_TIME_DURATION.into())),
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DAY_TIME_DURATION.into()))
//         ].to_vec(),
//         SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DAY_TIME_DURATION.into())),
//     ),
//     op_subtract_day_time_durations
// );
//
// pub(crate) fn op_subtract_day_time_durations(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
//     todo!()
// }
//
// // op:multiply-dayTimeDuration($arg1 as xs:dayTimeDuration, $arg2 as xs:double) as xs:dayTimeDuration
// pub(crate) static OP_MULTIPLY_DAY_TIME_DURATIONS() -> FUNCTION {
//     (
//     (
//         [
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DAY_TIME_DURATION.into())),
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DAY_TIME_DURATION.into()))
//         ].to_vec(),
//         SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DAY_TIME_DURATION.into())),
//     ),
//     op_multiply_day_time_durations
// );
//
// pub(crate) fn op_multiply_day_time_durations(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
//     todo!()
// }
//
// // op:divide-dayTimeDuration($arg1 as xs:dayTimeDuration, $arg2 as xs:double) as xs:dayTimeDuration
// pub(crate) static OP_DIVIDE_DAY_TIME_DURATION() -> FUNCTION {
//     (
//     (
//         [
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DAY_TIME_DURATION.into())),
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DOUBLE))
//         ].to_vec(),
//         SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DAY_TIME_DURATION.into())),
//     ),
//     op_divide_day_time_duration
// );
//
// pub(crate) fn op_divide_day_time_duration(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
//     todo!()
// }
//
// // op:divide-dayTimeDuration-by-dayTimeDuration($arg1 as xs:dayTimeDuration, $arg2 as xs:dayTimeDuration) as xs:decimal
// pub(crate) static OP_DIVIDE_DAY_TIME_DURATION_BY_DAY_TIME_DURATION() -> FUNCTION {
//     (
//     (
//         [
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DAY_TIME_DURATION.into())),
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DAY_TIME_DURATION.into()))
//         ].to_vec(),
//         SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DECIMAL.into())),
//     ),
//     op_divide_day_time_duration_by_day_time_duration
// );
//
// pub(crate) fn op_divide_day_time_duration_by_day_time_duration(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
//     todo!()
// }

// fn:year-from-dateTime($arg as xs:dateTime?) as xs:integer?
pub(crate) fn FN_YEAR_FROM_DATE_TIME() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DATE_TIME.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_INTEGER.into())),
        ),
        fn_year_from_date_time
    )
}

pub(crate) fn fn_year_from_date_time(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// fn:month-from-dateTime($arg as xs:dateTime?) as xs:integer?
pub(crate) fn FN_MONTH_FROM_DATE_TIME() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DATE_TIME.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_INTEGER.into())),
        ),
        fn_month_from_date_time
    )
}

pub(crate) fn fn_month_from_date_time(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// fn:day-from-dateTime($arg as xs:dateTime?) as xs:integer?
pub(crate) fn FN_DAY_FROM_DATE_TIME() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DATE_TIME.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_INTEGER.into())),
        ),
        fn_day_from_date_time
    )
}

pub(crate) fn fn_day_from_date_time(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// fn:hours-from-dateTime($arg as xs:dateTime?) as xs:integer?
pub(crate) fn FN_HOURS_FROM_DATE_TIME() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DATE_TIME.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_INTEGER.into())),
        ),
        fn_hours_from_date_time
    )
}

pub(crate) fn fn_hours_from_date_time(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// fn:minutes-from-dateTime($arg as xs:dateTime?) as xs:integer?
pub(crate) fn FN_MINUTES_FROM_DATE_TIME() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DATE_TIME.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_INTEGER.into())),
        ),
        fn_minutes_from_date_time
    )
}

pub(crate) fn fn_minutes_from_date_time(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// fn:seconds-from-dateTime($arg as xs:dateTime?) as xs:decimal?
pub(crate) fn FN_SECONDS_FROM_DATE_TIME() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DATE_TIME.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DECIMAL.into())),
        ),
        fn_seconds_from_date_time
    )
}

pub(crate) fn fn_seconds_from_date_time(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// fn:timezone-from-dateTime($arg as xs:dateTime?) as xs:dayTimeDuration?
pub(crate) fn FN_TIMEZONE_FROM_DATE_TIME() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DATE_TIME.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DAY_TIME_DURATION.into())),
        ),
        fn_timezone_from_date_time
    )
}

pub(crate) fn fn_timezone_from_date_time(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// fn:year-from-date($arg as xs:date?) as xs:integer?
pub(crate) fn FN_YEAR_FROM_DATE() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DATE.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_INTEGER.into())),
        ),
        fn_year_from_date
    )
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

// fn:month-from-date($arg as xs:date?) as xs:integer?
pub(crate) fn FN_MONTH_FROM_DATE() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DATE.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_INTEGER.into())),
        ),
        fn_month_from_date
    )
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

// fn:day-from-date($arg as xs:date?) as xs:integer?
pub(crate) fn FN_DAY_FROM_DATE() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DATE.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_INTEGER.into())),
        ),
        fn_day_from_date
    )
}

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

// fn:timezone-from-date($arg as xs:date?) as xs:dayTimeDuration?
pub(crate) fn FN_TIMEZONE_FROM_DATE() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DATE.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DAY_TIME_DURATION.into())),
        ),
        fn_timezone_from_date
    )
}

pub(crate) fn fn_timezone_from_date(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// fn:hours-from-time($arg as xs:time?) as xs:integer?
pub(crate) fn FN_HOURS_FROM_TIME() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_TIME.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_INTEGER.into())),
        ),
        fn_hours_from_time
    )
}

pub(crate) fn fn_hours_from_time(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// fn:minutes-from-time($arg as xs:time?) as xs:integer?
pub(crate) fn FN_MINUTES_FROM_TIME() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_TIME.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_INTEGER.into())),
        ),
        fn_minutes_from_time
    )
}

pub(crate) fn fn_minutes_from_time(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// fn:seconds-from-time($arg as xs:time?) as xs:decimal?
pub(crate) fn FN_SECONDS_FROM_TIME() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_TIME.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DECIMAL.into())),
        ),
        fn_seconds_from_time
    )
}

pub(crate) fn fn_seconds_from_time(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// fn:timezone-from-time($arg as xs:time?) as xs:dayTimeDuration?
pub(crate) fn FN_TIMEZONE_FROM_TIME() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_TIME.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DAY_TIME_DURATION.into())),
        ),
        fn_timezone_from_time
    )
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

// fn:adjust-dateTime-to-timezone($arg as xs:dateTime?) as xs:dateTime?
pub(crate) fn FN_ADJUST_DATE_TIME_TO_TIMEZONE_1() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DATE_TIME.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DATE_TIME.into())),
        ),
        fn_adjust_date_time_to_timezone
    )
}

// fn:adjust-dateTime-to-timezone($arg as xs:dateTime?, $timezone as xs:dayTimeDuration?) as xs:dateTime?
pub(crate) fn FN_ADJUST_DATE_TIME_TO_TIMEZONE_2() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DATE_TIME.into())),
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DAY_TIME_DURATION.into()))
            ].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DATE_TIME.into())),
        ),
        fn_adjust_date_time_to_timezone
    )
}

pub(crate) fn fn_adjust_date_time_to_timezone(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// fn:adjust-date-to-timezone($arg as xs:date?) as xs:date?
pub(crate) fn FN_ADJUST_DATE_TO_TIMEZONE_1() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DATE.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DATE.into())),
        ),
        fn_adjust_date_to_timezone
    )
}

// fn:adjust-date-to-timezone($arg as xs:date?, $timezone as xs:dayTimeDuration?) as xs:date?
pub(crate) fn FN_ADJUST_DATE_TO_TIMEZONE_2() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DATE.into())),
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DAY_TIME_DURATION.into()))
            ].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DATE.into())),
        ),
        fn_adjust_date_to_timezone
    )
}

pub(crate) fn fn_adjust_date_to_timezone(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// fn:adjust-time-to-timezone($arg as xs:time?) as xs:time?
pub(crate) fn FN_ADJUST_TIME_TO_TIMEZONE_1() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_TIME.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_TIME.into())),
        ),
        fn_adjust_time_to_timezone
    )
}

// fn:adjust-time-to-timezone($arg as xs:time?, $timezone as xs:dayTimeDuration?) as xs:time?
pub(crate) fn FN_ADJUST_TIME_TO_TIMEZONE_2() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_TIME.into())),
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DAY_TIME_DURATION.into()))
            ].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_TIME.into())),
        ),
        fn_adjust_time_to_timezone
    )
}

pub(crate) fn fn_adjust_time_to_timezone(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// // op:subtract-dateTimes($arg1 as xs:dateTime, $arg2 as xs:dateTime) as xs:dayTimeDuration
// pub(crate) fn OP_SUBTRACT_DATE_TIMES() -> FUNCTION {
//     (
//     (
//         [
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DATE_TIME.into())),
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DATE_TIME.into()))
//         ].to_vec(),
//         SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DAY_TIME_DURATION.into())),
//     ),
//     op_subtract_date_times
// );
//
// pub(crate) fn op_subtract_date_times(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
//     todo!()
// }
//
// // op:subtract-dates($arg1 as xs:date, $arg2 as xs:date) as xs:dayTimeDuration
// pub(crate) static OP_SUBTRACT_DATES() -> FUNCTION {
//     (
//     (
//         [
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DATE)),
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DATE))
//         ].to_vec(),
//         SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DAY_TIME_DURATION.into())),
//     ),
//     op_subtract_dates
// );
//
// pub(crate) fn op_subtract_dates(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
//     todo!()
// }
//
// // op:subtract-times($arg1 as xs:time, $arg2 as xs:time) as xs:dayTimeDuration
// pub(crate) static OP_SUBTRACT_TIMES() -> FUNCTION {
//     (
//     (
//         [
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_TIME.into())),
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_TIME.into()))
//         ].to_vec(),
//         SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DAY_TIME_DURATION.into())),
//     ),
//     op_subtract_times
// );
//
// pub(crate) fn op_subtract_times(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
//     todo!()
// }
//
// // op:add-yearMonthDuration-to-dateTime($arg1 as xs:dateTime, $arg2 as xs:yearMonthDuration) as xs:dateTime
// pub(crate) static OP_ADD_YEAR_MONTH_DURATION_TO_DATE_TIME() -> FUNCTION {
//     (
//     (
//         [
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DATE_TIME.into())),
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_YEAR_MONTH_DURATION))
//         ].to_vec(),
//         SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DATE_TIME.into())),
//     ),
//     op_add_year_month_duration_to_date_time
// );
//
// pub(crate) fn op_add_year_month_duration_to_date_time(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
//     todo!()
// }
//
// // op:add-dayTimeDuration-to-dateTime($arg1 as xs:dateTime, $arg2 as xs:dayTimeDuration) as xs:dateTime
// pub(crate) static OP_ADD_DAY_TIME_DURATION_TO_DATE_TIME() -> FUNCTION {
//     (
//     (
//         [
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DATE_TIME.into())),
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DAY_TIME_DURATION.into()))
//         ].to_vec(),
//         SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DATE_TIME.into())),
//     ),
//     op_add_day_time_duration_to_date_time
// );
//
// pub(crate) fn op_add_day_time_duration_to_date_time(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
//     todo!()
// }
//
// // op:subtract-yearMonthDuration-from-dateTime($arg1 as xs:dateTime, $arg2 as xs:yearMonthDuration) as xs:dateTime
// pub(crate) static OP_SUBTRACT_YEAR_MONTH_DURATION_FROM_DATE_TIME() -> FUNCTION {
//     (
//     (
//         [
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DATE_TIME.into())),
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_YEAR_MONTH_DURATION))
//         ].to_vec(),
//         SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DATE_TIME.into())),
//     ),
//     op_subtract_year_month_duration_from_date_time
// );
//
// pub(crate) fn op_subtract_year_month_duration_from_date_time(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
//     todo!()
// }
//
// // op:subtract-dayTimeDuration-from-dateTime($arg1 as xs:dateTime, $arg2 as xs:dayTimeDuration) as xs:dateTime
// pub(crate) static OP_SUBTRACT_DAY_TIME_DURATION_FROM_DATE_TIME() -> FUNCTION {
//     (
//     (
//         [
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DATE_TIME.into())),
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DAY_TIME_DURATION.into()))
//         ].to_vec(),
//         SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DATE_TIME.into())),
//     ),
//     op_subtract_day_time_duration_from_date_time
// );
//
// pub(crate) fn op_subtract_day_time_duration_from_date_time(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
//     todo!()
// }
//
// // op:add-yearMonthDuration-to-date($arg1 as xs:date, $arg2 as xs:yearMonthDuration) as xs:date
// pub(crate) static OP_ADD_YEAR_MONTH_DURATION_TO_DATE() -> FUNCTION {
//     (
//     (
//         [
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DATE)),
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_YEAR_MONTH_DURATION))
//         ].to_vec(),
//         SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DATE)),
//     ),
//     op_add_year_month_duration_to_date
// );
//
// pub(crate) fn op_add_year_month_duration_to_date(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
//     todo!()
// }
//
// // op:add-dayTimeDuration-to-date($arg1 as xs:date, $arg2 as xs:dayTimeDuration) as xs:date
// pub(crate) static OP_ADD_DAY_TIME_DURATION_TO_DATE() -> FUNCTION {
//     (
//     (
//         [
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DATE)),
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DAY_TIME_DURATION.into()))
//         ].to_vec(),
//         SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DATE)),
//     ),
//     op_add_day_time_duration_to_date
// );
//
// pub(crate) fn op_add_day_time_duration_to_date(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
//     todo!()
// }
//
// // op:subtract-yearMonthDuration-from-date($arg1 as xs:date, $arg2 as xs:yearMonthDuration) as xs:date
// pub(crate) static OP_SUBTRACT_YEAR_MONTH_DURATION_FROM_DATE() -> FUNCTION {
//     (
//     (
//         [
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DATE)),
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_YEAR_MONTH_DURATION))
//         ].to_vec(),
//         SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DATE)),
//     ),
//     op_subtract_year_month_duration_from_date
// );
//
// pub(crate) fn op_subtract_year_month_duration_from_date(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
//     todo!()
// }
//
// // op:subtract-dayTimeDuration-from-date($arg1 as xs:date, $arg2 as xs:dayTimeDuration) as xs:date
// pub(crate) static OP_SUBTRACT_DAY_TIME_DURATION_FROM_DATE() -> FUNCTION {
//     (
//     (
//         [
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DATE)),
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DAY_TIME_DURATION.into()))
//         ].to_vec(),
//         SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DATE)),
//     ),
//     op_subtract_day_time_duration_from_date
// );
//
// pub(crate) fn op_subtract_day_time_duration_from_date(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
//     todo!()
// }
//
// // op:add-dayTimeDuration-to-time($arg1 as xs:time, $arg2 as xs:dayTimeDuration) as xs:time
// pub(crate) static OP_ADD_DAY_TIME_DURATION_TO_TIME() -> FUNCTION {
//     (
//     (
//         [
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_TIME.into())),
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DAY_TIME_DURATION.into()))
//         ].to_vec(),
//         SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_TIME.into())),
//     ),
//     op_add_day_time_duration_to_time
// );
//
// pub(crate) fn op_add_day_time_duration_to_time(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
//     todo!()
// }
//
// // op:subtract-dayTimeDuration-from-time($arg1 as xs:time, $arg2 as xs:dayTimeDuration) as xs:time
// pub(crate) static OP_SUBTRACT_DAY_TIME_DURATION_FROM_TIME() -> FUNCTION {
//     (
//     (
//         [
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_TIME.into())),
//             SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DAY_TIME_DURATION.into()))
//         ].to_vec(),
//         SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_TIME.into())),
//     ),
//     op_subtract_day_time_duration_from_time
// );
//
// pub(crate) fn op_subtract_day_time_duration_from_time(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
//     todo!()
// }

// fn:format-dateTime($value as xs:dateTime?, $picture as xs:string) as xs:string?
pub(crate) fn FN_FORMAT_DATE_TIME_2() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DATE_TIME.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
            ].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
        ),
        fn_format_date_time
    )
}

// fn:format-dateTime($value as xs:dateTime?, $picture as xs:string, $language as xs:string?, $calendar as xs:string?, $place as xs:string?) as xs:string?
pub(crate) fn FN_FORMAT_DATE_TIME_5() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DATE_TIME.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
            ].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
        ),
        fn_format_date_time
    )
}

pub(crate) fn fn_format_date_time(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// fn:format-date($value as xs:date?, $picture as xs:string) as xs:string?
pub(crate) fn FN_FORMAT_DATE_2() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DATE.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
            ].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
        ),
        fn_format_date
    )
}

// fn:format-date($value as xs:date?, $picture as xs:string, $language as xs:string?, $calendar as xs:string?, $place as xs:string?) as xs:string?
pub(crate) fn FN_FORMAT_DATE_5() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DATE.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
            ].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
        ),
        fn_format_date
    )
}

pub(crate) fn fn_format_date(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// fn:format-time($value as xs:time?, $picture as xs:string) as xs:string?
pub(crate) fn FN_FORMAT_TIME_2() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_TIME.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
            ].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
        ),
        fn_format_time
    )
}

// fn:format-time($value as xs:time?, $picture as xs:string, $language as xs:string?, $calendar as xs:string?, $place as xs:string?) as xs:string?
pub(crate) fn FN_FORMAT_TIME_5() -> FUNCTION {
    (
        (
            [
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_TIME.into())),
                SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
                SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
            ].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into())),
        ),
        fn_format_time
    )
}

pub(crate) fn fn_format_time(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// fn:parse-ietf-date($value as xs:string?) as xs:dateTime?
pub(crate) fn FN_PARSE_IETF_DATE() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into()))].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DATE_TIME.into())),
        ),
        fn_parse_ietf_date
    )
}

pub(crate) fn fn_parse_ietf_date(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// fn:current-dateTime() as xs:dateTimeStamp
pub(crate) fn FN_CURRENT_DATE_TIME() -> FUNCTION {
    (
        (
            [].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DATE_TIME_STAMP.into())),
        ),
        fn_current_date
    )
}

pub(crate) fn fn_current_date_time(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// fn:current-date() as xs:date
pub(crate) fn FN_CURRENT_DATE() -> FUNCTION {
    (
        (
            [].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DATE.into())),
        ),
        fn_current_date
    )
}

pub(crate) fn fn_current_date(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    // TODO  deterministic
    Ok((env, Object::Atomic(Type::date_now())))
}

// fn:current-time() as xs:time
pub(crate) fn FN_CURRENT_TIME() -> FUNCTION {
    (
        (
            [].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_TIME.into())),
        ),
        fn_current_time
    )
}

pub(crate) fn fn_current_time(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    // TODO  deterministic
    Ok((env, Object::Atomic(Type::time_now())))
}

// fn:implicit-timezone() as xs:dayTimeDuration
pub(crate) fn FN_IMPLICIT_TIMEZONE() -> FUNCTION {
    (
        (
            [].to_vec(),
            SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_DAY_TIME_DURATION.into())),
        ),
        fn_implicit_timezone
    )
}

pub(crate) fn fn_implicit_timezone(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
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