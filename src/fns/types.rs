use crate::eval::{Object, string_to_decimal, DynamicContext, EvalResult};
use crate::eval::Environment;
use crate::parser::parse_duration::{string_to_dt_duration, string_to_ym_duration, string_to_duration, string_to_date, string_to_date_time};
use crate::parser::errors::ErrorCode;
use ordered_float::OrderedFloat;
use bigdecimal::{BigDecimal, FromPrimitive, ToPrimitive};
use crate::serialization::object_to_string;
use crate::fns::boolean::object_casting_bool;
use crate::values::{AnyURI, Boolean, Decimal, Double, Float, Integer, NCName, Str, Untyped};

pub(crate) fn xs_untyped_atomic_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    let item = arguments.get(0).unwrap();

    let str = object_to_string(&env, item);

    Ok((env, Object::Atomic(Untyped::boxed(str))))
}

pub(crate) fn xs_boolean_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    let item = arguments.get(0).unwrap();

    match object_casting_bool(item, true) {
        Ok(v) => Ok((env, Object::Atomic(Boolean::boxed(v)))),
        Err(e) => return Err(e)
    }
}

pub(crate) fn xs_string_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    let item = arguments.get(0).unwrap();

    let str = object_to_string(&env, item);

    Ok((env, Object::Atomic(Str::boxed(str))))
}

pub(crate) fn xs_ncname_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {

        [Object::Atomic(Str(str))] =>
            Ok((env, Object::Atomic(NCName::boxed(str.clone())))),

        _ => todo!()
    }
}

pub(crate) fn xs_anyuri_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {

        [Object::Atomic(Str(str))] =>
            Ok((env, Object::Atomic(AnyURI::boxed(str.clone())))),

        _ => todo!()
    }
}

pub(crate) fn xs_date_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Atomic(Str(str))] => {
            match string_to_date(str) {
                Ok(dt) => Ok((env, Object::Atomic(Box::new(dt)))),
                Err(e) => {
                    println!("{}", e);
                    todo!()
                }
            }
        },
        _ => todo!()
    }
}

pub(crate) fn xs_date_time_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Atomic(Str(str))] => {
            match string_to_date_time(str) {
                Ok(dt) => Ok((env, Object::Atomic(Box::new(dt)))),
                Err(e) => {
                    println!("{}", e);
                    todo!()
                }
            }
        },
        _ => todo!()
    }
}

pub(crate) fn xs_duration_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Atomic(Str(str))] => {
            match string_to_duration(str) {
                Ok(dt) => Ok((env, Object::Atomic(Box::new(dt)))),
                Err(e) => {
                    println!("{}", e);
                    todo!()
                }
            }
        },
        _ => todo!()
    }
}

pub(crate) fn xs_day_time_duration_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Atomic(Str(str))] => {
            match string_to_dt_duration(str) {
                Ok(dt) => Ok((env, Object::Atomic(Box::new(dt)))),
                Err(e) => {
                    println!("{}", e);
                    todo!()
                }
            }
        },
        _ => todo!()
    }
}

pub(crate) fn xs_year_month_duration_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Atomic(Str(str))] => {
            match string_to_ym_duration(str) {
                Ok(dt) => Ok((env, Object::Atomic(Box::new(dt)))),
                Err(..) => todo!()
            }
        },
        _ => todo!()
    }
}

pub(crate) fn xs_integer_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Atomic(Str(str))] =>
            Ok((env, Object::Atomic(Integer::boxed(str.parse::<i128>().unwrap())))),

        [Object::Atomic(Integer(integer))] =>
            Ok((env, Object::Atomic(Integer::boxed(*integer)))),

        _ => todo!()
    }
}

pub(crate) fn xs_non_positive_integer_eval(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    xs_integer_eval(env, arguments, context)
}

pub(crate) fn xs_negative_integer_eval(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    xs_integer_eval(env, arguments, context)
}

pub(crate) fn xs_long_eval(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    xs_integer_eval(env, arguments, context)
}

pub(crate) fn xs_int_eval(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    xs_integer_eval(env, arguments, context)
}

pub(crate) fn xs_short_eval(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    xs_integer_eval(env, arguments, context)
}

pub(crate) fn xs_byte_eval(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    xs_integer_eval(env, arguments, context)
}

pub(crate) fn xs_non_negative_integer_eval(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    xs_integer_eval(env, arguments, context)
}

