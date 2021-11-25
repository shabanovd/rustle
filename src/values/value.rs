use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Write, Debug, Formatter};
use crate::values::{QName, QNameResolved};
use crate::fns::Param;
use crate::parser::op::{Representation};
use crate::parser::errors::ErrorCode;
use ordered_float::OrderedFloat;
use bigdecimal::{BigDecimal, FromPrimitive, ToPrimitive};
use crate::eval::helpers::sort_and_dedup;
use crate::eval::expression::Expression;
use chrono::{Date, DateTime, FixedOffset, Local, TimeZone};
use crate::eval::{Environment, ErrorInfo};
use crate::eval::comparison::ValueOrdering;
use crate::eval::sequence_type::SequenceType;
use crate::parser::parse_duration::*;
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
    Integer = 110,
    Decimal = 140,
    Float   = 150,
    Double  = 160,

    NonPositiveInteger = 111,
    NegativeInteger = 112,
    Long = 120,
    Int = 121,
    Short = 122,
    Byte = 123,

    NonNegativeInteger = 113,
    UnsignedLong = 130,
    UnsignedInt = 131,
    UnsignedShort = 132,
    UnsignedByte = 133,

    PositiveInteger = 114,

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

    Integer(i128),
    Decimal(BigDecimal),
    Float(OrderedFloat<f32>),
    Double(OrderedFloat<f64>),

    // nonPositiveInteger(),
    // negativeInteger(),
    // long(),
    // int(),
    // short(),
    // byte(),

    // nonNegativeInteger(),
    // unsignedLong(),
    // unsignedInt(),
    // unsignedShort(),
    // unsignedByte(),

    // positiveInteger(),

    DateTime(DateTime<FixedOffset>),
    DateTimeStamp(),

    Date(Date<FixedOffset>),
    Time(Time<FixedOffset>),

    Duration { positive: bool, years: u32, months: u32, days: u32, hours: u32, minutes: u32, seconds: u32, microseconds: u32 },
    YearMonthDuration  { positive: bool, years: u32, months: u32 },
    DayTimeDuration { positive: bool, days: u32, hours: u32, minutes: u32, seconds: u32, microseconds: u32 },

    GYearMonth { year: i32, month: u32, tz_m: i32 },
    GYear { year: i32, tz_m: i32},
    GMonthDay { month: i32, day: u32, tz_m: i32 },
    GDay { day: i32, tz_m: i32 },
    GMonth { month: i32, tz_m: i32 },

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

    pub(crate) fn date_now() -> Type {
        let now = Local::now();
        let date = Date::from_utc(now.date().naive_utc(), TimeZone::from_offset(now.offset()));

        Type::Date(date)
    }

    pub(crate) fn time_now() -> Type {
        Type::Time(Time::now())
    }

    pub(crate) fn to_type(&self) -> Types {
        match self {
            Type::Untyped(_) => Types::Untyped,
            Type::String(_) => Types::String,
            Type::NormalizedString(_) => Types::NormalizedString,

            Type::AnyURI(_) => Types::AnyURI,

            Type::Integer(_) => Types::Integer,
            Type::Decimal(_) => Types::Decimal,
            Type::Float(_) => Types::Float,
            Type::Double(_) => Types::Double,

            Type::DateTime(_) => Types::DateTime,
            Type::DateTimeStamp() => Types::DateTimeStamp,
            Type::Date(_) => Types::Date,
            Type::Time(_) => Types::Time,
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
                    Types::String => Ok(Type::String(str.clone())),
                    _ => panic!("{:?} from {:?}", to, self)
                }
            },
            Type::Untyped(str) |
            Type::String(str) |
            Type::NormalizedString(str) => {
                match to {
                    Types::Untyped => Ok(Type::Untyped(str.clone())),
                    Types::String => Ok(Type::String(str.clone())),
                    Types::NormalizedString => Ok(Type::NormalizedString(str.clone())),
                    Types::AnyURI => Ok(Type::AnyURI(str.clone())),
                    Types::Name => Ok(Type::Name(str.clone())),
                    Types::NCName => Ok(Type::NCName(str.clone())),
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
                    Types::Boolean => {
                        if str == "false" || str == "0" {
                            Ok(Type::Boolean(false))
                        } else if str == "true" || str == "1" {
                            Ok(Type::Boolean(true))
                        } else {
                            Err((ErrorCode::FORG0001, format!("The string {:?} cannot be cast to a boolean", str)))
                        }
                    }
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
                    _ => panic!("{:?} from {:?}", to, self) // Err((ErrorCode::XPTY0004, String::from("TODO")))
                }
            }

            Type::Integer(_) => panic!("{:?} from {:?}", to, self),
            Type::Decimal(_) => panic!("{:?} from {:?}", to, self),
            Type::Float(_) => panic!("{:?} from {:?}", to, self),
            Type::Double(_) => panic!("{:?} from {:?}", to, self),

            Type::DateTime(_) => panic!("{:?} from {:?}", to, self),
            Type::DateTimeStamp() => panic!("{:?} from {:?}", to, self),
            Type::Date(_) => panic!("{:?} from {:?}", to, self),
            Type::Time(_) => panic!("{:?} from {:?}", to, self),
            Type::Duration { .. } => panic!("{:?} from {:?}", to, self),
            Type::YearMonthDuration { .. } => panic!("{:?} from {:?}", to, self),
            Type::DayTimeDuration { .. } => panic!("{:?} from {:?}", to, self),

            Type::GYearMonth { .. } => panic!("{:?} from {:?}", to, self),
            Type::GYear { .. } => panic!("{:?} from {:?}", to, self),
            Type::GMonthDay { .. } => panic!("{:?} from {:?}", to, self),
            Type::GDay { .. } => panic!("{:?} from {:?}", to, self),
            Type::GMonth { .. } => panic!("{:?} from {:?}", to, self),

            Type::Token(_) => panic!("{:?} from {:?}", to, self),
            Type::Language(_) => panic!("{:?} from {:?}", to, self),
            Type::NMTOKEN(_) => panic!("{:?} from {:?}", to, self),

            Type::Name(_) => panic!("{:?} from {:?}", to, self),
            Type::NCName(_) => panic!("{:?} from {:?}", to, self),
            Type::QName { .. } => panic!("{:?} from {:?}", to, self),

            Type::ID(_) => panic!("{:?} from {:?}", to, self),
            Type::IDREF(_) => panic!("{:?} from {:?}", to, self),
            Type::ENTITY(_) => panic!("{:?} from {:?}", to, self),

            Type::Boolean(v) => {
                match to {
                    Types::Boolean => Ok(Type::Boolean(v.clone())),
                    _ => panic!("{:?} from {:?}", to, self)
                }
            },

            Type::Base64Binary(_) => panic!("{:?} from {:?}", to, self),
            Type::HexBinary(_) => panic!("{:?} from {:?}", to, self),

            Type::NOTATION() => panic!("{:?} from {:?}", to, self),
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
            Type::Integer(_) |
            Type::Decimal(_) |
            Type::Float(_) |
            Type::Double(_) => Types::Numeric,
            Type::DateTime(_) => Types::DateTime,
            Type::DateTimeStamp() => Types::DateTimeStamp,
            Type::Date(_) => Types::Date,
            Type::Time(_) => Types::Time,
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
    if string.len() % 2 != 0 {
        Err(ErrorCode::FORG0001)
    } else {
        let result = (0..string.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&string[i..i + 2], 16).map_err(|e| ErrorCode::FORG0001))
            .collect();

        match result {
            Ok(binary) => Ok(binary),
            Err(code) => Err(code)
        }
    }
}

