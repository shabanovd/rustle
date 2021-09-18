use crate::eval::{Object, Type, EvalResult, string_to_decimal};
use crate::eval::Environment;
use crate::parser::parse_duration::{string_to_dt_duration, string_to_ym_duration, string_to_duration, string_to_date};
use crate::parser::errors::ErrorCode;
use ordered_float::OrderedFloat;
use bigdecimal::{BigDecimal, FromPrimitive, ToPrimitive};

pub fn xs_untyped_atomic_eval<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context_item: &Object) -> EvalResult<'a> {
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

pub fn xs_ncname_eval<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context_item: &Object) -> EvalResult<'a> {
    match arguments.as_slice() {

        [Object::Atomic(Type::String(string))] =>
            Ok((env, Object::Atomic(Type::NCName(string.clone())))),

        _ => todo!()
    }
}

pub fn xs_anyuri_eval<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context_item: &Object) -> EvalResult<'a> {
    match arguments.as_slice() {

        [Object::Atomic(Type::String(string))] =>
            Ok((env, Object::Atomic(Type::AnyURI(string.clone())))),

        _ => todo!()
    }
}

pub fn xs_date_eval<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context_item: &Object) -> EvalResult<'a> {
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

pub fn xs_duration_eval<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context_item: &Object) -> EvalResult<'a> {
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

pub fn xs_day_time_duration_eval<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context_item: &Object) -> EvalResult<'a> {
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

pub fn xs_year_month_duration_eval<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context_item: &Object) -> EvalResult<'a> {
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

pub fn xs_integer_eval<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context_item: &Object) -> EvalResult<'a> {
    match arguments.as_slice() {
        [Object::Atomic(Type::String(string))] =>
            Ok((env, Object::Atomic(Type::Integer(string.parse::<i128>().unwrap())))),

        [Object::Atomic(Type::Integer(integer))] =>
            Ok((env, Object::Atomic(Type::Integer(*integer)))),

        _ => todo!()
    }
}

pub fn xs_non_positive_integer_eval<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> EvalResult<'a> {
    xs_integer_eval(env, arguments, context_item)
}

pub fn xs_negative_integer_eval<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> EvalResult<'a> {
    xs_integer_eval(env, arguments, context_item)
}

pub fn xs_long_eval<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> EvalResult<'a> {
    xs_integer_eval(env, arguments, context_item)
}

pub fn xs_int_eval<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> EvalResult<'a> {
    xs_integer_eval(env, arguments, context_item)
}

pub fn xs_short_eval<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> EvalResult<'a> {
    xs_integer_eval(env, arguments, context_item)
}

pub fn xs_byte_eval<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> EvalResult<'a> {
    xs_integer_eval(env, arguments, context_item)
}

pub fn xs_non_negative_integer_eval<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> EvalResult<'a> {
    xs_integer_eval(env, arguments, context_item)
}

pub fn xs_unsigned_long_eval<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> EvalResult<'a> {
    xs_integer_eval(env, arguments, context_item)
}

pub fn xs_unsigned_int_eval<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> EvalResult<'a> {
    xs_integer_eval(env, arguments, context_item)
}

pub fn xs_unsigned_short_eval<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> EvalResult<'a> {
    xs_integer_eval(env, arguments, context_item)
}

pub fn xs_unsigned_byte_eval<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> EvalResult<'a> {
    xs_integer_eval(env, arguments, context_item)
}

pub fn xs_positive_integer_eval<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, context_item: &Object) -> EvalResult<'a> {
    xs_integer_eval(env, arguments, context_item)
}