pub(crate) fn xs_unsigned_long_eval(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    xs_integer_eval(env, arguments, context)
}

pub(crate) fn xs_unsigned_int_eval(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    xs_integer_eval(env, arguments, context)
}

pub(crate) fn xs_unsigned_short_eval(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    xs_integer_eval(env, arguments, context)
}

pub(crate) fn xs_unsigned_byte_eval(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    xs_integer_eval(env, arguments, context)
}

pub(crate) fn xs_positive_integer_eval(env: Box<Environment>, arguments: Vec<Object>, context: &DynamicContext) -> EvalResult {
    xs_integer_eval(env, arguments, context)
}

pub(crate) fn xs_decimal_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Atomic(Str(str))] => {
            match string_to_decimal(str) {
                Ok(number) => {
                    Ok((env, Object::Atomic(Decimal::boxed(number))))
                },
                Err(code) => Err((code, String::from("TODO")))
            }
        },
        [Object::Atomic(Integer(number))] => {
            if let Some(number) = BigDecimal::from_i128(*number) {
                Ok((env, Object::Atomic(Decimal::boxed(number))))
            } else {
                Err((ErrorCode::FORG0001, String::from("TODO")))
            }
        },
        [Object::Atomic(Decimal(number))] => {
            Ok((env, Object::Atomic(Decimal::boxed(number.clone()))))
        },
        [Object::Atomic(Float(number))] => {
            match BigDecimal::from_f32(number.into_inner()) {
                Some(number) => Ok((env, Object::Atomic(Decimal::boxed(number)))),
                None => Err((ErrorCode::FORG0001, String::from("TODO")))
            }
        },
        [Object::Atomic(Double(number))] => {
            match BigDecimal::from_f64(number.into_inner()) {
                Some(number) => Ok((env, Object::Atomic(Decimal::boxed(number)))),
                None => Err((ErrorCode::FORG0001, String::from("TODO")))
            }
        },

        _ => todo!()
    }
}

pub(crate) fn xs_float_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Atomic(Str(str))] => {
            match str.parse() {
                Ok(number) => {
                    Ok((env, Object::Atomic(Float::boxed(number))))
                },
                Err(..) => Err((ErrorCode::FORG0001, String::from("TODO")))
            }
        }
        [Object::Atomic(Integer(number))] => {
            match OrderedFloat::from_i128(*number) {
                Some(number) => Ok((env, Object::Atomic(Float::boxed(number)))),
                None => Err((ErrorCode::FORG0001, String::from("TODO")))
            }
        },
        [Object::Atomic(Decimal(number))] => {
            match number.to_f32() {
                Some(number) => {
                    let number = OrderedFloat::from(number);
                    Ok((env, Object::Atomic(Float::boxed(number))))
                },
                None => Err((ErrorCode::FORG0001, String::from("TODO")))
            }
        },
        [Object::Atomic(Float(number))] => {
            Ok((env, Object::Atomic(Float::boxed(*number))))
        },
        [Object::Atomic(Double(number))] => {
            Ok((env, Object::Atomic(Float::boxed(OrderedFloat::from(number.into_inner() as f32)))))
        },

        _ => todo!()
    }
}

pub(crate) fn xs_double_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {

        [Object::Atomic(Str(str))] => {
            match str.parse() {
                Ok(number) => {
                    Ok((env, Object::Atomic(Double::boxed(number))))
                },
                Err(..) => Err((ErrorCode::FORG0001, String::from("TODO")))
            }
        },
        [Object::Atomic(Integer(number))] => {
            if let Some(number) = OrderedFloat::from_i128(*number) {
                Ok((env, Object::Atomic(Double::boxed(number))))
            } else {
                Err((ErrorCode::FORG0001, String::from("TODO")))
            }
        },
        [Object::Atomic(Decimal(number))] => {
            match number.to_f64() {
                Some(number) => {
                    let number = OrderedFloat::from(number);
                    Ok((env, Object::Atomic(Double::boxed(number))))
                },
                None => Err((ErrorCode::FORG0001, String::from("TODO")))
            }
        },
        [Object::Atomic(Float(number))] => {
            Ok((env, Object::Atomic(Double::boxed(OrderedFloat::from(number.into_inner() as f64)))))
        },
        [Object::Atomic(Double(number))] => {
            Ok((env, Object::Atomic(Double::boxed(*number))))
        },
        _ => todo!()
    }
}