use crate::eval::{Environment, Object, Type, DynamicContext, EvalResult, ErrorInfo};
use crate::parser::parse_duration::*;
use crate::parser::errors::ErrorCode;
use ordered_float::OrderedFloat;
use bigdecimal::{BigDecimal, FromPrimitive, ToPrimitive};
use crate::eval::sequence_type::*;
use crate::serialization::object_to_string;
use crate::fns::boolean::object_casting_bool;
use crate::fns::FUNCTION;
use crate::values::{string_to_binary_base64, string_to_binary_hex, string_to_qname, Types};

fn empty_or_type(env: Box<Environment>, arguments: Vec<Object>, processing: fn(&Box<Environment>, &Object) -> Result<Type, ErrorInfo>) -> EvalResult {
    let item = arguments.get(0).unwrap();
    match item {
        Object::Empty => Ok((env, Object::Empty)),
        _ => {
            let t = processing(&env, item)?;
            Ok((env, Object::Atomic(t)))
        }
    }
}

// xs:string($arg as xs:anyAtomicType?) as xs:string?
pub(crate) fn FN_XS_STRING() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_STRING.into()))
        ),
        xs_string_eval
    )
}

pub(crate) fn xs_string_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    empty_or_type(env, arguments, |env, item| {
        let str = object_to_string(&env, item);
        Ok(Type::String(str))
    })
}

//xs:boolean($arg as xs:anyAtomicType?) as xs:boolean?
pub(crate) fn FN_XS_BOOLEAN() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_BOOLEAN.into()))
        ),
        xs_boolean_eval
    )
}

pub(crate) fn xs_boolean_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    empty_or_type(env, arguments, |env, item| {
        match item {
            Object::Atomic(t) => {
                let n = t.convert(Types::Boolean)?;
                Ok(n)
            }
            _ => Err((ErrorCode::TODO, String::from("TODO")))
        }
        // match object_casting_bool(item, true) {
        //     Ok(v) => Ok(Type::Boolean(v)),
        //     Err(e) => return Err(e)
        // }
    })
}

// xs:decimal($arg as xs:anyAtomicType?) as xs:decimal?
pub(crate) fn FN_XS_DECIMAL() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DECIMAL.into()))
        ),
        xs_decimal_eval
    )
}

pub(crate) fn xs_decimal_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Empty] => Ok((env, Object::Empty)),
        [Object::Atomic(t)] => {
            let n = t.convert(Types::Decimal)?;
            Ok((env, Object::Atomic(n)))
        }
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

// xs:float($arg as xs:anyAtomicType?) as xs:float?
pub(crate) fn FN_XS_FLOAT() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_FLOAT.into()))
        ),
        xs_float_eval
    )
}

pub(crate) fn xs_float_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Empty] => Ok((env, Object::Empty)),
        [Object::Atomic(t)] => {
            let n = t.convert(Types::Float)?;
            Ok((env, Object::Atomic(n)))
        }
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

// xs:double($arg as xs:anyAtomicType?) as xs:double?
pub(crate) fn FN_XS_DOUBLE() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DOUBLE.into()))
        ),
        xs_double_eval
    )
}

pub(crate) fn xs_double_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Empty] => Ok((env, Object::Empty)),
        [Object::Atomic(t)] => {
            let n = t.convert(Types::Double)?;
            Ok((env, Object::Atomic(n)))
        }
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

// xs:duration($arg as xs:anyAtomicType?) as xs:duration?
pub(crate) fn FN_XS_DURATION() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DURATION.into()))
        ),
        xs_duration_eval
    )
}

pub(crate) fn xs_duration_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Atomic(t)] => {
            let n = t.convert(Types::Duration)?;
            Ok((env, Object::Atomic(n)))
        }
        _ => todo!()
    }
}

// xs:dateTime($arg as xs:anyAtomicType?) as xs:dateTime?
pub(crate) fn FN_XS_DATE_TIME() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DATE_TIME.into()))
        ),
        xs_date_time_eval
    )
}

pub(crate) fn xs_date_time_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Atomic(t)] => {
            let n = t.convert(Types::DateTime)?;
            Ok((env, Object::Atomic(n)))
        }
        _ => todo!()
    }
}

