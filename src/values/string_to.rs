use bigdecimal::{BigDecimal, Zero};
use nom::bytes::complete::tag;
use nom::combinator::all_consuming;
use nom::error::Error;
use nom::multi::{many0, many1};
use nom::sequence::tuple;
use crate::eval::ErrorInfo;
use crate::parser::errors::ErrorCode;
use crate::values::{Type, Types};
use ordered_float::OrderedFloat;

fn is_minus_zero(str: &str) -> bool {
    let check: Result<(&str, (Vec<&str>, Vec<&str>)), nom::Err<Error<&str>>> =
        all_consuming(tuple((many0(tag("-")), many1(tag("0")))))(str);
    check.is_ok()
}

pub(crate) fn unsigned_byte(str: &str) -> Result<Type, ErrorInfo> {
    let str = str.trim();

    // workaround for -0 case
    if is_minus_zero(str) {
        Ok(Type::UnsignedByte(0))
    } else {
        match str.parse() {
            Ok(num) => Ok(Type::UnsignedByte(num)),
            Err(_) => Err((ErrorCode::FORG0001, format!("can't convert to byte {:?}", str)))
        }
    }
}

pub(crate) fn unsigned_short(str: &str) -> Result<Type, ErrorInfo> {
    let str = str.trim();

    // workaround for -0 case
    if is_minus_zero(str) {
        Ok(Type::UnsignedShort(0))
    } else {
        match str.trim().parse() {
            Ok(num) => Ok(Type::UnsignedShort(num)),
            Err(_) => Err((ErrorCode::FORG0001, format!("can't convert to short {:?}", str)))
        }
    }
}

pub(crate) fn unsigned_int(str: &str) -> Result<Type, ErrorInfo> {
    let str = str.trim();

    // workaround for -0 case
    if is_minus_zero(str) {
        Ok(Type::UnsignedInt(0))
    } else {
        match str.parse() {
            Ok(num) => Ok(Type::UnsignedInt(num)),
            Err(_) => Err((ErrorCode::FORG0001, format!("can't convert to int {:?}", str)))
        }
    }
}

pub(crate) fn unsigned_long(str: &str) -> Result<Type, ErrorInfo> {
    let str = str.trim();

    // workaround for -0 case
    if is_minus_zero(str) {
        Ok(Type::UnsignedLong(0))
    } else {
        match str.parse() {
            Ok(num) => Ok(Type::UnsignedLong(num)),
            Err(_) => Err((ErrorCode::FORG0001, format!("can't convert to long {:?}", str)))
        }
    }
}

pub(crate) fn byte(str: &str) -> Result<Type, ErrorInfo> {
    match str.trim().parse() {
        Ok(num) => Ok(Type::Byte(num)),
        Err(_) => Err((ErrorCode::FORG0001, format!("can't convert to byte {:?}", str)))
    }
}

pub(crate) fn short(str: &str) -> Result<Type, ErrorInfo> {
    match str.trim().parse() {
        Ok(num) => Ok(Type::Short(num)),
        Err(_) => Err((ErrorCode::FORG0001, format!("can't convert to short {:?}", str)))
    }
}

pub(crate) fn int(str: &str) -> Result<Type, ErrorInfo> {
    match str.trim().parse() {
        Ok(num) => Ok(Type::Int(num)),
        Err(_) => Err((ErrorCode::FORG0001, format!("can't convert to int {:?}", str)))
    }
}

pub(crate) fn long(str: &str) -> Result<Type, ErrorInfo> {
    match str.trim().parse() {
        Ok(num) => Ok(Type::Long(num)),
        Err(_) => Err((ErrorCode::FORG0001, format!("can't convert to long {:?}", str)))
    }
}

pub(crate) fn positive_integer(str: &str) -> Result<Type, ErrorInfo> {
    match str.trim().parse() {
        Ok(num) => {
            if num > 0 {
                Ok(Type::PositiveInteger(num))
            } else {
                Err((ErrorCode::FORG0001, format!("can't convert to positive integer {:?}", str)))
            }
        },
        Err(_) => Err((ErrorCode::FORG0001, format!("can't convert to positive integer {:?}", str)))
    }
}

pub(crate) fn non_negative_integer(str: &str) -> Result<Type, ErrorInfo> {
    match str.trim().parse() {
        Ok(num) => {
            if num >= 0 {
                Ok(Type::NonNegativeInteger(num))
            } else {
                Err((ErrorCode::FORG0001, format!("can't convert to non negative integer {:?}", str)))
            }
        },
        Err(_) => Err((ErrorCode::FORG0001, format!("can't convert to non negative integer {:?}", str)))
    }
}

pub(crate) fn non_positive_integer(str: &str) -> Result<Type, ErrorInfo> {
    match str.trim().parse() {
        Ok(num) => {
            if num <= 0 {
                Ok(Type::NonPositiveInteger(num))
            } else {
                Err((ErrorCode::FORG0001, format!("can't convert to non positive integer {:?}", str)))
            }
        },
        Err(_) => Err((ErrorCode::FORG0001, format!("can't convert to non positive integer {:?}", str)))
    }
}

