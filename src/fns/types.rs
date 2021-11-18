use crate::eval::{Object, Type, DynamicContext, EvalResult};
use crate::eval::Environment;
use crate::parser::parse_duration::{string_to_dt_duration, string_to_ym_duration, string_to_duration, string_to_date, string_to_date_time};
use crate::parser::errors::ErrorCode;
use ordered_float::OrderedFloat;
use bigdecimal::{BigDecimal, FromPrimitive, ToPrimitive};
use crate::serialization::object_to_string;
use crate::fns::boolean::object_casting_bool;

pub(crate) fn xs_untyped_atomic_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    let item = arguments.get(0).unwrap();

    let str = object_to_string(&env, item);

    Ok((env, Object::Atomic(Type::Untyped(str))))
}

pub(crate) fn xs_boolean_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    let item = arguments.get(0).unwrap();

    match object_casting_bool(item, true) {
        Ok(v) => Ok((env, Object::Atomic(Type::Boolean(v)))),
        Err(e) => return Err(e)
    }
}

pub(crate) fn xs_string_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    let item = arguments.get(0).unwrap();
    match item {
        Object::Empty => Ok((env, Object::Empty)),
        _ => {
            let str = object_to_string(&env, item);

            Ok((env, Object::Atomic(Type::String(str))))
        }
    }
}

pub(crate) fn xs_hex_binary_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    let item = arguments.get(0).unwrap();

    let str = object_to_string(&env, item);

    Ok((env, Object::Atomic(Type::HexBinary(str))))
}

pub(crate) fn xs_ncname_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Atomic(Type::Untyped(string))] |
        [Object::Atomic(Type::String(string))] =>
            Ok((env, Object::Atomic(Type::NCName(string.clone())))),

        _ => todo!()
    }
}

pub(crate) fn xs_anyuri_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Atomic(Type::Untyped(string))] |
        [Object::Atomic(Type::String(string))] =>
            Ok((env, Object::Atomic(Type::AnyURI(string.clone())))),

        _ => todo!()
    }
}

pub(crate) fn xs_date_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Atomic(Type::Untyped(string))] |
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

pub(crate) fn xs_date_time_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Atomic(Type::Untyped(string))] |
        [Object::Atomic(Type::String(string))] => {
            match string_to_date_time(string) {
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

pub(crate) fn xs_duration_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Atomic(Type::Untyped(string))] |
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

pub(crate) fn xs_day_time_duration_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Atomic(Type::Untyped(string))] |
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

pub(crate) fn xs_year_month_duration_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Atomic(Type::Untyped(string))] |
        [Object::Atomic(Type::String(string))] => {
            match string_to_ym_duration(string) {
                Ok(dt) => Ok((env, Object::Atomic(dt))),
                Err(..) => todo!()
            }
        },
        _ => todo!()
    }
}

pub(crate) fn xs_integer_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Empty] => Ok((env, Object::Empty)),
        [Object::Atomic(Type::Untyped(string))] |
        [Object::Atomic(Type::String(string))] |
        [Object::Atomic(Type::NormalizedString(string))] => {
            let value = crate::values::string_to::integer(string)?;
            Ok((env, Object::Atomic(value)))
        }
        [Object::Atomic(Type::Integer(integer))] =>
            Ok((env, Object::Atomic(Type::Integer(*integer)))),

        [Object::Atomic(Type::Decimal(num))] => {
            if let Some(num) = num.round(0).to_i128() {
                Ok((env, Object::Atomic(Type::Integer(num))))
            } else {
                Err((ErrorCode::TODO, String::from("TODO")))
            }
        }
        [Object::Atomic(Type::Float(num))] => {
            if let Some(num) = num.0.round().to_i128() {
                Ok((env, Object::Atomic(Type::Integer(num))))
            } else {
                Err((ErrorCode::TODO, String::from("TODO")))
            }
        }
        [Object::Atomic(Type::Double(num))] => {
            if let Some(num) = num.0.round().to_i128() {
                Ok((env, Object::Atomic(Type::Integer(num))))
            } else {
                Err((ErrorCode::TODO, String::from("TODO")))
            }
        }
        [Object::Node(rf)] => {
            match rf.to_typed_value() {
                Ok(str) => {
                    let value = crate::values::string_to::integer(&str)?;
                    Ok((env, Object::Atomic(value)))
                },
                Err(msg) => Err((ErrorCode::FORG0001, msg))
            }
        }
        _ => todo!("{:?}", arguments)
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
        [Object::Empty] => Ok((env, Object::Empty)),
        [Object::Atomic(Type::Untyped(string))] |
        [Object::Atomic(Type::String(string))] |
        [Object::Atomic(Type::NormalizedString(string))] => {
            let value = crate::values::string_to::decimal(string)?;
            Ok((env, Object::Atomic(value)))
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
        [Object::Node(rf)] => {
            match rf.to_typed_value() {
                Ok(str) => {
                    let value = crate::values::string_to::decimal(&str)?;
                    Ok((env, Object::Atomic(value)))
                },
                Err(msg) => Err((ErrorCode::FORG0001, msg))
            }
        }
        _ => todo!()
    }
}

pub(crate) fn xs_float_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Empty] => Ok((env, Object::Empty)),
        [Object::Atomic(Type::Untyped(string))] |
        [Object::Atomic(Type::String(string))] |
        [Object::Atomic(Type::NormalizedString(string))] => {
            let value = crate::values::string_to::float(string, false)?;
            Ok((env, Object::Atomic(value)))
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
        [Object::Node(rf)] => {
            match rf.to_typed_value() {
                Ok(str) => {
                    let value = crate::values::string_to::float(&str, false)?;
                    Ok((env, Object::Atomic(value)))
                },
                Err(msg) => Err((ErrorCode::FORG0001, msg))
            }
        }
        _ => todo!()
    }
}

pub(crate) fn xs_double_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Empty] => Ok((env, Object::Empty)),
        [Object::Atomic(Type::Untyped(string))] |
        [Object::Atomic(Type::String(string))] |
        [Object::Atomic(Type::NormalizedString(string))] => {
            let value = crate::values::string_to::double(string, false)?;
            Ok((env, Object::Atomic(value)))
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
        [Object::Node(rf)] => {
            match rf.to_typed_value() {
                Ok(str) => {
                    let value = crate::values::string_to::double(&str, false)?;
                    Ok((env, Object::Atomic(value)))
                },
                Err(msg) => Err((ErrorCode::FORG0001, msg))
            }
        }
        _ => todo!()
    }
}