// xs:time($arg as xs:anyAtomicType?) as xs:time?
pub(crate) fn FN_XS_TIME() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_TIME.into()))
        ),
        xs_time_eval
    )
}

pub(crate) fn xs_time_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Atomic(t)] => {
            let n = t.convert(Types::Time)?;
            Ok((env, Object::Atomic(n)))
        }
        _ => todo!()
    }
}

// xs:date($arg as xs:anyAtomicType?) as xs:date?
pub(crate) fn FN_XS_DATE() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DATE.into()))
        ),
        xs_date_eval
    )
}

pub(crate) fn xs_date_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Atomic(t)] => {
            let n = t.convert(Types::Date)?;
            Ok((env, Object::Atomic(n)))
        }
        _ => todo!()
    }
}

// xs:gYearMonth($arg as xs:anyAtomicType?) as xs:gYearMonth?
pub(crate) fn FN_XS_G_YEAR_MONTH() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_G_YEAR_MONTH.into()))
        ),
        xs_g_year_month_eval
    )
}

pub(crate) fn xs_g_year_month_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Atomic(t)] => {
            let n = t.convert(Types::GYearMonth)?;
            Ok((env, Object::Atomic(n)))
        }
        _ => todo!()
    }
}

// xs:gYear($arg as xs:anyAtomicType?) as xs:gYear?
pub(crate) fn FN_XS_G_YEAR() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_G_YEAR.into()))
        ),
        xs_g_year_eval
    )
}

pub(crate) fn xs_g_year_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Atomic(t)] => {
            let n = t.convert(Types::GYear)?;
            Ok((env, Object::Atomic(n)))
        }
        _ => todo!()
    }
}

// xs:gMonthDay($arg as xs:anyAtomicType?) as xs:gMonthDay?
pub(crate) fn FN_XS_G_MONTH_DAY() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_G_MONTH_DAY.into()))
        ),
        xs_g_month_day_eval
    )
}

pub(crate) fn xs_g_month_day_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Atomic(t)] => {
            let n = t.convert(Types::GMonthDay)?;
            Ok((env, Object::Atomic(n)))
        }
        _ => todo!()
    }
}

// xs:gDay($arg as xs:anyAtomicType?) as xs:gDay?
pub(crate) fn FN_XS_G_DAY() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_G_DAY.into()))
        ),
        xs_g_day_eval
    )
}

pub(crate) fn xs_g_day_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Atomic(t)] => {
            let n = t.convert(Types::GDay)?;
            Ok((env, Object::Atomic(n)))
        }
        _ => todo!()
    }
}

// xs:gMonth($arg as xs:anyAtomicType?) as xs:gMonth?
pub(crate) fn FN_XS_G_MONTH() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_G_MONTH.into()))
        ),
        xs_g_month_eval
    )
}

pub(crate) fn xs_g_month_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Atomic(t)] => {
            let n = t.convert(Types::GMonth)?;
            Ok((env, Object::Atomic(n)))
        }
        _ => todo!()
    }
}

// xs:hexBinary($arg as xs:anyAtomicType?) as xs:hexBinary?
pub(crate) fn FN_XS_HEX_BINARY() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_HEX_BINARY.into()))
        ),
        xs_hex_binary_eval
    )
}

pub(crate) fn xs_hex_binary_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Atomic(Type::Untyped(string))] |
        [Object::Atomic(Type::String(string))] => {
            match string_to_binary_hex(string) {
                Ok(binary) => Ok((env, Object::Atomic(Type::HexBinary(binary)))),
                Err(e) => Err((ErrorCode::FORG0001, String::from("TODO")))
            }
        },
        _ => todo!()
    }
}

// xs:base64Binary($arg as xs:anyAtomicType?) as xs:base64Binary?
pub(crate) fn FN_XS_BASE64_BINARY() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_BASE64_BINARY.into()))
        ),
        xs_base64_binary_eval
    )
}

pub(crate) fn xs_base64_binary_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Atomic(Type::Untyped(string))] |
        [Object::Atomic(Type::String(string))] => {
            match string_to_binary_base64(string) {
                Ok(binary) => Ok((env, Object::Atomic(Type::Base64Binary(binary)))),
                Err(e) => todo!("{:?}", e)
            }
        },
        _ => todo!()
    }
}

