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

fn convert(to_type: Types, env: Box<Environment>, arguments: Vec<Object>) -> EvalResult {
    match arguments.as_slice() {
        [Object::Empty] => Ok((env, Object::Empty)),
        [Object::Atomic(t)] => {
            let n = t.convert(to_type)?;
            Ok((env, Object::Atomic(n)))
        }
        [Object::Node(rf)] => {
            match rf.to_typed_value() {
                Ok(str) => {
                    let t = Type::Untyped(str);
                    let n = t.convert(to_type)?;
                    Ok((env, Object::Atomic(n)))
                },
                Err(msg) => Err((ErrorCode::FORG0001, msg))
            }
        }
        _ => todo!("{:?}", arguments)
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
    convert(Types::String, env, arguments)
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
    convert(Types::Boolean, env, arguments)
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
    convert(Types::Decimal, env, arguments)
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
    convert(Types::Float, env, arguments)
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
    convert(Types::Double, env, arguments)
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
    convert(Types::Duration, env, arguments)
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
    convert(Types::DateTime, env, arguments)
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
    convert(Types::Time, env, arguments)
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
    convert(Types::Date, env, arguments)
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
    convert(Types::GYearMonth, env, arguments)
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
    convert(Types::GYear, env, arguments)
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
    convert(Types::GMonthDay, env, arguments)
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
    convert(Types::GDay, env, arguments)
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
    convert(Types::GMonth, env, arguments)
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
    convert(Types::HexBinary, env, arguments)
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
    convert(Types::Base64Binary, env, arguments)
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
    convert(Types::AnyURI, env, arguments)
}

// xs:QName($arg as xs:anyAtomicType?) as xs:QName?
pub(crate) fn FN_XS_QNAME() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_QNAME.into()))
        ),
        xs_qname_eval
    )
}

pub(crate) fn xs_qname_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    convert(Types::QName, env, arguments)
}

// xs:normalizedString($arg as xs:anyAtomicType?) as xs:normalizedString?
pub(crate) fn FN_XS_NORMALIZED_STRING() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_NORMALIZED_STRING.into()))
        ),
        xs_normalized_string_eval
    )
}

pub(crate) fn xs_normalized_string_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    convert(Types::NormalizedString, env, arguments)
}

// xs:token($arg as xs:anyAtomicType?) as xs:token?
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
    convert(Types::Token, env, arguments)
}

// xs:language($arg as xs:anyAtomicType?) as xs:language?
pub(crate) fn FN_XS_LANGUAGE() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_LANGUAGE.into()))
        ),
        xs_language_eval
    )
}

pub(crate) fn xs_language_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    convert(Types::Language, env, arguments)
}

// xs:NMTOKEN($arg as xs:anyAtomicType?) as xs:NMTOKEN?
pub(crate) fn FN_XS_NMTOKEN() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_NMTOKEN.into()))
        ),
        xs_nmtoken_eval
    )
}

pub(crate) fn xs_nmtoken_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    convert(Types::NMTOKEN, env, arguments)
}

// xs:Name($arg as xs:anyAtomicType?) as xs:Name?
pub(crate) fn FN_XS_NAME() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_NAME.into()))
        ),
        xs_name_eval
    )
}

pub(crate) fn xs_name_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    convert(Types::Name, env, arguments)
}

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
    convert(Types::NCName, env, arguments)
}

// xs:ID($arg as xs:anyAtomicType?) as xs:ID?
pub(crate) fn FN_XS_ID() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_ID.into()))
        ),
        xs_id_eval
    )
}

pub(crate) fn xs_id_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    convert(Types::ID, env, arguments)
}

// xs:IDREF($arg as xs:anyAtomicType?) as xs:IDREF?
pub(crate) fn FN_XS_IDREF() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_IDREF.into()))
        ),
        xs_idref_eval
    )
}

pub(crate) fn xs_idref_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    convert(Types::IDREF, env, arguments)
}

// xs:ENTITY($arg as xs:anyAtomicType?) as xs:ENTITY?
pub(crate) fn FN_XS_ENTITY() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_ENTITY.into()))
        ),
        xs_entity_eval
    )
}

pub(crate) fn xs_entity_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    convert(Types::ENTITY, env, arguments)
}

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
    convert(Types::Integer, env, arguments)
}

// xs:nonPositiveInteger($arg as xs:anyAtomicType?) as xs:nonPositiveInteger?
pub(crate) fn FN_XS_NON_POSITIVE_INTEGER() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_NON_POSITIVE_INTEGER.into()))
        ),
        xs_non_positive_integer_eval
    )
}

pub(crate) fn xs_non_positive_integer_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    convert(Types::NonPositiveInteger, env, arguments)
}

