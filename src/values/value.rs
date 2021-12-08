use std::cmp::Ordering;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Write, Debug, Formatter};
use base64::DecodeError;
use crate::values::{QName, QNameResolved};
use crate::fns::Param;
use crate::parser::op::{Representation};
use crate::parser::errors::ErrorCode;
use ordered_float::OrderedFloat;
use bigdecimal::{BigDecimal, FromPrimitive, ToPrimitive, Zero};
use bigdecimal::num_traits::real::Real;
use crate::eval::helpers::sort_and_dedup;
use crate::eval::expression::Expression;
use chrono::{Date, Datelike, DateTime, FixedOffset, Local, Offset, TimeZone};
use hex::FromHexError;
use nom::bytes::complete::tag;
use nom::character::complete::{alpha1, alphanumeric1};
use nom::error::Error;
use nom::IResult;
use nom::multi::{many0, many_m_n};
use nom::sequence::{preceded, tuple};
use crate::eval::{Environment, ErrorInfo};
use crate::eval::comparison::ValueOrdering;
use crate::eval::sequence_type::SequenceType;
use crate::parser::parse_duration::*;
use crate::parser::parse_names::{parse_name, parse_ncname, parse_nmtoken, parse_qname};
use crate::serialization::object_to_string;
use crate::serialization::to_string::*;
use crate::tree::Reference;
use crate::values::time::Time;

// xs:untypedAtomic
// xs:dateTime
// 	xs:dateTimeStamp
// xs:date
// xs:time
// xs:duration
// 	xs:yearMonthDuration
// 	xs:dayTimeDuration
// xs:float
// xs:double
// xs:decimal
// 	xs:integer
// 		xs:nonPositiveInteger
// 			xs:negativeInteger
// 		xs:long
// 			xs:int
// 				xs:short
// 					xs:byte
// 		xs:nonNegativeInteger
// 			xs:unsignedLong
// 				xs:unsignedInt
// 					xs:unsignedShort
// 						xs:unsignedByte
// 			xs:positiveInteger
// xs:gYearMonth
// xs:gYear
// xs:gMonthDay
// xs:gDay
// xs:gMonth
// xs:string
// 	xs:normalizedString
// 		xs:token
// 			xs:language
// 			xs:NMTOKEN
// 			xs:Name
// 				xs:NCName
// 					xs:ID
// 					xs:IDREF
// 					xs:ENTITY
// xs:boolean
// xs:base64Binary
// xs:hexBinary
// xs:anyURI
// xs:QName
// xs:NOTATION

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd)]
pub enum Types {
    Untyped = 0,
    String = 1,
    NormalizedString = 2,

    Boolean = 10,

    AnyURI = 20,

    Numeric = 100,
    Integer = 120,
    Decimal = 140,
    Float   = 150,
    Double  = 160,

    Byte = 111,
    Short = 112,
    Int = 113,
    Long = 114,

    UnsignedByte = 115,
    UnsignedShort = 116,
    UnsignedInt = 117,
    UnsignedLong = 118,

    PositiveInteger = 121,
    NonNegativeInteger = 122,
    NonPositiveInteger = 123,
    NegativeInteger = 124,

    DateTime = 201,
    DateTimeStamp = 202,

    Date = 203,
    Time = 204,

    Duration = 301,
    YearMonthDuration = 302,
    DayTimeDuration = 303,

    GYearMonth = 401,
    GYear = 402,
    GMonthDay = 403,
    GDay = 404,
    GMonth = 405,

    Name,
    NCName,
    QName,

    Token,
    Language,
    NMTOKEN,

    ID,
    IDREF,
    ENTITY,

    Base64Binary,
    HexBinary,

    NOTATION,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone, Hash)]
pub enum Type {
    Untyped(String),

    // TODO CharRef { representation: Representation, reference: u32 }, ?
    String(String),
    NormalizedString(String),

    AnyURI(String),

    UnsignedByte(u8),
    UnsignedShort(u16),
    UnsignedInt(u32),
    UnsignedLong(u64),

    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),

    PositiveInteger(i128),
    NonNegativeInteger(i128),
    NonPositiveInteger(i128),
    NegativeInteger(i128),

    Integer(i128),
    Decimal(BigDecimal),
    Float(OrderedFloat<f32>),
    Double(OrderedFloat<f64>),

    DateTimeStamp(),
    DateTime { dt: DateTime<FixedOffset>, offset: bool },
    Date { date: Date<FixedOffset>, offset: bool },
    Time { time: Time<FixedOffset>, offset: bool },

    Duration { positive: bool, years: u32, months: u32, days: u32, hours: u32, minutes: u32, seconds: u32, microseconds: u32 },
    YearMonthDuration  { positive: bool, years: u32, months: u32 },
    DayTimeDuration { positive: bool, days: u32, hours: u32, minutes: u32, seconds: u32, microseconds: u32 },

    GYearMonth { year: i32, month: u32, tz_m: Option<i32> },
    GYear { year: i32, tz_m: Option<i32>},
    GMonthDay { month: u32, day: u32, tz_m: Option<i32> },
    GDay { day: u32, tz_m: Option<i32> },
    GMonth { month: u32, tz_m: Option<i32> },

    Token(String),
    Language(String),
    NMTOKEN(String),

    Name(String),
    NCName(String),
    QName { url: Option<String>, prefix: Option<String>, local_part: String },

    ID(String),
    IDREF(String),
    ENTITY(String),

    Boolean(bool),

    Base64Binary(Vec<u8>),
    HexBinary(Vec<u8>),

    NOTATION(),
}

impl Type {

    pub(crate) fn date_time_now() -> Type {
        let now = Local::now();
        let dt = DateTime::from_utc(now.naive_utc(), TimeZone::from_offset(now.offset()));

        Type::DateTime { dt, offset: true }
    }

    pub(crate) fn date_now() -> Type {
        let now = Local::now();
        let date = Date::from_utc(now.date().naive_utc(), TimeZone::from_offset(now.offset()));

        Type::Date { date, offset: true }
    }

    pub(crate) fn time_now() -> Type {
        Type::Time { time: Time::now(), offset: true }
    }

    pub(crate) fn to_type(&self) -> Types {
        match self {
            Type::Untyped(_) => Types::Untyped,
            Type::String(_) => Types::String,
            Type::NormalizedString(_) => Types::NormalizedString,

            Type::AnyURI(_) => Types::AnyURI,

            Type::UnsignedByte(_) => Types::UnsignedByte,
            Type::UnsignedShort(_) => Types::UnsignedShort,
            Type::UnsignedInt(_) => Types::UnsignedInt,
            Type::UnsignedLong(_) => Types::UnsignedLong,

            Type::Byte(_) => Types::Byte,
            Type::Short(_) => Types::Short,
            Type::Int(_) => Types::Int,
            Type::Long(_) => Types::Long,

            Type::PositiveInteger(_) => Types::PositiveInteger,
            Type::NonPositiveInteger(_) => Types::NonPositiveInteger,

            Type::NegativeInteger(_) => Types::NegativeInteger,
            Type::NonNegativeInteger(_) => Types::NonNegativeInteger,

            Type::Integer(_) => Types::Integer,
            Type::Decimal(_) => Types::Decimal,
            Type::Float(_) => Types::Float,
            Type::Double(_) => Types::Double,

            Type::DateTime { .. } => Types::DateTime,
            Type::DateTimeStamp() => Types::DateTimeStamp,
            Type::Date { .. } => Types::Date,
            Type::Time { .. } => Types::Time,
            Type::Duration { .. } => Types::Duration,
            Type::YearMonthDuration { .. } => Types::YearMonthDuration,
            Type::DayTimeDuration { .. } => Types::DayTimeDuration,

            Type::GYearMonth { .. } => Types::GYearMonth,
            Type::GYear { .. } => Types::GYear,
            Type::GMonthDay { .. } => Types::GMonthDay,
            Type::GDay { .. } => Types::GDay,
            Type::GMonth { .. } => Types::GMonth,

            Type::Token(_) => Types::Token,
            Type::Language(_) => Types::Language,
            Type::NMTOKEN(_) => Types::NMTOKEN,

            Type::Name(_) => Types::Name,
            Type::NCName(_) => Types::NCName,
            Type::QName { .. } => Types::QName,

            Type::ID(_) => Types::ID,
            Type::IDREF(_) => Types::IDREF,
            Type::ENTITY(_) => Types::ENTITY,

            Type::Boolean(_) => Types::Boolean,
            Type::Base64Binary(_) => Types::Base64Binary,
            Type::HexBinary(_) => Types::HexBinary,

            Type::NOTATION() => Types::NOTATION,
        }
    }