pub fn xs_decimal_eval<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context_item: &Object) -> EvalResult<'a> {
    match arguments.as_slice() {
        [Object::Atomic(Type::String(string))] => {
            match string_to_decimal(string) {
                Ok(number) => {
                    Ok((env, Object::Atomic(Type::Decimal(number))))
                },
                Err(code) => Err((code, String::from("TODO")))
            }
        },
        [Object::Atomic(Type::Integer(number))] => {
            if let Some(number) = BigDecimal::from_i128(*number) {
                Ok((env, Object::Atomic(Type::Decimal(number))))
            } else {
                Err((ErrorCode::FORG0001, String::from("TODO")))
            }
        },
        [Object::Atomic(Type::Decimal(number))] => {
            Ok((env, Object::Atomic(Type::Decimal(number.clone()))))
        },
        [Object::Atomic(Type::Float(number))] => {
            match BigDecimal::from_f32(number.into_inner()) {
                Some(number) => Ok((env, Object::Atomic(Type::Decimal(number)))),
                None => Err((ErrorCode::FORG0001, String::from("TODO")))
            }
        },
        [Object::Atomic(Type::Double(number))] => {
            match BigDecimal::from_f64(number.into_inner()) {
                Some(number) => Ok((env, Object::Atomic(Type::Decimal(number)))),
                None => Err((ErrorCode::FORG0001, String::from("TODO")))
            }
        },

        _ => todo!()
    }
}

pub fn xs_float_eval<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context_item: &Object) -> EvalResult<'a> {
    match arguments.as_slice() {
        [Object::Atomic(Type::String(string))] => {
            match string.parse() {
                Ok(number) => {
                    Ok((env, Object::Atomic(Type::Float(number))))
                },
                Err(..) => Err((ErrorCode::FORG0001, String::from("TODO")))
            }
        }
        [Object::Atomic(Type::Integer(number))] => {
            match OrderedFloat::from_i128(*number) {
                Some(number) => Ok((env, Object::Atomic(Type::Float(number)))),
                None => Err((ErrorCode::FORG0001, String::from("TODO")))
            }
        },
        [Object::Atomic(Type::Decimal(number))] => {
            match number.to_f32() {
                Some(number) => {
                    let number = OrderedFloat::from(number);
                    Ok((env, Object::Atomic(Type::Float(number))))
                },
                None => Err((ErrorCode::FORG0001, String::from("TODO")))
            }
        },
        [Object::Atomic(Type::Float(number))] => {
            Ok((env, Object::Atomic(Type::Float(*number))))
        },
        [Object::Atomic(Type::Double(number))] => {
            Ok((env, Object::Atomic(Type::Float(OrderedFloat::from(number.into_inner() as f32)))))
        },

        _ => todo!()
    }
}

pub fn xs_double_eval<'a>(env: Box<Environment<'a>>, arguments: Vec<Object>, _context_item: &Object) -> EvalResult<'a> {
    println!("arguments {:?}", arguments);
    match arguments.as_slice() {

        [Object::Atomic(Type::String(string))] => {
            match string.parse() {
                Ok(number) => {
                    Ok((env, Object::Atomic(Type::Double(number))))
                },
                Err(..) => Err((ErrorCode::FORG0001, String::from("TODO")))
            }
        },
        [Object::Atomic(Type::Integer(number))] => {
            if let Some(number) = OrderedFloat::from_i128(*number) {
                Ok((env, Object::Atomic(Type::Double(number))))
            } else {
                Err((ErrorCode::FORG0001, String::from("TODO")))
            }
        },
        [Object::Atomic(Type::Decimal(number))] => {
            match number.to_f64() {
                Some(number) => {
                    let number = OrderedFloat::from(number);
                    Ok((env, Object::Atomic(Type::Double(number))))
                },
                None => Err((ErrorCode::FORG0001, String::from("TODO")))
            }
        },
        [Object::Atomic(Type::Float(number))] => {
            Ok((env, Object::Atomic(Type::Double(OrderedFloat::from(number.into_inner() as f64)))))
        },
        [Object::Atomic(Type::Double(number))] => {
            Ok((env, Object::Atomic(Type::Double(*number))))
        },
        _ => todo!()
    }
}