// xs:anyURI($arg as xs:anyAtomicType?) as xs:anyURI?
pub(crate) fn FN_XS_ANY_URI() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_ANY_URI.into()))
        ),
        xs_any_uri_eval
    )
}

pub(crate) fn xs_any_uri_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Atomic(Type::Untyped(string))] |
        [Object::Atomic(Type::String(string))] =>
            Ok((env, Object::Atomic(Type::AnyURI(string.clone())))),

        _ => todo!()
    }
}

// xs:QName($arg as xs:anyAtomicType?) as xs:QName?
pub(crate) fn FN_XS_QNAME() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_QNAME.into()))
        ),
        xs_token_eval
    )
}

pub(crate) fn xs_qname_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Atomic(Type::Untyped(string))] |
        [Object::Atomic(Type::String(string))] => {
            let qname = string_to_qname(&env, string.clone());
            Ok((env, Object::Atomic(Type::QName { url: qname.url, prefix: qname.prefix, local_part: qname.local_part })))
        }
        _ => todo!()
    }
}

// TODO xs:normalizedString($arg as xs:anyAtomicType?) as xs:normalizedString?
// TODO xs:token($arg as xs:anyAtomicType?) as xs:token?
pub(crate) fn FN_XS_TOKEN() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_TOKEN.into()))
        ),
        xs_token_eval
    )
}

pub(crate) fn xs_token_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Atomic(Type::Untyped(string))] |
        [Object::Atomic(Type::String(string))] =>
            Ok((env, Object::Atomic(Type::NCName(string.clone())))),

        _ => todo!()
    }
}

// TODO xs:language($arg as xs:anyAtomicType?) as xs:language?
// TODO xs:NMTOKEN($arg as xs:anyAtomicType?) as xs:NMTOKEN?
// TODO xs:Name($arg as xs:anyAtomicType?) as xs:Name?

// xs:NCName($arg as xs:anyAtomicType?) as xs:NCName?
pub(crate) fn FN_XS_NCNAME() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_NCNAME.into()))
        ),
        xs_ncname_eval
    )
}

pub(crate) fn xs_ncname_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Atomic(Type::Untyped(string))] |
        [Object::Atomic(Type::String(string))] =>
            Ok((env, Object::Atomic(Type::NCName(string.clone())))),

        _ => todo!()
    }
}

// TODO xs:ID($arg as xs:anyAtomicType?) as xs:ID?
// TODO xs:IDREF($arg as xs:anyAtomicType?) as xs:IDREF?
// TODO xs:ENTITY($arg as xs:anyAtomicType?) as xs:ENTITY?

// xs:integer($arg as xs:anyAtomicType?) as xs:integer?
pub(crate) fn FN_XS_INTEGER() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_INTEGER.into()))
        ),
        xs_integer_eval
    )
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

// xs:nonPositiveInteger($arg as xs:anyAtomicType?) as xs:nonPositiveInteger?
pub(crate) fn FN_XS_NON_POSITIVE_INTEGER() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_NON_POSITIVE_INTEGER.into()))
        ),
        xs_integer_eval
    )
}

// xs:negativeInteger($arg as xs:anyAtomicType?) as xs:negativeInteger?
pub(crate) fn FN_XS_NEGATIVE_INTEGER() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_NEGATIVE_INTEGER.into()))
        ),
        xs_integer_eval
    )
}

// xs:long($arg as xs:anyAtomicType?) as xs:long?
pub(crate) fn FN_XS_LONG() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_LONG.into()))
        ),
        xs_integer_eval
    )
}

// xs:int($arg as xs:anyAtomicType?) as xs:int?
pub(crate) fn FN_XS_INT() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_INT.into()))
        ),
        xs_integer_eval
    )
}

// xs:short($arg as xs:anyAtomicType?) as xs:short?
pub(crate) fn FN_XS_SHORT() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_SHORT.into()))
        ),
        xs_integer_eval
    )
}