pub(crate) fn negative_integer(str: &str) -> Result<Type, ErrorInfo> {
    match str.trim().parse() {
        Ok(num) => {
            if num < 0 {
                Ok(Type::NegativeInteger(num))
            } else {
                Err((ErrorCode::FORG0001, format!("can't convert to negative integer {:?}", str)))
            }
        },
        Err(_) => Err((ErrorCode::FORG0001, format!("can't convert to negative integer {:?}", str)))
    }
}

pub(crate) fn integer(str: &str) -> Result<Type, ErrorInfo> {
    match str.trim().parse() {
        Ok(num) => Ok(Type::Integer(num)),
        Err(_) => Err((ErrorCode::FORG0001, format!("can't convert to integer {:?}", str)))
    }
}

pub(crate) fn decimal(str: &str) -> Result<Type, ErrorInfo> {
    match str.trim().parse::<BigDecimal>() {
        Ok(num) => {
            // workaround -0 and 'e|E' case
            if str.chars().any(|c| c == 'e' || c == 'E') || (str.starts_with("-") && num.is_zero()) {
                Err((ErrorCode::FORG0001, format!("can't convert to decimal {:?}", str)))
            } else {
                Ok(Type::Decimal(num))
            }
        },
        Err(_) => Err((ErrorCode::FORG0001, format!("can't convert to decimal {:?}", str)))
    }
}

pub(crate) fn float(mut str: &str, nan_on_error: bool) -> Result<Type, ErrorInfo> {
    str = str.trim();

    // workaround for NaN, INF, +INF and -INF cases
    if str.len() == 3 {
        if str.eq_ignore_ascii_case("nan") {
            return if str == "NaN" {
                Ok(Type::Float(OrderedFloat::from(f32::NAN)))
            } else {
                Err(ErrorCode::forg0001(&str.to_string(), Types::Float))
            }
        } else if str.eq_ignore_ascii_case("inf") {
            return if str == "INF" {
                Ok(Type::Float(OrderedFloat::from(f32::INFINITY)))
            } else {
                Err(ErrorCode::forg0001(&str.to_string(), Types::Float))
            }
        }
    } else if str.len() == 4 {
        if str.eq_ignore_ascii_case("+inf") {
            return if str == "+INF" {
                Ok(Type::Float(OrderedFloat::from(f32::INFINITY)))
            } else {
                Err(ErrorCode::forg0001(&str.to_string(), Types::Float))
            }
        } else if str.eq_ignore_ascii_case("-inf") {
            return if str == "-INF" {
                Ok(Type::Float(OrderedFloat::from(f32::NEG_INFINITY)))
            } else {
                Err(ErrorCode::forg0001(&str.to_string(), Types::Float))
            }
        }
    }

    match str.parse() {
        Ok(num) => Ok(Type::Float(num)),
        Err(_) => {
            if nan_on_error {
                Ok(Type::Float(OrderedFloat::from(f32::NAN)))
            } else {
                Err(ErrorCode::forg0001(&str.to_string(), Types::Float))
            }
        }
    }
}

pub(crate) fn double(mut str: &str, nan_on_error: bool) -> Result<Type, ErrorInfo> {
    str = str.trim();

    // workaround for NaN, INF, +INF and -INF cases
    if str.len() == 3 {
        if str.eq_ignore_ascii_case("nan") {
            return if str == "NaN" {
                Ok(Type::Double(OrderedFloat::from(f64::NAN)))
            } else {
                Err(ErrorCode::forg0001(&str.to_string(), Types::Double))
            }
        } else if str.eq_ignore_ascii_case("inf") {
            return if str == "INF" {
                Ok(Type::Double(OrderedFloat::from(f64::INFINITY)))
            } else {
                Err(ErrorCode::forg0001(&str.to_string(), Types::Double))
            }
        }
    } else if str.len() == 4 {
        if str.eq_ignore_ascii_case("+inf") {
            return if str == "+INF" {
                Ok(Type::Double(OrderedFloat::from(f64::INFINITY)))
            } else {
                Err(ErrorCode::forg0001(&str.to_string(), Types::Double))
            }
        } else if str.eq_ignore_ascii_case("-inf") {
            return if str == "-INF" {
                Ok(Type::Double(OrderedFloat::from(f64::NEG_INFINITY)))
            } else {
                Err(ErrorCode::forg0001(&str.to_string(), Types::Double))
            }
        }
    }

    match str.parse() {
        Ok(num) => Ok(Type::Double(num)),
        Err(_) => {
            if nan_on_error {
                Ok(Type::Double(OrderedFloat::from(f64::NAN)))
            } else {
                Err(ErrorCode::forg0001(&str.to_string(), Types::Double))
            }
        }
    }
}