// xs:negativeInteger($arg as xs:anyAtomicType?) as xs:negativeInteger?
pub(crate) fn FN_XS_NEGATIVE_INTEGER() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_NEGATIVE_INTEGER.into()))
        ),
        xs_negative_integer_eval
    )
}

pub(crate) fn xs_negative_integer_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    convert(Types::NegativeInteger, env, arguments)
}

// xs:long($arg as xs:anyAtomicType?) as xs:long?
pub(crate) fn FN_XS_LONG() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_LONG.into()))
        ),
        xs_long_eval
    )
}

pub(crate) fn xs_long_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    convert(Types::Long, env, arguments)
}

// xs:int($arg as xs:anyAtomicType?) as xs:int?
pub(crate) fn FN_XS_INT() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_INT.into()))
        ),
        xs_int_eval
    )
}

pub(crate) fn xs_int_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    convert(Types::Int, env, arguments)
}

// xs:short($arg as xs:anyAtomicType?) as xs:short?
pub(crate) fn FN_XS_SHORT() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_SHORT.into()))
        ),
        xs_short_eval
    )
}

pub(crate) fn xs_short_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    convert(Types::Short, env, arguments)
}

// xs:byte($arg as xs:anyAtomicType?) as xs:byte?
pub(crate) fn FN_XS_BYTE() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_BYTE.into()))
        ),
        xs_byte_eval
    )
}

pub(crate) fn xs_byte_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    convert(Types::Byte, env, arguments)
}

// xs:nonNegativeInteger($arg as xs:anyAtomicType?) as xs:nonNegativeInteger?
pub(crate) fn FN_XS_NON_NEGATIVE_INTEGER() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_NON_NEGATIVE_INTEGER.into()))
        ),
        xs_non_negative_integer_eval
    )
}

pub(crate) fn xs_non_negative_integer_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    convert(Types::NonNegativeInteger, env, arguments)
}

// xs:unsignedLong($arg as xs:anyAtomicType?) as xs:unsignedLong?
pub(crate) fn FN_XS_UNSIGNED_LONG() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_UNSIGNED_LONG.into()))
        ),
        xs_unsigned_long_eval
    )
}

pub(crate) fn xs_unsigned_long_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    convert(Types::UnsignedLong, env, arguments)
}

// xs:unsignedInt($arg as xs:anyAtomicType?) as xs:unsignedInt?
pub(crate) fn FN_XS_UNSIGNED_INT() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_UNSIGNED_INT.into()))
        ),
        xs_unsigned_int_eval
    )
}

pub(crate) fn xs_unsigned_int_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    convert(Types::UnsignedInt, env, arguments)
}

// xs:unsignedShort($arg as xs:anyAtomicType?) as xs:unsignedShort?
pub(crate) fn FN_XS_UNSIGNED_SHORT() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_UNSIGNED_SHORT.into()))
        ),
        xs_unsigned_short_eval
    )
}

pub(crate) fn xs_unsigned_short_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    convert(Types::UnsignedShort, env, arguments)
}

// xs:unsignedByte($arg as xs:anyAtomicType?) as xs:unsignedByte?
pub(crate) fn FN_XS_UNSIGNED_BYTE() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_UNSIGNED_BYTE.into()))
        ),
        xs_unsigned_byte_eval
    )
}

pub(crate) fn xs_unsigned_byte_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    convert(Types::UnsignedByte, env, arguments)
}

// xs:positiveInteger($arg as xs:anyAtomicType?) as xs:positiveInteger?
pub(crate) fn FN_XS_POSITIVE_INTEGER() -> FUNCTION {
    (
        (
            [SequenceType::zero_or_one(ItemType::AnyAtomicType)].to_vec(),
            SequenceType::zero_or_one(ItemType::AtomicOrUnionType(XS_POSITIVE_INTEGER.into()))
        ),
        xs_positive_integer_eval
    )
}

pub(crate) fn xs_positive_integer_eval(env: Box<Environment>, arguments: Vec<Object>, _context: &DynamicContext) -> EvalResult {
    convert(Types::PositiveInteger, env, arguments)
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
    convert(Types::YearMonthDuration, env, arguments)
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
    convert(Types::DayTimeDuration, env, arguments)
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
    convert(Types::Untyped, env, arguments)
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
    convert(Types::DateTimeStamp, env, arguments)
}

// TODO xs:NMTOKENS($arg as xs:anyAtomicType?) as xs:NMTOKEN*
// TODO xs:ENTITIES($arg as xs:anyAtomicType?) as xs:ENTITY*
// TODO xs:IDREFS($arg as xs:anyAtomicType?) as xs:IDREF*

// TODO xs:numeric($arg as xs:anyAtomicType?) as xs:numeric?

// TODO xs:error($arg as xs:anyAtomicType?) as xs:error?

// TODO eg:hatSize($arg as xs:anyAtomicType?) as my:hatSize?