const HEX_BYTES: &str = "000102030405060708090a0b0c0d0e0f\
101112131415161718191a1b1c1d1e1f\
202122232425262728292a2b2c2d2e2f\
303132333435363738393a3b3c3d3e3f\
404142434445464748494a4b4c4d4e4f\
505152535455565758595a5b5c5d5e5f\
606162636465666768696a6b6c6d6e6f\
707172737475767778797a7b7c7d7e7f\
808182838485868788898a8b8c8d8e8f\
909192939495969798999a9b9c9d9e9f\
a0a1a2a3a4a5a6a7a8a9aaabacadaeaf\
b0b1b2b3b4b5b6b7b8b9babbbcbdbebf\
c0c1c2c3c4c5c6c7c8c9cacbcccdcecf\
d0d1d2d3d4d5d6d7d8d9dadbdcdddedf\
e0e1e2e3e4e5e6e7e8e9eaebecedeeef\
f0f1f2f3f4f5f6f7f8f9fafbfcfdfeff";

pub fn binary_hex_to_string(binary: &Vec<u8>) -> Result<String, ErrorCode> {
    let mut string = String::with_capacity(binary.len() * 2);
    for &b in binary {
        // write!(&mut string, "{:02x}", b).unwrap();
        let i = 2 * b as usize;
        if let Some(str) = HEX_BYTES.get(i..i + 2) {
            string.push_str(str);
        } else {
            return Err(ErrorCode::TODO)
        }
    }
    Ok(string)
}

const UPPERCASEOFFSET: i8 = 65;
const LOWERCASEOFFSET: i8 = 71;
const DIGITOFFSET: i8 = -4;

pub fn string_to_binary_base64(string: &String) -> Result<Vec<u8>, ErrorCode> {
    let mut binary = Vec::with_capacity(string.len());
    for ch in string.chars() {
        let ch = ch as i8;
        let b = match ch {
            // A-Z
            65..=90 => ch - UPPERCASEOFFSET,
            // a-z
            97..=122 => ch - LOWERCASEOFFSET,
            // 0-9
            48..=57 => ch - DIGITOFFSET,
            // +
            43 => 62,
            // /
            47 => 63,
            _ => return Err(ErrorCode::FORG0001),
        } as u8;
        binary.push(b);
    };

    Ok(binary)
}

pub fn binary_base64_string(binary: &Vec<u8>) -> Result<String, ErrorCode> {
    let mut string = String::with_capacity(binary.len());
    for b in binary {
        let b = *b as i8;
        let ch = match b {
            // A-Z
            0..=25 => b + UPPERCASEOFFSET,
            // a-z
            26..=51 => b + LOWERCASEOFFSET,
            // 0-9
            52..=61 => b + DIGITOFFSET,
            // +
            62 => 43,
            // /
            63 => 47,
            _ => return Err(ErrorCode::TODO),
        } as u8;
        string.push(ch as char);
    }
    Ok(string)
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