// xs:byte($arg as xs:anyAtomicType?) as xs:byte?
pub(crate) fn FN_XS_BYTE() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_BYTE.into()))
        ),
        xs_integer_eval
    )
}

// xs:nonNegativeInteger($arg as xs:anyAtomicType?) as xs:nonNegativeInteger?
pub(crate) fn FN_XS_NON_NEGATIVE_INTEGER() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_NON_NEGATIVE_INTEGER.into()))
        ),
        xs_integer_eval
    )
}

// xs:unsignedLong($arg as xs:anyAtomicType?) as xs:unsignedLong?
pub(crate) fn FN_XS_UNSIGNED_LONG() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_UNSIGNED_LONG.into()))
        ),
        xs_integer_eval
    )
}

// xs:unsignedInt($arg as xs:anyAtomicType?) as xs:unsignedInt?
pub(crate) fn FN_XS_UNSIGNED_INT() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_UNSIGNED_INT.into()))
        ),
        xs_integer_eval
    )
}

// xs:unsignedShort($arg as xs:anyAtomicType?) as xs:unsignedShort?
pub(crate) fn FN_XS_UNSIGNED_SHORT() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_UNSIGNED_SHORT.into()))
        ),
        xs_integer_eval
    )
}

// xs:unsignedByte($arg as xs:anyAtomicType?) as xs:unsignedByte?
pub(crate) fn FN_XS_UNSIGNED_BYTE() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_UNSIGNED_BYTE.into()))
        ),
        xs_integer_eval
    )
}

// xs:positiveInteger($arg as xs:anyAtomicType?) as xs:positiveInteger?
pub(crate) fn FN_XS_POSITIVE_INTEGER() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_POSITIVE_INTEGER.into()))
        ),
        xs_integer_eval
    )
}

// xs:yearMonthDuration($arg as xs:anyAtomicType?) as xs:yearMonthDuration?
pub(crate) fn FN_XS_YEAR_MONTH_DURATION() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_YEAR_MONTH_DURATION.into()))
        ),
        xs_year_month_duration_eval
    )
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

// xs:dayTimeDuration($arg as xs:anyAtomicType?) as xs:dayTimeDuration?
pub(crate) fn FN_XS_DAY_TIME_DURATION() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DAY_TIME_DURATION.into()))
        ),
        xs_day_time_duration_eval
    )
}

pub(crate) fn xs_day_time_duration_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    match arguments.as_slice() {
        [Object::Atomic(Type::Untyped(string))] |
        [Object::Atomic(Type::String(string))] => {
            match string_to_date_time_duration(string) {
                Ok(dt) => Ok((env, Object::Atomic(dt))),
                Err(e) => Err((ErrorCode::FORG0001, String::from("TODO"))),
            }
        },
        _ => todo!()
    }
}

// xs:untypedAtomic($arg as xs:anyAtomicType?) as xs:untypedAtomic?
pub(crate) fn FN_XS_UNTYPED_ATOMIC() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_UNTYPED_ATOMIC.into()))
        ),
        xs_untyped_atomic_eval
    )
}

pub(crate) fn xs_untyped_atomic_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    let item = arguments.get(0).unwrap();


    let str = object_to_string(&env, item);

    Ok((env, Object::Atomic(Type::Untyped(str))))
}

// xs:dateTimeStamp($arg as xs:anyAtomicType?) as xs:dateTimeStamp?
pub(crate) fn FN_XS_DATE_TIME_STAMP() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_DATE_TIME_STAMP.into()))
        ),
        xs_date_time_stamp_eval
    )
}

pub(crate) fn xs_date_time_stamp_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    todo!()
}

// TODO xs:NMTOKENS($arg as xs:anyAtomicType?) as xs:NMTOKEN*
// TODO xs:ENTITIES($arg as xs:anyAtomicType?) as xs:ENTITY*
// TODO xs:IDREFS($arg as xs:anyAtomicType?) as xs:IDREF*

// TODO xs:numeric($arg as xs:anyAtomicType?) as xs:numeric?

// TODO xs:error($arg as xs:anyAtomicType?) as xs:error?

// TODO eg:hatSize($arg as xs:anyAtomicType?) as my:hatSize?