    pub(crate) fn convert(&self, to: Types) -> Result<Type, ErrorInfo> {
        match self {
            Type::AnyURI(str) => {
                match to {
                    Types::Untyped => Ok(Type::Untyped(str.clone())),
                    Types::String => Ok(Type::String(str.clone())),
                    Types::AnyURI => Ok(Type::AnyURI(str.clone())),
                    _ => Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            },
            Type::Untyped(str) |
            Type::String(str) |
            Type::NormalizedString(str) => {
                match to {
                    Types::Untyped => Ok(Type::Untyped(str.clone())),
                    Types::String => Ok(Type::String(str.clone())),
                    Types::NormalizedString => {
                        let str = normalizing_string(str);
                        Ok(Type::NormalizedString(str))
                    },
                    Types::Language => {
                        let lang = string_to_language(str)?;
                        Ok(Type::Language(lang))
                    }
                    Types::AnyURI => Ok(Type::AnyURI(str.clone())),
                    Types::Name => {
                        let name = string_to_name(str)?;
                        Ok(Type::Name(name))
                    },
                    Types::NCName => {
                        let name = string_to_ncname(str)?;
                        Ok(Type::NCName(name))
                    },
                    Types::NMTOKEN => {
                        let name = string_to_nmtoken(str)?;
                        Ok(Type::NMTOKEN(name))
                    },
                    Types::QName => {
                        match crate::parser::parse_names::parse_qname(str) {
                            Ok((input, qname)) => {
                                if input.is_empty() {
                                    Ok(Type::QName { url: qname.url, prefix: qname.prefix, local_part: qname.local_part })
                                } else {
                                    Err((ErrorCode::FORG0001, format!("The string {:?} cannot be cast to a QName", str)))
                                }
                            },
                            Err(_) => Err((ErrorCode::FORG0001, format!("The string {:?} cannot be cast to a QName", str)))
                        }
                    },
                    Types::Token => {
                        let str = string_to_token(str);
                        Ok(Type::Token(str))
                    }
                    Types::Boolean => {
                        if str == "false" || str == "0" {
                            Ok(Type::Boolean(false))
                        } else if str == "true" || str == "1" {
                            Ok(Type::Boolean(true))
                        } else {
                            Err((ErrorCode::FORG0001, format!("The string {:?} cannot be cast to a boolean", str)))
                        }
                    }

                    Types::Byte => crate::values::string_to::byte(str),
                    Types::Short => crate::values::string_to::short(str),
                    Types::Int => crate::values::string_to::int(str),
                    Types::Long => crate::values::string_to::long(str),

                    Types::UnsignedByte => crate::values::string_to::unsigned_byte(str),
                    Types::UnsignedShort => crate::values::string_to::unsigned_short(str),
                    Types::UnsignedInt => crate::values::string_to::unsigned_int(str),
                    Types::UnsignedLong => crate::values::string_to::unsigned_long(str),

                    Types::PositiveInteger => crate::values::string_to::positive_integer(str),
                    Types::NonNegativeInteger => crate::values::string_to::non_negative_integer(str),
                    Types::NonPositiveInteger => crate::values::string_to::non_positive_integer(str),
                    Types::NegativeInteger => crate::values::string_to::negative_integer(str),

                    Types::Integer => crate::values::string_to::integer(str),
                    Types::Decimal => crate::values::string_to::decimal(str),
                    Types::Float => crate::values::string_to::float(str, false),
                    Types::Double => crate::values::string_to::double(str, false),
                    Types::DateTime => {
                        match parse_date_time_complete(str) {
                            Ok((_, t)) => Ok(t),
                            Err(_) => Err((ErrorCode::FORG0001, format!("can't convert to DateTime {:?}", str)))
                        }
                    }
                    Types::DateTimeStamp => {
                        todo!()
                        // match parse_date_time_stamp_complete(str) {
                        //     Ok((_, t)) => Ok(t),
                        //     Err(_) => Err((ErrorCode::FORG0001, format!("can't convert to DateTime {:?}", str)))
                        // }
                    }
                    Types::Date => {
                        match parse_date_complete(str) {
                            Ok((_, t)) => Ok(t),
                            Err(_) => Err((ErrorCode::FORG0001, format!("can't convert to Date {:?}", str)))
                        }
                    }
                    Types::Time => {
                        match parse_time_complete(str) {
                            Ok((_, t)) => Ok(t),
                            Err(_) => Err((ErrorCode::FORG0001, format!("can't convert to Time {:?}", str)))
                        }
                    }
                    Types::Duration => {
                        match parse_duration_complete(str) {
                            Ok((_, t)) => Ok(t),
                            Err(_) => Err((ErrorCode::FORG0001, format!("can't convert to Duration {:?}", str)))
                        }
                    }
                    Types::DayTimeDuration => {
                        match parse_day_time_duration_complete(str) {
                            Ok((_, t)) => Ok(t),
                            Err(_) => Err((ErrorCode::FORG0001, format!("can't convert to DayTimeDuration {:?}", str)))
                        }
                    }
                    Types::YearMonthDuration => {
                        match parse_year_month_duration_complete(str) {
                            Ok((_, t)) => Ok(t),
                            Err(_) => Err((ErrorCode::FORG0001, format!("can't convert to YearMonthDuration {:?}", str)))
                        }
                    }
                    Types::GYearMonth => {
                        match string_to_year_month(str) {
                            Ok(t) => Ok(t),
                            Err(msg) => Err((ErrorCode::FORG0001, msg))
                        }
                    }
                    Types::GYear => {
                        match string_to_year(str) {
                            Ok(t) => Ok(t),
                            Err(msg) => Err((ErrorCode::FORG0001, msg))
                        }
                    }
                    Types::GMonthDay => {
                        match string_to_month_day(str) {
                            Ok(t) => Ok(t),
                            Err(msg) => Err((ErrorCode::FORG0001, msg))
                        }
                    }
                    Types::GDay => {
                        match string_to_day(str) {
                            Ok(t) => Ok(t),
                            Err(msg) => Err((ErrorCode::FORG0001, msg))
                        }
                    }
                    Types::GMonth => {
                        match string_to_month(str) {
                            Ok(t) => Ok(t),
                            Err(msg) => Err((ErrorCode::FORG0001, msg))
                        }
                    }
                    Types::Base64Binary => {
                        match string_to_binary_base64(str) {
                            Ok(binary) => Ok(Type::Base64Binary(binary)),
                            Err(code) => Err((code, String::from("TODO")))
                        }
                    }
                    Types::HexBinary => {
                        match string_to_binary_hex(str) {
                            Ok(binary) => Ok(Type::HexBinary(binary)),
                            Err(code) => Err((code, String::from("TODO")))
                        }
                    }
                    Types::ID => {
                        if str.is_empty() {
                            Err((ErrorCode::FORG0001, String::from("TODO")))
                        } else {
                            Ok(Type::ID(str.clone()))
                        }
                    },
                    Types::IDREF => {
                        if str.is_empty() {
                            Err((ErrorCode::FORG0001, String::from("TODO")))
                        } else {
                            Ok(Type::IDREF(str.clone()))
                        }
                    },
                    Types::ENTITY => {
                        if str.is_empty() {
                            Err((ErrorCode::FORG0001, String::from("TODO")))
                        } else {
                            Ok(Type::ENTITY(str.clone()))
                        }
                    },
                    _ => panic!("{:?} from {:?}", to, self) // Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            }

            Type::Byte(number) => {
                match to {
                    Types::Untyped => {
                        let data = number.to_string();
                        Ok(Type::Untyped(data))
                    }
                    Types::String => {
                        let data = number.to_string();
                        Ok(Type::String(data))
                    }
                    Types::Language => Err((ErrorCode::FORG0001, String::from("TODO"))),
                    Types::Boolean => Ok(Type::Boolean(!number.is_zero())),

                    Types::UnsignedLong => {
                        if let Some(num) = number.to_u64() {
                            Ok(Type::UnsignedLong(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedInt => {
                        if let Some(num) = number.to_u32() {
                            Ok(Type::UnsignedInt(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedShort => {
                        if let Some(num) = number.to_u16() {
                            Ok(Type::UnsignedShort(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedByte => {
                        if let Some(num) = number.to_u8() {
                            Ok(Type::UnsignedByte(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }

                    Types::Long => {
                        if let Some(num) = number.to_i64() {
                            Ok(Type::Long(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::Int => {
                        if let Some(num) = number.to_i32() {
                            Ok(Type::Int(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::Short => {
                        if let Some(num) = number.to_i16() {
                            Ok(Type::Short(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::Byte => {
                        if let Some(num) = number.to_i8() {
                            Ok(Type::Byte(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }

                    Types::PositiveInteger => {
                        if let Some(num) = number.to_i128() {
                            if num > 0 {
                                Ok(Type::PositiveInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NonNegativeInteger => {
                        if let Some(num) = number.to_i128() {
                            if num >= 0 {
                                Ok(Type::NonNegativeInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NonPositiveInteger => {
                        if let Some(num) = number.to_i128() {
                            if num <= 0 {
                                Ok(Type::NonPositiveInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NegativeInteger => {
                        if let Some(num) = number.to_i128() {
                            if num < 0 {
                                Ok(Type::NegativeInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },

                    Types::Integer => {
                        if let Some(num) = number.to_i128() {
                            Ok(Type::Integer(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::Decimal => {
                        match BigDecimal::from_i8(*number) {
                            Some(number) => Ok((Type::Decimal(number))),
                            None => Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    },
                    Types::Float => {
                        match number.to_f32() {
                            Some(number) => {
                                let number = OrderedFloat::from(number);
                                Ok(Type::Float(number))
                            },
                            None => Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    },
                    Types::Double => {
                        match number.to_f64() {
                            Some(number) => {
                                let number = OrderedFloat::from(number);
                                Ok(Type::Double(number))
                            },
                            None => Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    },
                    _ => Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            }
            Type::Short(number) => {
                match to {
                    Types::Untyped => {
                        let data = number.to_string();
                        Ok(Type::Untyped(data))
                    }
                    Types::String => {
                        let data = number.to_string();
                        Ok(Type::String(data))
                    }
                    Types::Language => Err((ErrorCode::FORG0001, String::from("TODO"))),
                    Types::Boolean => Ok(Type::Boolean(!number.is_zero())),

                    Types::UnsignedLong => {
                        if let Some(num) = number.to_u64() {
                            Ok(Type::UnsignedLong(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedInt => {
                        if let Some(num) = number.to_u32() {
                            Ok(Type::UnsignedInt(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedShort => {
                        if let Some(num) = number.to_u16() {
                            Ok(Type::UnsignedShort(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedByte => {
                        if let Some(num) = number.to_u8() {
                            Ok(Type::UnsignedByte(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }

                    Types::Long => {
                        if let Some(num) = number.to_i64() {
                            Ok(Type::Long(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::Int => {
                        if let Some(num) = number.to_i32() {
                            Ok(Type::Int(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::Short => {
                        if let Some(num) = number.to_i16() {
                            Ok(Type::Short(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::Byte => {
                        if let Some(num) = number.to_i8() {
                            Ok(Type::Byte(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }

                    Types::PositiveInteger => {
                        if let Some(num) = number.to_i128() {
                            if num > 0 {
                                Ok(Type::PositiveInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NonNegativeInteger => {
                        if let Some(num) = number.to_i128() {
                            if num >= 0 {
                                Ok(Type::NonNegativeInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NonPositiveInteger => {
                        if let Some(num) = number.to_i128() {
                            if num <= 0 {
                                Ok(Type::NonPositiveInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NegativeInteger => {
                        if let Some(num) = number.to_i128() {
                            if num < 0 {
                                Ok(Type::NegativeInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },

                    Types::Integer => {
                        if let Some(num) = number.to_i128() {
                            Ok(Type::Integer(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::Decimal => {
                        match BigDecimal::from_i16(*number) {
                            Some(number) => Ok((Type::Decimal(number))),
                            None => Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    },
                    Types::Float => {
                        match number.to_f32() {
                            Some(number) => {
                                let number = OrderedFloat::from(number);
                                Ok(Type::Float(number))
                            },
                            None => Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    },
                    Types::Double => {
                        match number.to_f64() {
                            Some(number) => {
                                let number = OrderedFloat::from(number);
                                Ok(Type::Double(number))
                            },
                            None => Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    },
                    _ => Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            }
            Type::Int(number) => {
                match to {
                    Types::Untyped => {
                        let data = number.to_string();
                        Ok(Type::Untyped(data))
                    }
                    Types::String => {
                        let data = number.to_string();
                        Ok(Type::String(data))
                    }
                    Types::Language => Err((ErrorCode::FORG0001, String::from("TODO"))),
                    Types::Boolean => Ok(Type::Boolean(!number.is_zero())),

                    Types::UnsignedLong => {
                        if let Some(num) = number.to_u64() {
                            Ok(Type::UnsignedLong(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedInt => {
                        if let Some(num) = number.to_u32() {
                            Ok(Type::UnsignedInt(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedShort => {
                        if let Some(num) = number.to_u16() {
                            Ok(Type::UnsignedShort(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedByte => {
                        if let Some(num) = number.to_u8() {
                            Ok(Type::UnsignedByte(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }

                    Types::Long => {
                        if let Some(num) = number.to_i64() {
                            Ok(Type::Long(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::Int => {
                        if let Some(num) = number.to_i32() {
                            Ok(Type::Int(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::Short => {
                        if let Some(num) = number.to_i16() {
                            Ok(Type::Short(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::Byte => {
                        if let Some(num) = number.to_i8() {
                            Ok(Type::Byte(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }

                    Types::PositiveInteger => {
                        if let Some(num) = number.to_i128() {
                            if num > 0 {
                                Ok(Type::PositiveInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NonNegativeInteger => {
                        if let Some(num) = number.to_i128() {
                            if num >= 0 {
                                Ok(Type::NonNegativeInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NonPositiveInteger => {
                        if let Some(num) = number.to_i128() {
                            if num <= 0 {
                                Ok(Type::NonPositiveInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NegativeInteger => {
                        if let Some(num) = number.to_i128() {
                            if num < 0 {
                                Ok(Type::NegativeInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },

                    Types::Integer => {
                        if let Some(num) = number.to_i128() {
                            Ok(Type::Integer(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::Decimal => {
                        match BigDecimal::from_i32(*number) {
                            Some(number) => Ok((Type::Decimal(number))),
                            None => Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    },
                    Types::Float => {
                        match number.to_f32() {
                            Some(number) => {
                                let number = OrderedFloat::from(number);
                                Ok(Type::Float(number))
                            },
                            None => Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    },
                    Types::Double => {
                        match number.to_f64() {
                            Some(number) => {
                                let number = OrderedFloat::from(number);
                                Ok(Type::Double(number))
                            },
                            None => Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    },
                    _ => Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            }
            Type::Long(number) => {
                match to {
                    Types::Untyped => {
                        let data = number.to_string();
                        Ok(Type::Untyped(data))
                    }
                    Types::String => {
                        let data = number.to_string();
                        Ok(Type::String(data))
                    }
                    Types::Language => Err((ErrorCode::FORG0001, String::from("TODO"))),
                    Types::Boolean => Ok(Type::Boolean(!number.is_zero())),

                    Types::UnsignedLong => {
                        if let Some(num) = number.to_u64() {
                            Ok(Type::UnsignedLong(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedInt => {
                        if let Some(num) = number.to_u32() {
                            Ok(Type::UnsignedInt(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedShort => {
                        if let Some(num) = number.to_u16() {
                            Ok(Type::UnsignedShort(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedByte => {
                        if let Some(num) = number.to_u8() {
                            Ok(Type::UnsignedByte(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }

                    Types::Long => {
                        if let Some(num) = number.to_i64() {
                            Ok(Type::Long(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::Int => {
                        if let Some(num) = number.to_i32() {
                            Ok(Type::Int(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::Short => {
                        if let Some(num) = number.to_i16() {
                            Ok(Type::Short(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::Byte => {
                        if let Some(num) = number.to_i8() {
                            Ok(Type::Byte(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }

                    Types::PositiveInteger => {
                        if let Some(num) = number.to_i128() {
                            if num > 0 {
                                Ok(Type::PositiveInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NonNegativeInteger => {
                        if let Some(num) = number.to_i128() {
                            if num >= 0 {
                                Ok(Type::NonNegativeInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NonPositiveInteger => {
                        if let Some(num) = number.to_i128() {
                            if num <= 0 {
                                Ok(Type::NonPositiveInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NegativeInteger => {
                        if let Some(num) = number.to_i128() {
                            if num < 0 {
                                Ok(Type::NegativeInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },

                    Types::Integer => {
                        if let Some(num) = number.to_i128() {
                            Ok(Type::Integer(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::Decimal => {
                        match BigDecimal::from_i64(*number) {
                            Some(number) => Ok((Type::Decimal(number))),
                            None => Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    },
                    Types::Float => {
                        match number.to_f32() {
                            Some(number) => {
                                let number = OrderedFloat::from(number);
                                Ok(Type::Float(number))
                            },
                            None => Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    },
                    Types::Double => {
                        match number.to_f64() {
                            Some(number) => {
                                let number = OrderedFloat::from(number);
                                Ok(Type::Double(number))
                            },
                            None => Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    },
                    _ => Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            }

            Type::UnsignedByte(number) => {
                match to {
                    Types::Untyped => {
                        let data = number.to_string();
                        Ok(Type::Untyped(data))
                    }
                    Types::String => {
                        let data = number.to_string();
                        Ok(Type::String(data))
                    }
                    Types::Language => Err((ErrorCode::FORG0001, String::from("TODO"))),
                    Types::Boolean => Ok(Type::Boolean(!number.is_zero())),

                    Types::UnsignedLong => {
                        if let Some(num) = number.to_u64() {
                            Ok(Type::UnsignedLong(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedInt => {
                        if let Some(num) = number.to_u32() {
                            Ok(Type::UnsignedInt(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedShort => {
                        if let Some(num) = number.to_u16() {
                            Ok(Type::UnsignedShort(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedByte => {
                        if let Some(num) = number.to_u8() {
                            Ok(Type::UnsignedByte(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }

                    Types::Long => {
                        if let Some(num) = number.to_i64() {
                            Ok(Type::Long(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::Int => {
                        if let Some(num) = number.to_i32() {
                            Ok(Type::Int(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::Short => {
                        if let Some(num) = number.to_i16() {
                            Ok(Type::Short(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::Byte => {
                        if let Some(num) = number.to_i8() {
                            Ok(Type::Byte(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }

                    Types::PositiveInteger => {
                        if let Some(num) = number.to_i128() {
                            if num > 0 {
                                Ok(Type::PositiveInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NonNegativeInteger => {
                        if let Some(num) = number.to_i128() {
                            if num >= 0 {
                                Ok(Type::NonNegativeInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NonPositiveInteger => {
                        if let Some(num) = number.to_i128() {
                            if num <= 0 {
                                Ok(Type::NonPositiveInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NegativeInteger => {
                        if let Some(num) = number.to_i128() {
                            if num < 0 {
                                Ok(Type::NegativeInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },

                    Types::Integer => {
                        if let Some(num) = number.to_i128() {
                            Ok(Type::Integer(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::Decimal => {
                        match BigDecimal::from_u8(*number) {
                            Some(number) => Ok((Type::Decimal(number))),
                            None => Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    },
                    Types::Float => {
                        match number.to_f32() {
                            Some(number) => {
                                let number = OrderedFloat::from(number);
                                Ok(Type::Float(number))
                            },
                            None => Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    },
                    Types::Double => {
                        match number.to_f64() {
                            Some(number) => {
                                let number = OrderedFloat::from(number);
                                Ok(Type::Double(number))
                            },
                            None => Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    },
                    _ => Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            }
            Type::UnsignedShort(number) => {
                match to {
                    Types::Untyped => {
                        let data = number.to_string();
                        Ok(Type::Untyped(data))
                    }
                    Types::String => {
                        let data = number.to_string();
                        Ok(Type::String(data))
                    }
                    Types::Language => Err((ErrorCode::FORG0001, String::from("TODO"))),
                    Types::Boolean => Ok(Type::Boolean(!number.is_zero())),

                    Types::UnsignedLong => {
                        if let Some(num) = number.to_u64() {
                            Ok(Type::UnsignedLong(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedInt => {
                        if let Some(num) = number.to_u32() {
                            Ok(Type::UnsignedInt(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedShort => {
                        if let Some(num) = number.to_u16() {
                            Ok(Type::UnsignedShort(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedByte => {
                        if let Some(num) = number.to_u8() {
                            Ok(Type::UnsignedByte(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }

                    Types::Long => {
                        if let Some(num) = number.to_i64() {
                            Ok(Type::Long(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::Int => {
                        if let Some(num) = number.to_i32() {
                            Ok(Type::Int(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::Short => {
                        if let Some(num) = number.to_i16() {
                            Ok(Type::Short(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::Byte => {
                        if let Some(num) = number.to_i8() {
                            Ok(Type::Byte(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }

                    Types::PositiveInteger => {
                        if let Some(num) = number.to_i128() {
                            if num > 0 {
                                Ok(Type::PositiveInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NonNegativeInteger => {
                        if let Some(num) = number.to_i128() {
                            if num >= 0 {
                                Ok(Type::NonNegativeInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NonPositiveInteger => {
                        if let Some(num) = number.to_i128() {
                            if num <= 0 {
                                Ok(Type::NonPositiveInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NegativeInteger => {
                        if let Some(num) = number.to_i128() {
                            if num < 0 {
                                Ok(Type::NegativeInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },

                    Types::Integer => {
                        if let Some(num) = number.to_i128() {
                            Ok(Type::Integer(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::Decimal => {
                        match BigDecimal::from_u16(*number) {
                            Some(number) => Ok((Type::Decimal(number))),
                            None => Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    },
                    Types::Float => {
                        match number.to_f32() {
                            Some(number) => {
                                let number = OrderedFloat::from(number);
                                Ok(Type::Float(number))
                            },
                            None => Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    },
                    Types::Double => {
                        match number.to_f64() {
                            Some(number) => {
                                let number = OrderedFloat::from(number);
                                Ok(Type::Double(number))
                            },
                            None => Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    },
                    _ => Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            }
            Type::UnsignedInt(number) => {
                match to {
                    Types::Untyped => {
                        let data = number.to_string();
                        Ok(Type::Untyped(data))
                    }
                    Types::String => {
                        let data = number.to_string();
                        Ok(Type::String(data))
                    }
                    Types::Language => Err((ErrorCode::FORG0001, String::from("TODO"))),
                    Types::Boolean => Ok(Type::Boolean(!number.is_zero())),

                    Types::UnsignedLong => {
                        if let Some(num) = number.to_u64() {
                            Ok(Type::UnsignedLong(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedInt => {
                        if let Some(num) = number.to_u32() {
                            Ok(Type::UnsignedInt(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedShort => {
                        if let Some(num) = number.to_u16() {
                            Ok(Type::UnsignedShort(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedByte => {
                        if let Some(num) = number.to_u8() {
                            Ok(Type::UnsignedByte(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }

                    Types::Long => {
                        if let Some(num) = number.to_i64() {
                            Ok(Type::Long(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::Int => {
                        if let Some(num) = number.to_i32() {
                            Ok(Type::Int(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::Short => {
                        if let Some(num) = number.to_i16() {
                            Ok(Type::Short(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::Byte => {
                        if let Some(num) = number.to_i8() {
                            Ok(Type::Byte(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }

                    Types::PositiveInteger => {
                        if let Some(num) = number.to_i128() {
                            if num > 0 {
                                Ok(Type::PositiveInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NonNegativeInteger => {
                        if let Some(num) = number.to_i128() {
                            if num >= 0 {
                                Ok(Type::NonNegativeInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NonPositiveInteger => {
                        if let Some(num) = number.to_i128() {
                            if num <= 0 {
                                Ok(Type::NonPositiveInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NegativeInteger => {
                        if let Some(num) = number.to_i128() {
                            if num < 0 {
                                Ok(Type::NegativeInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },

                    Types::Integer => {
                        if let Some(num) = number.to_i128() {
                            Ok(Type::Integer(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::Decimal => {
                        match BigDecimal::from_u32(*number) {
                            Some(number) => Ok((Type::Decimal(number))),
                            None => Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    },
                    Types::Float => {
                        match number.to_f32() {
                            Some(number) => {
                                let number = OrderedFloat::from(number);
                                Ok(Type::Float(number))
                            },
                            None => Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    },
                    Types::Double => {
                        match number.to_f64() {
                            Some(number) => {
                                let number = OrderedFloat::from(number);
                                Ok(Type::Double(number))
                            },
                            None => Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    },
                    _ => Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            }
            Type::UnsignedLong(number) => {
                match to {
                    Types::Untyped => {
                        let data = number.to_string();
                        Ok(Type::Untyped(data))
                    }
                    Types::String => {
                        let data = number.to_string();
                        Ok(Type::String(data))
                    }
                    Types::Language => Err((ErrorCode::FORG0001, String::from("TODO"))),
                    Types::Boolean => Ok(Type::Boolean(!number.is_zero())),

                    Types::UnsignedLong => {
                        if let Some(num) = number.to_u64() {
                            Ok(Type::UnsignedLong(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedInt => {
                        if let Some(num) = number.to_u32() {
                            Ok(Type::UnsignedInt(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedShort => {
                        if let Some(num) = number.to_u16() {
                            Ok(Type::UnsignedShort(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedByte => {
                        if let Some(num) = number.to_u8() {
                            Ok(Type::UnsignedByte(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }

                    Types::Long => {
                        if let Some(num) = number.to_i64() {
                            Ok(Type::Long(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::Int => {
                        if let Some(num) = number.to_i32() {
                            Ok(Type::Int(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::Short => {
                        if let Some(num) = number.to_i16() {
                            Ok(Type::Short(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::Byte => {
                        if let Some(num) = number.to_i8() {
                            Ok(Type::Byte(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }

                    Types::PositiveInteger => {
                        if let Some(num) = number.to_i128() {
                            if num > 0 {
                                Ok(Type::PositiveInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NonNegativeInteger => {
                        if let Some(num) = number.to_i128() {
                            if num >= 0 {
                                Ok(Type::NonNegativeInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NonPositiveInteger => {
                        if let Some(num) = number.to_i128() {
                            if num <= 0 {
                                Ok(Type::NonPositiveInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NegativeInteger => {
                        if let Some(num) = number.to_i128() {
                            if num < 0 {
                                Ok(Type::NegativeInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },

                    Types::Integer => {
                        if let Some(num) = number.to_i128() {
                            Ok(Type::Integer(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::Decimal => {
                        match BigDecimal::from_u64(*number) {
                            Some(number) => Ok((Type::Decimal(number))),
                            None => Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    },
                    Types::Float => {
                        match number.to_f32() {
                            Some(number) => {
                                let number = OrderedFloat::from(number);
                                Ok(Type::Float(number))
                            },
                            None => Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    },
                    Types::Double => {
                        match number.to_f64() {
                            Some(number) => {
                                let number = OrderedFloat::from(number);
                                Ok(Type::Double(number))
                            },
                            None => Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    },
                    _ => Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            }

            Type::PositiveInteger(number) => {
                match to {
                    Types::Untyped => {
                        let data = number.to_string();
                        Ok(Type::Untyped(data))
                    }
                    Types::String => {
                        let data = number.to_string();
                        Ok(Type::String(data))
                    }
                    Types::Language => Err((ErrorCode::FORG0001, String::from("TODO"))),
                    Types::Boolean => Ok(Type::Boolean(!number.is_zero())),

                    Types::UnsignedLong => {
                        if let Some(num) = number.to_u64() {
                            Ok(Type::UnsignedLong(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedInt => {
                        if let Some(num) = number.to_u32() {
                            Ok(Type::UnsignedInt(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedShort => {
                        if let Some(num) = number.to_u16() {
                            Ok(Type::UnsignedShort(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedByte => {
                        if let Some(num) = number.to_u8() {
                            Ok(Type::UnsignedByte(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }

                    Types::Long => {
                        if let Some(num) = number.to_i64() {
                            Ok(Type::Long(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::Int => {
                        if let Some(num) = number.to_i32() {
                            Ok(Type::Int(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::Short => {
                        if let Some(num) = number.to_i16() {
                            Ok(Type::Short(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::Byte => {
                        if let Some(num) = number.to_i8() {
                            Ok(Type::Byte(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }

                    Types::PositiveInteger => {
                        if let Some(num) = number.to_i128() {
                            if num > 0 {
                                Ok(Type::PositiveInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NonNegativeInteger => {
                        if let Some(num) = number.to_i128() {
                            if num >= 0 {
                                Ok(Type::NonNegativeInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NonPositiveInteger => {
                        if let Some(num) = number.to_i128() {
                            if num <= 0 {
                                Ok(Type::NonPositiveInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NegativeInteger => {
                        if let Some(num) = number.to_i128() {
                            if num < 0 {
                                Ok(Type::NegativeInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },

                    Types::Integer => {
                        if let Some(num) = number.to_i128() {
                            Ok(Type::Integer(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::Decimal => {
                        match BigDecimal::from_i128(*number) {
                            Some(number) => Ok((Type::Decimal(number))),
                            None => Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    },
                    Types::Float => {
                        match number.to_f32() {
                            Some(number) => {
                                let number = OrderedFloat::from(number);
                                Ok(Type::Float(number))
                            },
                            None => Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    },
                    Types::Double => {
                        match number.to_f64() {
                            Some(number) => {
                                let number = OrderedFloat::from(number);
                                Ok(Type::Double(number))
                            },
                            None => Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    },
                    _ => Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            }
            Type::NonNegativeInteger(number) => {
                match to {
                    Types::Untyped => {
                        let data = number.to_string();
                        Ok(Type::Untyped(data))
                    }
                    Types::String => {
                        let data = number.to_string();
                        Ok(Type::String(data))
                    }
                    Types::Language => Err((ErrorCode::FORG0001, String::from("TODO"))),
                    Types::Boolean => Ok(Type::Boolean(!number.is_zero())),

                    Types::UnsignedLong => {
                        if let Some(num) = number.to_u64() {
                            Ok(Type::UnsignedLong(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedInt => {
                        if let Some(num) = number.to_u32() {
                            Ok(Type::UnsignedInt(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedShort => {
                        if let Some(num) = number.to_u16() {
                            Ok(Type::UnsignedShort(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedByte => {
                        if let Some(num) = number.to_u8() {
                            Ok(Type::UnsignedByte(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }

                    Types::Long => {
                        if let Some(num) = number.to_i64() {
                            Ok(Type::Long(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::Int => {
                        if let Some(num) = number.to_i32() {
                            Ok(Type::Int(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::Short => {
                        if let Some(num) = number.to_i16() {
                            Ok(Type::Short(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::Byte => {
                        if let Some(num) = number.to_i8() {
                            Ok(Type::Byte(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }

                    Types::PositiveInteger => {
                        if let Some(num) = number.to_i128() {
                            if num > 0 {
                                Ok(Type::PositiveInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NonNegativeInteger => {
                        if let Some(num) = number.to_i128() {
                            if num >= 0 {
                                Ok(Type::NonNegativeInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NonPositiveInteger => {
                        if let Some(num) = number.to_i128() {
                            if num <= 0 {
                                Ok(Type::NonPositiveInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NegativeInteger => {
                        if let Some(num) = number.to_i128() {
                            if num < 0 {
                                Ok(Type::NegativeInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },

                    Types::Integer => {
                        if let Some(num) = number.to_i128() {
                            Ok(Type::Integer(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::Decimal => {
                        match BigDecimal::from_i128(*number) {
                            Some(number) => Ok((Type::Decimal(number))),
                            None => Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    },
                    Types::Float => {
                        match number.to_f32() {
                            Some(number) => {
                                let number = OrderedFloat::from(number);
                                Ok(Type::Float(number))
                            },
                            None => Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    },
                    Types::Double => {
                        match number.to_f64() {
                            Some(number) => {
                                let number = OrderedFloat::from(number);
                                Ok(Type::Double(number))
                            },
                            None => Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    },
                    _ => Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            }
            Type::NonPositiveInteger(number) => {
                match to {
                    Types::Untyped => {
                        let data = number.to_string();
                        Ok(Type::Untyped(data))
                    }
                    Types::String => {
                        let data = number.to_string();
                        Ok(Type::String(data))
                    }
                    Types::Language => Err((ErrorCode::FORG0001, String::from("TODO"))),
                    Types::Boolean => Ok(Type::Boolean(!number.is_zero())),

                    Types::UnsignedLong => {
                        if let Some(num) = number.to_u64() {
                            Ok(Type::UnsignedLong(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedInt => {
                        if let Some(num) = number.to_u32() {
                            Ok(Type::UnsignedInt(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedShort => {
                        if let Some(num) = number.to_u16() {
                            Ok(Type::UnsignedShort(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedByte => {
                        if let Some(num) = number.to_u8() {
                            Ok(Type::UnsignedByte(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }

                    Types::Long => {
                        if let Some(num) = number.to_i64() {
                            Ok(Type::Long(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::Int => {
                        if let Some(num) = number.to_i32() {
                            Ok(Type::Int(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::Short => {
                        if let Some(num) = number.to_i16() {
                            Ok(Type::Short(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::Byte => {
                        if let Some(num) = number.to_i8() {
                            Ok(Type::Byte(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }

                    Types::PositiveInteger => {
                        if let Some(num) = number.to_i128() {
                            if num > 0 {
                                Ok(Type::PositiveInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NonNegativeInteger => {
                        if let Some(num) = number.to_i128() {
                            if num >= 0 {
                                Ok(Type::NonNegativeInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NonPositiveInteger => {
                        if let Some(num) = number.to_i128() {
                            if num <= 0 {
                                Ok(Type::NonPositiveInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NegativeInteger => {
                        if let Some(num) = number.to_i128() {
                            if num < 0 {
                                Ok(Type::NegativeInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },

                    Types::Integer => {
                        if let Some(num) = number.to_i128() {
                            Ok(Type::Integer(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::Decimal => {
                        match BigDecimal::from_i128(*number) {
                            Some(number) => Ok((Type::Decimal(number))),
                            None => Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    },
                    Types::Float => {
                        match number.to_f32() {
                            Some(number) => {
                                let number = OrderedFloat::from(number);
                                Ok(Type::Float(number))
                            },
                            None => Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    },
                    Types::Double => {
                        match number.to_f64() {
                            Some(number) => {
                                let number = OrderedFloat::from(number);
                                Ok(Type::Double(number))
                            },
                            None => Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    },
                    _ => Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            }
            Type::NegativeInteger(number) => {
                match to {
                    Types::Untyped => {
                        let data = number.to_string();
                        Ok(Type::Untyped(data))
                    }
                    Types::String => {
                        let data = number.to_string();
                        Ok(Type::String(data))
                    }
                    Types::Language => Err((ErrorCode::FORG0001, String::from("TODO"))),
                    Types::Boolean => Ok(Type::Boolean(!number.is_zero())),

                    Types::UnsignedLong => {
                        if let Some(num) = number.to_u64() {
                            Ok(Type::UnsignedLong(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedInt => {
                        if let Some(num) = number.to_u32() {
                            Ok(Type::UnsignedInt(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedShort => {
                        if let Some(num) = number.to_u16() {
                            Ok(Type::UnsignedShort(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedByte => {
                        if let Some(num) = number.to_u8() {
                            Ok(Type::UnsignedByte(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }

                    Types::Long => {
                        if let Some(num) = number.to_i64() {
                            Ok(Type::Long(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::Int => {
                        if let Some(num) = number.to_i32() {
                            Ok(Type::Int(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::Short => {
                        if let Some(num) = number.to_i16() {
                            Ok(Type::Short(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::Byte => {
                        if let Some(num) = number.to_i8() {
                            Ok(Type::Byte(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }

                    Types::PositiveInteger => {
                        if let Some(num) = number.to_i128() {
                            if num > 0 {
                                Ok(Type::PositiveInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NonNegativeInteger => {
                        if let Some(num) = number.to_i128() {
                            if num >= 0 {
                                Ok(Type::NonNegativeInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NonPositiveInteger => {
                        if let Some(num) = number.to_i128() {
                            if num <= 0 {
                                Ok(Type::NonPositiveInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NegativeInteger => {
                        if let Some(num) = number.to_i128() {
                            if num < 0 {
                                Ok(Type::NegativeInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },

                    Types::Integer => {
                        if let Some(num) = number.to_i128() {
                            Ok(Type::Integer(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::Decimal => {
                        match BigDecimal::from_i128(*number) {
                            Some(number) => Ok((Type::Decimal(number))),
                            None => Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    },
                    Types::Float => {
                        match number.to_f32() {
                            Some(number) => {
                                let number = OrderedFloat::from(number);
                                Ok(Type::Float(number))
                            },
                            None => Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    },
                    Types::Double => {
                        match number.to_f64() {
                            Some(number) => {
                                let number = OrderedFloat::from(number);
                                Ok(Type::Double(number))
                            },
                            None => Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    },
                    _ => Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            }

            Type::Integer(number) => {
                match to {
                    Types::Untyped => {
                        let data = number.to_string();
                        Ok(Type::Untyped(data))
                    }
                    Types::String => {
                        let data = number.to_string();
                        Ok(Type::String(data))
                    }
                    Types::Language => Err((ErrorCode::FORG0001, String::from("TODO"))),
                    Types::Boolean => Ok(Type::Boolean(!number.is_zero())),

                    Types::UnsignedLong => {
                        if let Some(num) = number.to_u64() {
                            Ok(Type::UnsignedLong(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedInt => {
                        if let Some(num) = number.to_u32() {
                            Ok(Type::UnsignedInt(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedShort => {
                        if let Some(num) = number.to_u16() {
                            Ok(Type::UnsignedShort(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedByte => {
                        if let Some(num) = number.to_u8() {
                            Ok(Type::UnsignedByte(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }

                    Types::Long => {
                        if let Some(num) = number.to_i64() {
                            Ok(Type::Long(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::Int => {
                        if let Some(num) = number.to_i32() {
                            Ok(Type::Int(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::Short => {
                        if let Some(num) = number.to_i16() {
                            Ok(Type::Short(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::Byte => {
                        if let Some(num) = number.to_i8() {
                            Ok(Type::Byte(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }

                    Types::PositiveInteger => {
                        if let Some(num) = number.to_i128() {
                            if num > 0 {
                                Ok(Type::PositiveInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NonNegativeInteger => {
                        if let Some(num) = number.to_i128() {
                            if num >= 0 {
                                Ok(Type::NonNegativeInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NonPositiveInteger => {
                        if let Some(num) = number.to_i128() {
                            if num <= 0 {
                                Ok(Type::NonPositiveInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NegativeInteger => {
                        if let Some(num) = number.to_i128() {
                            if num < 0 {
                                Ok(Type::NegativeInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },

                    Types::Integer => {
                        if let Some(num) = number.to_i128() {
                            Ok(Type::Integer(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::Decimal => {
                        match BigDecimal::from_i128(*number) {
                            Some(number) => Ok((Type::Decimal(number))),
                            None => Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    },
                    Types::Float => {
                        match number.to_f32() {
                            Some(number) => {
                                let number = OrderedFloat::from(number);
                                Ok(Type::Float(number))
                            },
                            None => Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    },
                    Types::Double => {
                        match number.to_f64() {
                            Some(number) => {
                                let number = OrderedFloat::from(number);
                                Ok(Type::Double(number))
                            },
                            None => Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    },
                    _ => Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            }
            Type::Decimal(number) => {
                match to {
                    Types::Untyped => {
                        let data = number.to_string();
                        Ok(Type::Untyped(data))
                    }
                    Types::String => {
                        let data = number.to_string();
                        Ok(Type::String(data))
                    }
                    Types::Language => Err((ErrorCode::FORG0001, String::from("TODO"))),
                    Types::Boolean => Ok(Type::Boolean(!number.is_zero())),

                    Types::UnsignedLong => {
                        if let Some(num) = number.to_u64() {
                            Ok(Type::UnsignedLong(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedInt => {
                        if let Some(num) = number.to_u32() {
                                Ok(Type::UnsignedInt(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                    }
                    Types::UnsignedShort => {
                        if let Some(num) = number.to_u16() {
                                Ok(Type::UnsignedShort(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                    }
                    Types::UnsignedByte => {
                        if let Some(num) = number.to_u8() {
                                Ok(Type::UnsignedByte(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                    }

                    Types::Long => {
                        if let Some(num) = number.to_i64() {
                            Ok(Type::Long(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::Int => {
                        if let Some(num) = number.to_i32() {
                            Ok(Type::Int(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::Short => {
                        if let Some(num) = number.to_i16() {
                            Ok(Type::Short(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::Byte => {
                        if let Some(num) = number.to_i8() {
                            Ok(Type::Byte(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },

                    Types::PositiveInteger => {
                        if let Some(num) = number.to_i128() {
                            if num > 0 {
                                Ok(Type::PositiveInteger(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NonNegativeInteger => {
                        if let Some(num) = number.to_i128() {
                                if num >= 0 {
                                    Ok(Type::NonNegativeInteger(num))
                                } else {
                                    Err((ErrorCode::FOCA0002, String::from("TODO")))
                                }
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                    },
                    Types::NonPositiveInteger => {
                        if let Some(num) = number.to_i128() {
                                if num <= 0 {
                                    Ok(Type::NonPositiveInteger(num))
                                } else {
                                    Err((ErrorCode::FOCA0002, String::from("TODO")))
                                }
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                    },
                    Types::NegativeInteger => {
                        if let Some(num) = number.to_i128() {
                                if num < 0 {
                                    Ok(Type::NegativeInteger(num))
                                } else {
                                    Err((ErrorCode::FOCA0002, String::from("TODO")))
                                }
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                    },

                    Types::Integer => {
                        if let Some(num) = number.to_i128() {
                            Ok(Type::Integer(num))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::Decimal => Ok((Type::Decimal(number.clone()))),
                    Types::Float => {
                        match number.to_f32() {
                            Some(number) => {
                                let number = OrderedFloat::from(number);
                                Ok(Type::Float(number))
                            },
                            None => Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    },
                    Types::Double => {
                        match number.to_f64() {
                            Some(number) => {
                                let number = OrderedFloat::from(number);
                                Ok(Type::Double(number))
                            },
                            None => Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    },
                    _ => Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            }
            Type::Float(number) => {
                match to {
                    Types::Untyped => {
                        let data = float_to_string(number, false);
                        Ok(Type::Untyped(data))
                    }
                    Types::String => {
                        let data = float_to_string(number, false);
                        Ok(Type::String(data))
                    }
                    Types::Language => Err((ErrorCode::FORG0001, String::from("TODO"))),
                    Types::Boolean => {
                        let b = if number.is_nan() || number.is_zero() {
                            false
                        } else {
                            true
                        };
                        Ok(Type::Boolean(b))
                    }

                    Types::UnsignedLong => {
                        if number.is_zero()|| number.is_normal() {
                            if let Some(num) = number.to_u64() {
                                Ok(Type::UnsignedLong(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedInt => {
                        if number.is_zero()|| number.is_normal() {
                            if let Some(num) = number.to_u32() {
                                Ok(Type::UnsignedInt(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedShort => {
                        if number.is_zero()|| number.is_normal() {
                            if let Some(num) = number.to_u16() {
                                Ok(Type::UnsignedShort(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedByte => {
                        if number.is_zero()|| number.is_normal() {
                            if let Some(num) = number.to_u8() {
                                Ok(Type::UnsignedByte(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }

                    Types::Long => {
                        if number.is_zero()|| number.is_normal() {
                            if let Some(num) = number.0.to_i64() {
                                Ok((Type::Long(num)))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::Int => {
                        if number.is_zero()|| number.is_normal() {
                            if let Some(num) = number.0.to_i32() {
                                Ok((Type::Int(num)))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::Short => {
                        if number.is_zero()|| number.is_normal() {
                            if let Some(num) = number.0.to_i16() {
                                Ok((Type::Short(num)))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::Byte => {
                        if number.is_zero()|| number.is_normal() {
                            if let Some(num) = number.0.to_i8() {
                                Ok((Type::Byte(num)))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },

                    Types::PositiveInteger => {
                        if number.is_zero()|| number.is_normal() {
                            if let Some(num) = number.to_i128() {
                                if num > 0 {
                                    Ok(Type::PositiveInteger(num))
                                } else {
                                    Err((ErrorCode::FOCA0002, String::from("TODO")))
                                }
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NonNegativeInteger => {
                        if number.is_zero()|| number.is_normal() {
                            if let Some(num) = number.to_i128() {
                                if num >= 0 {
                                    Ok(Type::NonNegativeInteger(num))
                                } else {
                                    Err((ErrorCode::FOCA0002, String::from("TODO")))
                                }
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NonPositiveInteger => {
                        if number.is_zero()|| number.is_normal() {
                            if let Some(num) = number.to_i128() {
                                if num <= 0 {
                                    Ok(Type::NonPositiveInteger(num))
                                } else {
                                    Err((ErrorCode::FOCA0002, String::from("TODO")))
                                }
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NegativeInteger => {
                        if number.is_zero()|| number.is_normal() {
                            if let Some(num) = number.to_i128() {
                                if num < 0 {
                                    Ok(Type::NegativeInteger(num))
                                } else {
                                    Err((ErrorCode::FOCA0002, String::from("TODO")))
                                }
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },

                    Types::Integer => {
                        if number.is_zero()|| number.is_normal() {
                            if let Some(num) = number.0.to_i128() {
                                Ok((Type::Integer(num)))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::Decimal => {
                        match BigDecimal::from_f32(number.into_inner()) {
                            Some(number) => Ok((Type::Decimal(number))),
                            None => Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    },
                    Types::Float => Ok(Type::Float(*number)),
                    Types::Double => Ok(Type::Double(OrderedFloat(number.0 as f64))),
                    _ => Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            }
            Type::Double(number) => {
                match to {
                    Types::Untyped => {
                        let data = double_to_string(number, true);
                        Ok(Type::Untyped(data))
                    }
                    Types::String => {
                        let data = double_to_string(number, true);
                        Ok(Type::String(data))
                    }
                    Types::Language => Err((ErrorCode::FORG0001, String::from("TODO"))),
                    Types::Boolean => {
                        let b = if number.is_nan() || number.is_zero() {
                            false
                        } else {
                            true
                        };
                        Ok(Type::Boolean(b))
                    }

                    Types::UnsignedLong => {
                        if number.is_zero()|| number.is_normal() {
                            if let Some(num) = number.to_u64() {
                                Ok(Type::UnsignedLong(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedInt => {
                        if number.is_zero()|| number.is_normal() {
                            if let Some(num) = number.to_u32() {
                                Ok(Type::UnsignedInt(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedShort => {
                        if number.is_zero()|| number.is_normal() {
                            if let Some(num) = number.to_u16() {
                                Ok(Type::UnsignedShort(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }
                    Types::UnsignedByte => {
                        if number.is_zero()|| number.is_normal() {
                            if let Some(num) = number.to_u8() {
                                Ok(Type::UnsignedByte(num))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    }

                    Types::Long => {
                        if number.is_normal() || number.is_zero() {
                            if let Some(num) = number.0.to_i64() {
                                Ok((Type::Long(num)))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::Int => {
                        if number.is_normal() || number.is_zero() {
                            if let Some(num) = number.0.to_i32() {
                                Ok((Type::Int(num)))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::Short => {
                        if number.is_normal() || number.is_zero() {
                            if let Some(num) = number.0.to_i16() {
                                Ok((Type::Short(num)))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::Byte => {
                        if number.is_normal() || number.is_zero() {
                            if let Some(num) = number.0.to_i8() {
                                Ok((Type::Byte(num)))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },

                    Types::PositiveInteger => {
                        if number.is_zero()|| number.is_normal() {
                            if let Some(num) = number.to_i128() {
                                if num > 0 {
                                    Ok(Type::PositiveInteger(num))
                                } else {
                                    Err((ErrorCode::FOCA0002, String::from("TODO")))
                                }
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NonNegativeInteger => {
                        if number.is_zero()|| number.is_normal() {
                            if let Some(num) = number.to_i128() {
                                if num >= 0 {
                                    Ok(Type::NonNegativeInteger(num))
                                } else {
                                    Err((ErrorCode::FOCA0002, String::from("TODO")))
                                }
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NonPositiveInteger => {
                        if number.is_zero()|| number.is_normal() {
                            if let Some(num) = number.to_i128() {
                                if num <= 0 {
                                    Ok(Type::NonPositiveInteger(num))
                                } else {
                                    Err((ErrorCode::FOCA0002, String::from("TODO")))
                                }
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NegativeInteger => {
                        if number.is_zero()|| number.is_normal() {
                            if let Some(num) = number.to_i128() {
                                if num < 0 {
                                    Ok(Type::NegativeInteger(num))
                                } else {
                                    Err((ErrorCode::FOCA0002, String::from("TODO")))
                                }
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },

                    Types::Integer => {
                        if number.is_normal() || number.is_zero() {
                            if let Some(num) = number.0.to_i128() {
                                Ok((Type::Integer(num)))
                            } else {
                                Err((ErrorCode::FOCA0002, String::from("TODO")))
                            }
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::Decimal => {
                        match BigDecimal::from_f64(number.into_inner()) {
                            Some(number) => Ok((Type::Decimal(number))),
                            None => Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    },
                    Types::Float => Ok(Type::Float(OrderedFloat(number.0 as f32))),
                    Types::Double => Ok(Type::Double(*number)),
                    _ => Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            }

            Type::DateTime { dt, offset } => {
                match to {
                    Types::Untyped => {
                        let data = date_time_to_string(dt, offset);
                        Ok(Type::Untyped(data))
                    }
                    Types::String => {
                        let data = date_time_to_string(dt, offset);
                        Ok(Type::String(data))
                    }
                    Types::Language => Err((ErrorCode::FORG0001, String::from("TODO"))),
                    Types::DateTime => {
                        Ok(Type::DateTime { dt: dt.clone(), offset: *offset })
                    }
                    Types::Time => {
                        let time = Time::from(dt.time(), dt.offset().clone());
                        Ok(Type::Time { time, offset: *offset })
                    }
                    Types::Date => {
                        let date = dt.date();
                        Ok(Type::Date { date, offset: *offset })
                    }
                    Types::GYearMonth => {
                        let tz_m = if *offset { Some(dt.timezone().local_minus_utc() / 60) } else { None };
                        Ok(Type::GYearMonth { year: dt.year(), month: dt.month(), tz_m })
                    }
                    Types::GYear => {
                        let tz_m = if *offset { Some(dt.timezone().local_minus_utc() / 60) } else { None };
                        Ok(Type::GYear { year: dt.year(), tz_m })
                    }
                    Types::GMonthDay => {
                        let tz_m = if *offset { Some(dt.timezone().local_minus_utc() / 60) } else { None };
                        Ok(Type::GMonthDay { month: dt.month(), day: dt.day(), tz_m })
                    }
                    Types::GDay => {
                        let tz_m = if *offset { Some(dt.timezone().local_minus_utc() / 60) } else { None };
                        Ok(Type::GDay { day: dt.day(), tz_m })
                    }
                    Types::GMonth => {
                        let tz_m = if *offset { Some(dt.timezone().local_minus_utc() / 60) } else { None };
                        Ok(Type::GMonth { month: dt.month(), tz_m })
                    }
                    _ => Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            },
            Type::DateTimeStamp() => panic!("{:?} from {:?}", to, self),
            Type::Date { date, offset } => {
                match to {
                    Types::Untyped => {
                        let data = date_to_string(date, offset);
                        Ok(Type::Untyped(data))
                    }
                    Types::String => {
                        let data = date_to_string(date, offset);
                        Ok(Type::String(data))
                    }
                    Types::Language => Err((ErrorCode::FORG0001, String::from("TODO"))),
                    Types::DateTime => {
                        Ok(Type::DateTime { dt: date.and_hms(0,0,0), offset: *offset })
                    }
                    Types::Date => {
                        Ok(Type::Date { date: date.clone(), offset: *offset })
                    }
                    Types::GYearMonth => {
                        let tz_m = if *offset { Some(date.timezone().local_minus_utc() / 60) } else { None };
                        Ok(Type::GYearMonth { year: date.year(), month: date.month(), tz_m })
                    }
                    Types::GYear => {
                        let tz_m = if *offset { Some(date.timezone().local_minus_utc() / 60) } else { None };
                        Ok(Type::GYear { year: date.year(), tz_m })
                    }
                    Types::GMonthDay => {
                        let tz_m = if *offset { Some(date.timezone().local_minus_utc() / 60) } else { None };
                        Ok(Type::GMonthDay { month: date.month(), day: date.day(), tz_m })
                    }
                    Types::GDay => {
                        let tz_m = if *offset { Some(date.timezone().local_minus_utc() / 60) } else { None };
                        Ok(Type::GDay { day: date.day(), tz_m })
                    }
                    Types::GMonth => {
                        let tz_m = if *offset { Some(date.timezone().local_minus_utc() / 60) } else { None };
                        Ok(Type::GMonth { month: date.month(), tz_m })
                    }
                    _ => Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            }
            Type::Time { time, offset } => {
                match to {
                    Types::Untyped => {
                        let data = time_to_string(time, offset);
                        Ok(Type::Untyped(data))
                    }
                    Types::String => {
                        let data = time_to_string(time, offset);
                        Ok(Type::String(data))
                    }
                    Types::Language => Err((ErrorCode::FORG0001, String::from("TODO"))),
                    Types::Time => {
                        Ok(Type::Time { time: time.clone(), offset: *offset })
                    }
                    _ => Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            }
            Type::Duration { positive, years, months, days, hours, minutes, seconds, microseconds } => {
                match to {
                    Types::Untyped => {
                        let data = duration_to_string(*positive, *years, *months, *days, *hours, *minutes, *seconds, *microseconds);
                        Ok(Type::Untyped(data))
                    }
                    Types::String => {
                        let data = duration_to_string(*positive, *years, *months, *days, *hours, *minutes, *seconds, *microseconds);
                        Ok(Type::String(data))
                    }
                    Types::Language => Err((ErrorCode::FORG0001, String::from("TODO"))),
                    Types::Duration => {
                        Ok(Type::Duration { positive: *positive, years: *years, months: *months, days: *days, hours: *hours, minutes: *minutes, seconds: *seconds, microseconds: *microseconds } )
                    }
                    Types::YearMonthDuration => {
                        Ok(Type::YearMonthDuration { positive: *positive, years: *years, months: *months } )
                    }
                    Types::DayTimeDuration => {
                        Ok(Type::DayTimeDuration { positive: *positive, days: *days, hours: *hours, minutes: *minutes, seconds: *seconds, microseconds: *microseconds } )
                    }
                    _ => Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            },
            Type::YearMonthDuration { positive, years, months } => {
                match to {
                    Types::Untyped => {
                        let data = year_month_duration_to_string(*positive, *years, *months);
                        Ok(Type::Untyped(data))
                    }
                    Types::String => {
                        let data = year_month_duration_to_string(*positive, *years, *months);
                        Ok(Type::String(data))
                    }
                    Types::Language => Err((ErrorCode::FORG0001, String::from("TODO"))),
                    Types::Duration => {
                        Ok(Type::Duration { positive: *positive, years: *years, months: *months, days: 0, hours: 0, minutes: 0, seconds: 0, microseconds: 0 } )
                    }
                    Types::YearMonthDuration => {
                        Ok(Type::YearMonthDuration { positive: *positive, years: *years, months: *months } )
                    }
                    Types::DayTimeDuration => {
                        Ok(Type::DayTimeDuration { positive: *positive, days: 0, hours: 0, minutes: 0, seconds: 0, microseconds: 0 } )
                    }
                    _ => Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            },
            Type::DayTimeDuration { positive, days, hours, minutes, seconds, microseconds } => {
                match to {
                    Types::Untyped => {
                        let data = day_time_duration_to_string(*positive, *days, *hours, *minutes, *seconds, *microseconds);
                        Ok(Type::Untyped(data))
                    }
                    Types::String => {
                        let data = day_time_duration_to_string(*positive, *days, *hours, *minutes, *seconds, *microseconds);
                        Ok(Type::String(data))
                    }
                    Types::Language => Err((ErrorCode::FORG0001, String::from("TODO"))),
                    Types::Duration => {
                        Ok(Type::Duration { positive: *positive, years: 0, months: 0, days: *days, hours: *hours, minutes: *minutes, seconds: *seconds, microseconds: *microseconds } )
                    }
                    Types::YearMonthDuration => {
                        Ok(Type::YearMonthDuration { positive: *positive, years: 0, months: 0 } )
                    }
                    Types::DayTimeDuration => {
                        Ok(Type::DayTimeDuration { positive: *positive, days: *days, hours: *hours, minutes: *minutes, seconds: *seconds, microseconds: *microseconds } )
                    }
                    _ => Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            },

            Type::GYearMonth { year, month, tz_m } => {
                match to {
                    Types::Untyped => {
                        let data = g_year_month_to_string(*year, *month, *tz_m);
                        Ok(Type::Untyped(data))
                    }
                    Types::String => {
                        let data = g_year_month_to_string(*year, *month, *tz_m);
                        Ok(Type::String(data))
                    }
                    Types::Language => Err((ErrorCode::FORG0001, String::from("TODO"))),
                    Types::GYearMonth => {
                        Ok(Type::GYearMonth { year: *year, month: *month, tz_m: *tz_m })
                    }
                    _ => Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            }
            Type::GYear { year, tz_m } => {
                match to {
                    Types::Untyped => {
                        let data = g_year_to_string(*year, *tz_m);
                        Ok(Type::Untyped(data))
                    }
                    Types::String => {
                        let data = g_year_to_string(*year, *tz_m);
                        Ok(Type::String(data))
                    }
                    Types::Language => Err((ErrorCode::FORG0001, String::from("TODO"))),
                    Types::GYear => {
                        Ok(Type::GYear { year: *year, tz_m: *tz_m })
                    }
                    _ => Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            }
            Type::GMonthDay { month, day, tz_m } => {
                match to {
                    Types::Untyped => {
                        let data = g_month_day_to_string(*month, *day, *tz_m);
                        Ok(Type::Untyped(data))
                    }
                    Types::String => {
                        let data = g_month_day_to_string(*month, *day, *tz_m);
                        Ok(Type::String(data))
                    }
                    Types::Language => Err((ErrorCode::FORG0001, String::from("TODO"))),
                    Types::GMonthDay => {
                        Ok(Type::GMonthDay { month: *month, day: *day, tz_m: *tz_m })
                    }
                    _ => Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            }
            Type::GDay { day, tz_m } => {
                match to {
                    Types::Untyped => {
                        let data = g_day_to_string(*day, *tz_m);
                        Ok(Type::Untyped(data))
                    }
                    Types::String => {
                        let data = g_day_to_string(*day, *tz_m);
                        Ok(Type::String(data))
                    }
                    Types::Language => Err((ErrorCode::FORG0001, String::from("TODO"))),
                    Types::GDay => {
                        Ok(Type::GDay { day: *day, tz_m: *tz_m })
                    }
                    _ => Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            }
            Type::GMonth { month, tz_m } => {
                match to {
                    Types::Untyped => {
                        let data = g_month_to_string(*month, *tz_m);
                        Ok(Type::Untyped(data))
                    }
                    Types::String => {
                        let data = g_month_to_string(*month, *tz_m);
                        Ok(Type::String(data))
                    }
                    Types::Language => Err((ErrorCode::FORG0001, String::from("TODO"))),
                    Types::GMonth => {
                        Ok(Type::GMonth { month: *month, tz_m: *tz_m })
                    }
                    _ => Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            }

            Type::Token(str) => {
                match to {
                    Types::String => Ok(Type::String(str.clone())),
                    Types::Language => {
                        let lang = string_to_language(str)?;
                        Ok(Type::Language(lang))
                    }
                    Types::Token => Ok(Type::Token(str.clone())),
                    _ => panic!("{:?} from {:?}", to, self),
                }
            }
            Type::Language(str) => {
                match to {
                    Types::String => Ok(Type::String(str.clone())),
                    Types::Language => Ok(Type::Language(str.clone())),
                    _ => panic!("{:?} from {:?}", to, self),
                }
            }
            Type::NMTOKEN(str) => {
                match to {
                    Types::String => Ok(Type::String(str.clone())),
                    Types::Language => {
                        let lang = string_to_language(str)?;
                        Ok(Type::Language(lang))
                    }
                    Types::NMTOKEN => Ok(Type::NMTOKEN(str.clone())),
                    _ => panic!("{:?} from {:?}", to, self),
                }
            }

            Type::Name(str) => {
                match to {
                    Types::String => Ok(Type::String(str.clone())),
                    Types::Language => {
                        let lang = string_to_language(str)?;
                        Ok(Type::Language(lang))
                    }
                    Types::Name => Ok(Type::Name(str.clone())),
                    _ => panic!("{:?} from {:?}", to, self),
                }
            }
            Type::NCName(str) => {
                match to {
                    Types::String => Ok(Type::String(str.clone())),
                    Types::Language => {
                        let lang = string_to_language(str)?;
                        Ok(Type::Language(lang))
                    }
                    Types::NCName => Ok(Type::NCName(str.clone())),
                    _ => panic!("{:?} from {:?}", to, self),
                }
            }
            Type::QName { url, prefix, local_part } => {
                match to {
                    Types::String => {
                        let mut str = if let Some(prefix) = prefix {
                            let mut str = String::with_capacity(local_part.len() + 1 + prefix.len());
                            str.push_str(prefix.as_str());
                            str.push_str(":");
                            str
                        } else {
                            String::with_capacity(local_part.len())
                        };
                        str.push_str(local_part.as_str());
                        Ok(Type::String(str))
                    }
                    Types::QName => Ok(Type::QName { url: url.clone(), prefix: prefix.clone(), local_part: local_part.clone() }),
                    _ => panic!("{:?} from {:?}", to, self),
                }
            }

            Type::ID(str) => {
                match to {
                    Types::String => Ok(Type::String(str.clone())),
                    Types::Language => {
                        let lang = string_to_language(str)?;
                        Ok(Type::Language(lang))
                    }
                    Types::ID => Ok(Type::ID(str.clone())),
                    _ => panic!("{:?} from {:?}", to, self),
                }
            }
            Type::IDREF(str) => {
                match to {
                    Types::String => Ok(Type::String(str.clone())),
                    Types::Language => {
                        let lang = string_to_language(str)?;
                        Ok(Type::Language(lang))
                    }
                    Types::IDREF => Ok(Type::IDREF(str.clone())),
                    _ => panic!("{:?} from {:?}", to, self),
                }
            }
            Type::ENTITY(str) => {
                match to {
                    Types::String => Ok(Type::String(str.clone())),
                    Types::Language => {
                        let lang = string_to_language(str)?;
                        Ok(Type::Language(lang))
                    }
                    Types::ENTITY => Ok(Type::ENTITY(str.clone())),
                    _ => panic!("{:?} from {:?}", to, self),
                }
            }

            Type::Boolean(v) => {
                match to {
                    Types::Untyped => {
                        let data = if *v { "true".to_string() } else { "false".to_string() };
                        Ok(Type::Untyped(data))
                    }
                    Types::String => {
                        let data = if *v { "true".to_string() } else { "false".to_string() };
                        Ok(Type::String(data))
                    }
                    Types::NormalizedString => {
                        let data = if *v { "true".to_string() } else { "false".to_string() };
                        Ok(Type::NormalizedString(data))
                    }
                    Types::Language => {
                        let data = if *v { "true".to_string() } else { "false".to_string() };
                        Ok(Type::Language(data))
                    }

                    Types::UnsignedLong => {
                        let num = if *v { 1 } else { 0 };
                        Ok(Type::UnsignedLong(num))
                    }
                    Types::UnsignedInt => {
                        let num = if *v { 1 } else { 0 };
                        Ok(Type::UnsignedInt(num))
                    }
                    Types::UnsignedShort => {
                        let num = if *v { 1 } else { 0 };
                        Ok(Type::UnsignedShort(num))
                    }
                    Types::UnsignedByte => {
                        let num = if *v { 1 } else { 0 };
                        Ok(Type::UnsignedByte(num))
                    }

                    Types::Long => {
                        let num = if *v { 1 } else { 0 };
                        Ok(Type::Long(num))
                    }
                    Types::Int => {
                        let num = if *v { 1 } else { 0 };
                        Ok(Type::Int(num))
                    }
                    Types::Short => {
                        let num = if *v { 1 } else { 0 };
                        Ok(Type::Short(num))
                    }
                    Types::Byte => {
                        let num = if *v { 1 } else { 0 };
                        Ok(Type::Byte(num))
                    }

                    Types::PositiveInteger => {
                        if *v {
                            Ok(Type::PositiveInteger(1))
                        } else {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        }
                    },
                    Types::NonNegativeInteger => {
                        let num = if *v { 1 } else { 0 };
                        Ok(Type::NonNegativeInteger(num))
                    },
                    Types::NonPositiveInteger => {
                        if *v {
                            Err((ErrorCode::FOCA0002, String::from("TODO")))
                        } else {
                            Ok(Type::NonPositiveInteger(0))
                        }
                    },
                    Types::NegativeInteger => {
                        Err((ErrorCode::FORG0001, String::from("TODO")))
                    },

                    Types::Integer => {
                        let number = if *v { 1 } else { 0 };
                        Ok(Type::Integer(number))
                    }
                    Types::Decimal => {
                        let number = if *v { 1 } else { 0 };
                        if let Some(num) = BigDecimal::from_i128(number) {
                            Ok(Type::Decimal(num))
                        } else {
                            Err((ErrorCode::XPTY0004, String::from("TODO")))
                        }
                    }
                    Types::Float => {
                        let number = if *v { 1 } else { 0 } as f32;
                        Ok(Type::Float(OrderedFloat::from(number)))
                    }
                    Types::Double => {
                        let number = if *v { 1 } else { 0 } as f64;
                        Ok(Type::Double(OrderedFloat::from(number)))
                    }

                    Types::Boolean => Ok(Type::Boolean(v.clone())),
                    _ => Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            },

            Type::Base64Binary(binary) => {
                match to {
                    Types::Untyped => {
                        match binary_base64_to_string(binary) {
                            Ok(data) => Ok(Type::Untyped(data)),
                            Err(code) => Err((code, String::from("TODO")))
                        }
                    }
                    Types::String => {
                        match binary_base64_to_string(binary) {
                            Ok(data) => Ok(Type::String(data)),
                            Err(code) => Err((code, String::from("TODO")))
                        }
                    }
                    Types::Base64Binary => Ok(Type::Base64Binary(binary.clone())),
                    Types::HexBinary => Ok(Type::HexBinary(binary.clone())),
                    _ => Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            },
            Type::HexBinary(binary) => {
                match to {
                    Types::Untyped => {
                        match binary_hex_to_string(binary) {
                            Ok(data) => Ok(Type::Untyped(data)),
                            Err(code) => Err((code, String::from("TODO")))
                        }
                    }
                    Types::String => {
                        match binary_hex_to_string(binary) {
                            Ok(data) => Ok(Type::String(data)),
                            Err(code) => Err((code, String::from("TODO")))
                        }
                    }
                    Types::Base64Binary => Ok(Type::Base64Binary(binary.clone())),
                    Types::HexBinary => Ok(Type::HexBinary(binary.clone())),
                    _ => Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            }

            Type::NOTATION() => panic!("{:?} from {:?}", to, self),
        }
    }

    pub(crate) fn to_u8(&self, force: bool) -> Option<u8> {
        if let Some(num) = self.to_i128(force) {
            match u8::try_from(num) {
                Ok(n) => Some(n),
                Err(_) => None
            }
        } else {
            None
        }
    }

    pub(crate) fn to_u16(&self, force: bool) -> Option<u16> {
        if let Some(num) = self.to_i128(force) {
            match u16::try_from(num) {
                Ok(n) => Some(n),
                Err(_) => None
            }
        } else {
            None
        }
    }

    pub(crate) fn to_u32(&self, force: bool) -> Option<u32> {
        if let Some(num) = self.to_i128(force) {
            match u32::try_from(num) {
                Ok(n) => Some(n),
                Err(_) => None
            }
        } else {
            None
        }
    }

    pub(crate) fn to_u64(&self, force: bool) -> Option<u64> {
        if let Some(num) = self.to_i128(force) {
            match u64::try_from(num) {
                Ok(n) => Some(n),
                Err(_) => None
            }
        } else {
            None
        }
    }

    pub(crate) fn to_i8(&self, force: bool) -> Option<i8> {
        if let Some(num) = self.to_i128(force) {
            match i8::try_from(num) {
                Ok(n) => Some(n),
                Err(_) => None
            }
        } else {
            None
        }
    }

    pub(crate) fn to_i16(&self, force: bool) -> Option<i16> {
        if let Some(num) = self.to_i128(force) {
            match i16::try_from(num) {
                Ok(n) => Some(n),
                Err(_) => None
            }
        } else {
            None
        }
    }

    pub(crate) fn to_i32(&self, force: bool) -> Option<i32> {
        if let Some(num) = self.to_i128(force) {
            match i32::try_from(num) {
                Ok(n) => Some(n),
                Err(_) => None
            }
        } else {
            None
        }
    }

    pub(crate) fn to_i64(&self, force: bool) -> Option<i64> {
        if let Some(num) = self.to_i128(force) {
            match i64::try_from(num) {
                Ok(n) => Some(n),
                Err(_) => None
            }
        } else {
            None
        }
    }

    pub(crate) fn to_i128(&self, force: bool) -> Option<i128> {
        match self {
            Type::Untyped(str) => {
                match crate::values::string_to::double(str, true) {
                    Ok(num) => num.to_i128(force),
                    Err(_) => None
                }
            }

            Type::UnsignedByte(num) => Some(*num as i128),
            Type::UnsignedShort(num) => Some(*num as i128),
            Type::UnsignedInt(num) => Some(*num as i128),
            Type::UnsignedLong(num) => Some(*num as i128),

            Type::Byte(num) => Some(*num as i128),
            Type::Short(num) => Some(*num as i128),
            Type::Int(num) => Some(*num as i128),
            Type::Long(num) => Some(*num as i128),

            Type::Byte(num) => Some(*num as i128),
            Type::Short(num) => Some(*num as i128),
            Type::Int(num) => Some(*num as i128),
            Type::Long(num) => Some(*num as i128),

            Type::PositiveInteger(num) => Some(*num),
            Type::NonNegativeInteger(num) => Some(*num),
            Type::NonPositiveInteger(num) => Some(*num),
            Type::NegativeInteger(num) => Some(*num),

            Type::Integer(num) => Some(*num),
            Type::Decimal(num) => {
                let rounded = num.round(0);
                if force || &rounded == num {
                    rounded.to_i128()
                } else {
                    None
                }
            },
            Type::Float(num) => {
                let rounded = num.round();
                if force || rounded == num.0 {
                    rounded.to_i128()
                } else {
                    None
                }
            }
            Type::Double(num) => {
                let rounded = num.round();
                if force || rounded == num.0 {
                    rounded.to_i128()
                } else {
                    None
                }
            }
            _ => None
        }
    }

    fn to_decimal(&self) -> Option<BigDecimal> {
        match self {
            Type::Integer(number) => BigDecimal::from_i128(*number),
            Type::Decimal(number) => Some(number.clone()),
            Type::Float(number) => {
                BigDecimal::from_f32(number.into_inner())
            }
            Type::Double(number) => {
                BigDecimal::from_f64(number.into_inner())
            },
            _ => None
        }
    }

    fn to_float(&self) -> Option<OrderedFloat<f32>> {
        match self {
            Type::Integer(number) => OrderedFloat::from_i128(*number),
            Type::Decimal(number) => {
                if let Some(number) = number.to_f32() {
                    OrderedFloat::from_f32(number)
                } else {
                    None
                }
            },
            Type::Float(number) => Some(*number),
            Type::Double(number) => {
                if let Some(number) = number.to_f32() {
                    OrderedFloat::from_f32(number)
                } else {
                    None
                }
            },
            _ => None
        }
    }

    fn to_double(&self) -> Option<OrderedFloat<f64>> {
        match self {
            Type::Integer(number) => OrderedFloat::from_i128(*number),
            Type::Decimal(number) => {
                if let Some(number) = number.to_f64() {
                    OrderedFloat::from_f64(number)
                } else {
                    None
                }
            },
            Type::Float(number) => {
                if let Some(number) = number.to_f64() {
                    OrderedFloat::from_f64(number)
                } else {
                    None
                }
            },
            Type::Double(number) => Some(number.clone()),
            _ => None
        }
    }

    fn is_nan(&self) -> bool {
        match self {
            Type::Float(num) => num.is_nan(),
            Type::Double(num) => num.is_nan(),
            _ => false
        }
    }

    fn to_comparison_type(&self) -> Types {
        match self {
            Type::Untyped(_) |
            Type::String(_) |
            Type::NormalizedString(_) |
            Type::AnyURI(_) => Types::String,

            Type::Byte(_) |
            Type::Short(_) |
            Type::Int(_) |
            Type::Long(_) |

            Type::UnsignedByte(_) |
            Type::UnsignedShort(_) |
            Type::UnsignedInt(_) |
            Type::UnsignedLong(_) |

            Type::PositiveInteger(_) |
            Type::NonNegativeInteger(_) |
            Type::NonPositiveInteger(_) |
            Type::NegativeInteger(_) |

            Type::Integer(_) |
            Type::Decimal(_) |
            Type::Float(_) |
            Type::Double(_) => Types::Numeric,
            Type::DateTime { .. } => Types::DateTime,
            Type::DateTimeStamp() => Types::DateTimeStamp,
            Type::Date { .. } => Types::Date,
            Type::Time { .. } => Types::Time,
            Type::Duration { .. } => Types::Duration,
            Type::YearMonthDuration { .. } => Types::YearMonthDuration,
            Type::DayTimeDuration { .. } => Types::DayTimeDuration,
            Type::GYearMonth { .. } => Types::GYearMonth,
            Type::GYear { .. } => Types::GYear,
            Type::GMonthDay { .. } => Types::GMonthDay,
            Type::GDay { .. } => Types::GDay,
            Type::GMonth { .. } => Types::GMonth,
            Type::Token(_) => Types::Token,
            Type::Language(_) => Types::Language,
            Type::NMTOKEN(_) => Types::NMTOKEN,
            Type::Name(_) => Types::Name,
            Type::NCName(_) => Types::NCName,
            Type::QName { .. } => Types::QName,
            Type::ID(_) => Types::ID,
            Type::IDREF(_) => Types::IDREF,
            Type::ENTITY(_) => Types::ENTITY,
            Type::Boolean(_) => Types::Boolean,
            Type::Base64Binary(_) => Types::Base64Binary,
            Type::HexBinary(_) => Types::HexBinary,
            Type::NOTATION() => Types::NOTATION,
        }
    }

    fn is_comparable(&self, other: &Type) -> bool {
        println!("is_comparable {:?} vs {:?}", self.to_comparison_type(), other.to_comparison_type());
        self.to_comparison_type() == other.to_comparison_type()
    }

    pub(crate) fn value_comparison(&self, other: &Type) -> Result<ValueOrdering, ErrorInfo> {
        if self == other {
            return Ok(ValueOrdering::Equal);
        }

        if !self.is_comparable(other) {
            return Err((ErrorCode::XPTY0004, String::from("TODO")));
        }

        if self.is_nan() || other.is_nan() {
            return Ok(ValueOrdering::AlwaysNotEqual);
        }

        match self {
            Type::Untyped(l_str) |
            Type::AnyURI(l_str) |
            Type::String(l_str) |
            Type::NormalizedString(l_str) => {
                // xs:string or xs:anyURI => xs:string
                if let Type::String(r_str) = other.convert(Types::String)? {
                    Ok(ValueOrdering::from(l_str.cmp(&r_str)))
                } else {
                    Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            }
            Type::NCName(l_str) => {
                match other {
                    Type::NCName(r_str) => {
                        Ok(ValueOrdering::from(l_str.cmp(&r_str)))
                    }
                    _ => Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            }
            Type::QName { url: l_url, prefix: l_prefix, local_part: l_local_part } => {
                if let Type::QName { url: r_url, prefix: r_prefix, local_part: r_local_part } = other.convert(Types::QName)? {
                    if l_url == &r_url && l_local_part == &r_local_part {
                        Ok(ValueOrdering::QNameEqual)
                    } else {
                        Ok(ValueOrdering::QNameNotEqual)
                    }
                } else {
                    Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            },
            Type::Boolean(lbt) => {
                if let Type::Boolean(rbt) = other.convert(Types::Boolean)? {
                    Ok(ValueOrdering::from(lbt.cmp(&rbt)))
                } else {
                    Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            },

            Type::Byte(..) |
            Type::Short(..) |
            Type::Int(..) |
            Type::Long(..) |

            Type::UnsignedByte(..) |
            Type::UnsignedShort(..) |
            Type::UnsignedInt(..) |
            Type::UnsignedLong(..) |

            Type::PositiveInteger(..) |
            Type::NonNegativeInteger(..) |
            Type::NonPositiveInteger(..) |
            Type::NegativeInteger(..) |

            Type::Integer(..) |
            Type::Decimal(..) |
            Type::Float(..) |
            Type::Double(..) => {
                let lnt = self.to_type();
                let rnt = other.to_type();

                // xs:integer, xs:decimal or xs:float => xs:float
                // xs:integer, xs:decimal, xs:float, or xs:double => xs:double
                let nt = if lnt > rnt { lnt } else { rnt };
                match nt {
                    Types::UnsignedByte => {
                        if let Some(left_num) = self.to_u8(false) {
                            if let Some(right_num) = other.to_u8(false) {
                                return Ok(ValueOrdering::from(left_num.cmp(&right_num)));
                            }
                        }
                        return Err((ErrorCode::XPTY0004, String::from("TODO")));
                    }
                    Types::UnsignedShort => {
                        if let Some(left_num) = self.to_u16(false) {
                            if let Some(right_num) = other.to_u16(false) {
                                return Ok(ValueOrdering::from(left_num.cmp(&right_num)));
                            }
                        }
                        return Err((ErrorCode::XPTY0004, String::from("TODO")));
                    }
                    Types::UnsignedInt => {
                        if let Some(left_num) = self.to_u32(false) {
                            if let Some(right_num) = other.to_u32(false) {
                                return Ok(ValueOrdering::from(left_num.cmp(&right_num)));
                            }
                        }
                        return Err((ErrorCode::XPTY0004, String::from("TODO")));
                    }
                    Types::UnsignedLong => {
                        if let Some(left_num) = self.to_u64(false) {
                            if let Some(right_num) = other.to_u64(false) {
                                return Ok(ValueOrdering::from(left_num.cmp(&right_num)));
                            }
                        }
                        return Err((ErrorCode::XPTY0004, String::from("TODO")));
                    }

                    Types::Byte => {
                        if let Some(left_num) = self.to_i8(false) {
                            if let Some(right_num) = other.to_i8(false) {
                                return Ok(ValueOrdering::from(left_num.cmp(&right_num)));
                            }
                        }
                        return Err((ErrorCode::XPTY0004, String::from("TODO")));
                    }
                    Types::Short => {
                        if let Some(left_num) = self.to_i16(false) {
                            if let Some(right_num) = other.to_i16(false) {
                                return Ok(ValueOrdering::from(left_num.cmp(&right_num)));
                            }
                        }
                        return Err((ErrorCode::XPTY0004, String::from("TODO")));
                    }
                    Types::Int => {
                        if let Some(left_num) = self.to_i32(false) {
                            if let Some(right_num) = other.to_i32(false) {
                                return Ok(ValueOrdering::from(left_num.cmp(&right_num)));
                            }
                        }
                        return Err((ErrorCode::XPTY0004, String::from("TODO")));
                    }
                    Types::Long => {
                        if let Some(left_num) = self.to_i64(false) {
                            if let Some(right_num) = other.to_i64(false) {
                                return Ok(ValueOrdering::from(left_num.cmp(&right_num)));
                            }
                        }
                        return Err((ErrorCode::XPTY0004, String::from("TODO")));
                    }

                    Types::PositiveInteger |
                    Types::NonNegativeInteger |
                    Types::NonPositiveInteger |
                    Types::NegativeInteger |
                    Types::Integer => {
                        if let Some(left_num) = self.to_i128(false) {
                            if let Some(right_num) = other.to_i128(false) {
                                return Ok(ValueOrdering::from(left_num.cmp(&right_num)));
                            }
                        }
                        return Err((ErrorCode::XPTY0004, String::from("TODO")));
                    },
                    Types::Decimal => {
                        if let Some(left_num) = self.to_decimal() {
                            if let Some(right_num) = other.to_decimal() {
                                return Ok(ValueOrdering::from(left_num.cmp(&right_num)));
                            }
                        }
                        return Err((ErrorCode::XPTY0004, String::from("TODO")));
                    },
                    Types::Float => {
                        if let Some(left_num) = self.to_float() {
                            if let Some(right_num) = other.to_float() {
                                return Ok(ValueOrdering::from(left_num.cmp(&right_num)));
                            }
                        }
                        return Err((ErrorCode::XPTY0004, String::from("TODO")));
                    },
                    Types::Double => {
                        if let Some(left_num) = self.to_double() {
                            if let Some(right_num) = other.to_double() {
                                return Ok(ValueOrdering::from(left_num.cmp(&right_num)));
                            }
                        }
                        return Err((ErrorCode::XPTY0004, String::from("TODO")));
                    },
                    _ => panic!("internal error")
                }
            },
            Type::DateTime { dt: l_dt, offset: l_offset } => {
                if let Type::DateTime { dt: r_dt, offset: r_offset } = other.convert(Types::DateTime)? {
                    Ok(ValueOrdering::from(l_dt.cmp(&r_dt)))
                } else {
                    Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            }
            Type::Time { time: l_time, offset: l_offset } => {
                if let Type::Time { time: r_time, offset: r_offset } = other.convert(Types::Time)? {
                    Ok(ValueOrdering::from(l_time.cmp(&r_time)))
                } else {
                    Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            }
            Type::Date { date: l_date, offset: l_offset } => {
                if let Type::Date { date: r_date, offset: r_offset } = other.convert(Types::Date)? {
                    Ok(ValueOrdering::from(l_date.cmp(&r_date)))
                } else {
                    Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            }
            // TODO priority for durations!
            Type::Duration {
                positive: l_positive,
                years: l_years, months: l_months, days: l_days,
                hours: l_hours, minutes: l_minutes, seconds: l_seconds, microseconds: l_microseconds
            } => {
                if let Type::Duration {
                        positive: r_positive,
                        years: r_years, months: r_months, days: r_days,
                        hours: r_hours, minutes: r_minutes, seconds: r_seconds, microseconds: r_microseconds
                    } = other.convert(Types::Duration)?
                {
                    let l_sign = if *l_positive { 1 } else { -1 };
                    let l_ms = l_sign * *l_microseconds as i128 + (1000 * (*l_seconds as i128 + 60 * (*l_minutes as i128 + 60 * (*l_hours as i128 + 24 * (*l_days as i128 + 31 * (*l_months as i128 + 12 * *l_years as i128))))));
                    let r_sign = if r_positive { 1 } else { -1 };
                    let r_ms = r_sign * r_microseconds as i128 + (1000 * (r_seconds as i128 + 60 * (r_minutes as i128 + 60 * (r_hours as i128 + 24 * (r_days as i128 + 31 * (r_months as i128 + 12 * r_years as i128))))));
                    Ok(ValueOrdering::from(l_ms.cmp(&r_ms)))
                } else {
                    Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            }
            Type::YearMonthDuration { positive: l_positive, years: l_years, months: l_months } => {
                if let Type::YearMonthDuration { positive: r_positive, years: r_years, months: r_months } = other.convert(Types::YearMonthDuration)? {
                    let l_sign = if *l_positive { 1 } else { -1 };
                    let l_ms = l_sign * *l_months as i128 + (12 * *l_years as i128);
                    let r_sign = if r_positive { 1 } else { -1 };
                    let r_ms = r_sign * r_months as i128 + (12 * r_years as i128);
                    Ok(ValueOrdering::from(l_ms.cmp(&r_ms)))
                } else {
                    Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            }
            Type::DayTimeDuration {
                positive: l_positive, days: l_days,
                hours: l_hours, minutes: l_minutes, seconds: l_seconds, microseconds: l_microseconds
            } => {
                if let Type::DayTimeDuration {
                    positive: r_positive, days: r_days,
                    hours: r_hours, minutes: r_minutes, seconds: r_seconds, microseconds: r_microseconds
                } = other.convert(Types::DayTimeDuration)?
                {
                    let l_sign = if *l_positive { 1 } else { -1 };
                    let l_ms = l_sign * *l_microseconds as i128 + (1000 * (*l_seconds as i128 + 60 * (*l_minutes as i128 + 60 * (*l_hours as i128 + 24 * (*l_days as i128)))));
                    let r_sign = if r_positive { 1 } else { -1 };
                    let r_ms = r_sign * r_microseconds as i128 + (1000 * (r_seconds as i128 + 60 * (r_minutes as i128 + 60 * (r_hours as i128 + 24 * (r_days as i128)))));
                    Ok(ValueOrdering::from(l_ms.cmp(&r_ms)))
                } else {
                    Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            }

            Type::GYearMonth { year: l_year, month: l_month, tz_m: l_tz_m } => {
                if let Type::GYearMonth { year: r_year, month: r_month, tz_m: r_tz_m } = other.convert(Types::GYearMonth)? {
                    Ok(ValueOrdering::from(((*l_year * 12) + *l_month as i32).cmp(&((r_year * 12) + r_month as i32))))
                } else {
                    Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            }
            Type::GYear { year: l_year,  tz_m: l_tz_m } => {
                if let Type::GYear { year: r_year, tz_m: r_tz_m } = other.convert(Types::GYear)? {
                    Ok(ValueOrdering::from((*l_year).cmp(&r_year)))
                } else {
                    Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            }
            Type::GMonthDay { month: l_month, day: l_day, tz_m: l_tz_m } => {
                if let Type::GMonthDay { month: r_month, day: r_day, tz_m: r_tz_m } = other.convert(Types::GMonthDay)? {
                    Ok(ValueOrdering::from((*l_month * 31 + *l_day).cmp(&(r_month * 31 + r_day))))
                } else {
                    Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            }
            Type::GDay { day: l_day, tz_m: l_tz_m } => {
                if let Type::GDay { day: r_day, tz_m: r_tz_m } = other.convert(Types::GDay)? {
                    Ok(ValueOrdering::from((*l_day).cmp(&r_day)))
                } else {
                    Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            }
            Type::GMonth { month: l_month, tz_m: l_tz_m } => {
                if let Type::GMonth { month: r_month, tz_m: r_tz_m } = other.convert(Types::GMonth)? {
                    Ok(ValueOrdering::from((*l_month).cmp(&r_month)))
                } else {
                    Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            }
            _ => panic!("{:?} vs {:?}", self, other) // Err((ErrorCode::XPTY0004, String::from("TODO")))
        }
    }
}

pub(crate) fn object_to_qname(t: Object) -> QName {
    match t {
        Object::Atomic(Type::String(str)) => {
            if str.contains(":") {
                todo!()
            }
            QName::local_part(str.as_str())
        }
        Object::Atomic(Type::QName { prefix, url, local_part }) =>
                       QName { prefix, url, local_part },
        _ => panic!("can't convert to QName {:?}", t)
    }
}

pub(crate) fn string_to_qname(env: &Environment, str: String) -> QName {
    if str.contains(":") {
        let mut parts = str.split(":").map(|s| s.to_string()).collect::<Vec<_>>();
        if parts.len() == 2 {
            QName::new(parts.remove(0), parts.remove(0))
        } else {
            panic!("raise error!")
        }
    } else {
        QName::local_part(str)
    }
}

pub fn string_to_double(string: &String) -> Result<Object, ErrorCode> {
    match string.trim().parse() {
        Ok(number) => {
            Ok(Object::Atomic(Type::Double(number)))
        },
        Err(..) => Err(ErrorCode::FORG0001)
    }
}

pub fn string_to_decimal(string: &String) -> Result<BigDecimal, ErrorCode> {
    match string.trim().parse() {
        Ok(num) => Ok(num),
        Err(..) => Err(ErrorCode::FORG0001)
    }
}

pub fn string_to_binary_hex(string: &String) -> Result<Vec<u8>, ErrorCode> {
    match hex::decode(string) {
        Ok(binary) => Ok(binary),
        Err(_) => Err(ErrorCode::FORG0001)
    }
}

pub fn binary_hex_to_string(binary: &Vec<u8>) -> Result<String, ErrorCode> {
    Ok(hex::encode(binary).to_uppercase())
}

pub fn string_to_binary_base64(string: &String) -> Result<Vec<u8>, ErrorCode> {
    match base64::decode(string) {
        Ok(binary) => Ok(binary),
        Err(_) => Err(ErrorCode::FORG0001)
    }
}

pub fn binary_base64_to_string(binary: &Vec<u8>) -> Result<String, ErrorCode> {
    Ok(base64::encode(binary))
}

#[derive(Clone)]
pub enum Object {
    Range { min: i128, max: i128 },

    Error { code: String },
    CharRef { representation: Representation, reference: u32 },
    EntityRef(String),

    Nothing,

    Empty,
    Sequence(Vec<Object>),

    Atomic(Type),
    Node(Reference),

    Array(Vec<Object>),
    Map(HashMap<Type, Object>),

    Function { parameters: Vec<Param>, st: Option<SequenceType>, body: Box<dyn Expression> },
    FunctionRef { name: QNameResolved, arity: usize },

    Return(Box<Object>),
}

impl Object {
    pub(crate) fn to_integer(&self) -> Result<i128, ErrorInfo> {
        match self {
            Object::Atomic(t) => {
                let n = t.convert(Types::Integer)?;
                match n {
                    Type::Integer(num) => Ok(num),
                    _ => Err((ErrorCode::XPTY0004, format!("can't convert to int {:?}", t)))
                }
            },
            Object::Node(rf) => {
                match rf.to_typed_value() {
                    Ok(num) => {
                        match num.parse() {
                            Ok(v) => Ok(v),
                            Err(..) => Err((ErrorCode::XPTY0004, format!("can't convert to int {:?}", num)))
                        }
                    },
                    Err(msg) => Err((ErrorCode::XPTY0004, format!("can't convert node to int")))
                }
            }
            _ => Err((ErrorCode::XPTY0004, format!("can't convert to int {:?}", self)))
        }
    }
}

impl<'a> PartialEq<Self> for Object {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Object::Range { min: l_min, max: l_max } => {
                match other {
                    Object::Range { min: r_min, max: r_max } => l_min == r_min && l_max == r_max,
                    _ => false
                }
            }
            Object::Error { code: l_code } => {
                match other {
                    Object::Error { code: r_code } => l_code == r_code,
                    _ => false
                }
            }
            Object::CharRef { representation: l_representation, reference: l_reference } => {
                match other {
                    Object::CharRef { representation: r_representation, reference: r_reference } =>
                        l_representation == r_representation && l_reference == r_reference,
                    _ => false
                }
            }
            Object::EntityRef(l_ref) => {
                match other {
                    Object::EntityRef(r_ref) => l_ref == r_ref,
                    _ => false
                }
            }
            Object::Nothing => {
                match other {
                    Object::Nothing => true,
                    _ => false
                }
            }
            Object::Empty => {
                match other {
                    Object::Empty => true,
                    _ => false
                }
            }
            Object::Sequence(l_items) => {
                match other {
                    Object::Sequence(r_items) => l_items == r_items,
                    _ => false
                }
            }
            Object::Atomic(l_type) => {
                match other {
                    Object::Atomic(r_type) => l_type == r_type,
                    _ => false
                }
            }
            Object::Node(l_node) => {
                match other {
                    Object::Node(r_node) => l_node == r_node,
                    _ => false
                }
            }
            Object::Array(l_items) => {
                match other {
                    Object::Array(r_items) => l_items == r_items,
                    _ => false
                }
            }
            Object::Map(l_entries) => {
                match other {
                    Object::Map(r_entries) => l_entries == r_entries,
                    _ => false
                }
            }
            Object::Function { .. } => {
                false
            }
            Object::FunctionRef { name: l_name, arity: l_arity } => {
                match other {
                    Object::FunctionRef { name: r_name, arity: r_arity } => l_name == r_name && l_arity == r_arity,
                    _ => false
                }
            }
            Object::Return(l_item) => {
                match other {
                    Object::Return(r_item) => l_item == r_item,
                    _ => false
                }
            }
        }
    }
}

impl Debug for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Object::Range { min, max } => {
                f.debug_tuple("Range")
                    .field(min)
                    .field(max)
                    .finish()
            }
            Object::Error { code } => {
                f.debug_tuple("Error")
                    .field(code)
                    .finish()
            }
            Object::CharRef { representation, reference } => {
                let data = match representation {
                    Representation::Hexadecimal => { format!("&#x{:X};", reference) }
                    Representation::Decimal => { format!("&#{};", reference) }
                };

                f.debug_tuple("CharRef")
                    .field(&data)
                    .finish()
            }
            Object::EntityRef(code) => {
                f.debug_tuple("EntityRef")
                    .field(code)
                    .finish()
            }
            Object::Nothing => f.debug_struct("Nothing").finish(),
            Object::Empty => f.debug_struct("Empty").finish(),
            Object::Sequence(items) => {
                f.debug_tuple("Sequence")
                    .field(items)
                    .finish()
            }
            Object::Atomic(t) => {
                f.debug_tuple("Atomic")
                    .field(t)
                    .finish()
            }
            Object::Node(node) => {
                f.debug_tuple("Node")
                    .field(node)
                    .finish()
            }
            Object::Array(items) => {
                f.debug_tuple("Array")
                    .field(items)
                    .finish()
            }
            Object::Map(entries) => {
                f.debug_tuple("Map")
                    .field(entries)
                    .finish()
            }
            Object::Function { parameters, .. } => {
                f.debug_tuple("Function")
                    .field(parameters)
                    .finish()
            }
            Object::FunctionRef { name, arity } => {
                f.debug_struct("FunctionRef")
                    .field("name", name)
                    .field("arity", arity)
                    .finish_non_exhaustive()
            }
            Object::Return(_) => {
                f.debug_struct("Return").finish_non_exhaustive()
            }
        }
    }
}

impl Eq for Object {}

impl Ord for Object {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            Object::Atomic(t1) => {
                match other {
                    Object::Atomic(t2) => {
                        t1.cmp(t2)
                    },
                    _ => Ordering::Greater,
                }
            },
            Object::Node(n1) => {
                match other {
                    Object::Node(n2) => {
                        n1.cmp(n2)
                    },
                    _ => Ordering::Less,
                }
            },
            _ => Ordering::Less
        }
    }
}

impl PartialOrd<Self> for Object {
    fn partial_cmp(&self, other: &Object) -> Option<Ordering> {
        match self {
            Object::Atomic(t1) => {
                match other {
                    Object::Atomic(t2) => {
                        t1.partial_cmp(t2)
                    },
                    _ => None,
                }
            },
            Object::Node(n1) => {
                match other {
                    Object::Node(n2) => {
                        Some(n1.cmp(n2))
                    },
                    _ => None,
                }
            },
            _ => None
        }
    }
}

fn zero_or_one(items: &mut Vec<Object>) -> Result<Object, ErrorInfo> {
    sort_and_dedup(items);
    if items.len() == 1 {
        Ok(items.remove(0))
    } else if items.len() == 0 {
        Ok(Object::Empty)
    } else {
        Err((ErrorCode::XPTY0004, String::from("TODO")))
    }
}

pub fn atomization_of_vec(env: &Box<Environment>, items: Vec<Object>) -> Result<Object, ErrorInfo> {
    let mut result = Vec::with_capacity(items.len());
    for item in items {
        let value = atomization(env, item)?;
        match value {
            Object::Sequence(elements) => {
                for el in elements {
                    result.push(el);
                }
            }
            _ => result.push(value)
        }
    }
    zero_or_one(&mut result)
}

pub(crate) fn atomization(env: &Box<Environment>, obj: Object) -> Result<Object, ErrorInfo> {
    match obj {
        Object::Atomic(..) => Ok(obj),
        Object::Node(rf) => {
            match rf.to_typed_value() {
                Ok(data) => Ok(Object::Atomic(Type::Untyped(data))),
                Err(msg) => Err((ErrorCode::TODO, msg))
            }

        },
        Object::Array(items) => atomization_of_vec(env, items),
        Object::Sequence(items) => atomization_of_vec(env, items),
        Object::Range { min, max } => {
            if min == max {
                Ok(Object::Atomic(Type::Integer(min)))
            } else {
                Err((ErrorCode::XPTY0004, String::from("TODO")))
            }
        },
        Object::Empty => Ok(obj), // or it can be XPST0005?
        Object::Function { .. } |
        Object::FunctionRef { .. } |
        Object::Map(..) => Err((ErrorCode::FOTY0013, String::from("TODO"))),
        _ => todo!()
    }
}

pub(crate) fn sequence_atomization(env: &Box<Environment>, obj: Object) -> Result<Object, ErrorInfo> {
    match obj {
        Object::Range { .. } |
        Object::Array(..) |
        Object::Sequence(..) |
        Object::Atomic(..) => Ok(obj),
        Object::Node(rf) => {
            match rf.to_typed_value() {
                Ok(data) => Ok(Object::Atomic(Type::Untyped(data))),
                Err(msg) => Err((ErrorCode::TODO, msg))
            }
        },
        Object::Empty => Ok(obj), // or it can be XPST0005?
        _ => todo!()
    }
}

pub(crate) fn normalizing_string(str: &String) -> String {
    str.trim_matches(|c| c == ' ' || c == '\t' || c == '\n' || c == '\r')
        .replace("\t", " ")
        .replace("\n", " ")
        .replace("\r", " ")
}

pub(crate) fn string_to_token(str: &String) -> String {
    let mut old_str= normalizing_string(str);

    // TODO optimize this
    let mut new_str = old_str.replace("  ", " ");
    while old_str != new_str {
        old_str = new_str;
        new_str = old_str.replace("  ", " ");
    }
    new_str
}

fn string_to_name(input: &str) -> Result<String, ErrorInfo> {
    let input = input.trim();
    if input.is_empty() {
        Err((ErrorCode::FORG0001, String::from("TODO")))
    } else {
        match parse_name(input) {
            Ok((i, name)) => {
                if i.len() == 0 {
                    Ok(input.to_string())
                } else {
                    Err((ErrorCode::FORG0001, String::from("TODO")))
                }
            }
            Err(_) => Err((ErrorCode::FORG0001, String::from("TODO")))
        }
    }
}

fn string_to_ncname(input: &str) -> Result<String, ErrorInfo> {
    let input = input.trim();
    if input.is_empty() {
        Err((ErrorCode::FORG0001, String::from("TODO")))
    } else {
        match parse_ncname(input) {
            Ok((i, name)) => {
                if i.len() == 0 {
                    Ok(input.to_string())
                } else {
                    Err((ErrorCode::FORG0001, String::from("TODO")))
                }
            }
            Err(_) => Err((ErrorCode::FORG0001, String::from("TODO")))
        }
    }
}

fn string_to_nmtoken(input: &str) -> Result<String, ErrorInfo> {
    let input = input.trim();
    if input.is_empty() {
        Err((ErrorCode::FORG0001, String::from("TODO")))
    } else {
        match parse_nmtoken(input) {
            Ok((i, name)) => {
                if i.len() == 0 {
                    Ok(input.to_string())
                } else {
                    Err((ErrorCode::FORG0001, String::from("TODO")))
                }
            }
            Err(_) => Err((ErrorCode::FORG0001, String::from("TODO")))
        }
    }
}

fn string_to_language(input: &String) -> Result<String, ErrorInfo> {
    let input = input.trim();
    let result: Result<(&str, (&str, Vec<&str>)), nom::Err<Error<&str>>> = tuple((
        alpha1,
        many0(
            preceded(
                tag("-"),
                alphanumeric1,
            )
        )
    ))(input);
    match result {
        Ok((i, (fst, others))) => {
            if i.len() == 0 {
                if fst.len() < 8 {
                    for o in others {
                        if o.len() > 8 {
                            return Err((ErrorCode::FORG0001, String::from("TODO")))
                        }
                    }
                    return Ok(input.to_string())
                }
            }
            return Err((ErrorCode::FORG0001, String::from("TODO")))
        },
        Err(_) => Err((ErrorCode::FORG0001, String::from("TODO")))
    }
}

pub(crate) fn new_g_month_day(month: u32, day: u32, tz_m: Option<i32>) -> Result<Type, String> {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => {
            if day < 1 || day > 31 {
                return Err(String::from("TODO"))
            }
        }
        4 | 6 | 9 | 11 => {
            if day < 1 || day > 30 {
                return Err(String::from("TODO"))
            }
        }
        2 => {
            if day < 1 || day > 29 {
                return Err(String::from("TODO"))
            }
        }
        _ => return Err(String::from("TODO"))
    }

    if let Some(tz) = tz_m {
        // The ·recoverable timezone· of a date will always be a duration between '+12:00' and '11:59'.

        // Timezones are durations with (integer-valued) hour and minute properties
        // (with the hour magnitude limited to at most 14,
        // and the minute magnitude limited to at most 59,
        // except that if the hour magnitude is 14, the minute value must be 0);
        // they may be both positive or both negative.
        if tz < -840 || tz > 840 {
            return Err(String::from("TODO"))
        }
    }

    Ok(Type::GMonthDay { month, day, tz_m })
}