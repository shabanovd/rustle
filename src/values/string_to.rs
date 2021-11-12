use crate::eval::ErrorInfo;
use crate::parser::errors::ErrorCode;
use crate::values::Type;
use ordered_float::OrderedFloat;

pub(crate) fn integer(str: &str) -> Result<Type, ErrorInfo> {
    match str.parse() {
        Ok(num) => Ok(Type::Integer(num)),
        Err(_) => Err((ErrorCode::FORG0001, format!("can't convert to int {:?}", str)))
    }
}

pub(crate) fn decimal(str: &str) -> Result<Type, ErrorInfo> {
    match str.parse() {
        Ok(num) => Ok(Type::Decimal(num)),
        Err(_) => Err((ErrorCode::FORG0001, format!("can't convert to decimal {:?}", str)))
    }
}

pub(crate) fn float(str: &str, nan_on_error: bool) -> Result<Type, ErrorInfo> {
    match str.parse() {
        Ok(num) => Ok(Type::Float(num)),
        Err(_) => {
            if nan_on_error {
                Ok(Type::Float(OrderedFloat::from(f32::NAN)))
            } else {
                Err((ErrorCode::FORG0001, format!("can't convert to decimal {:?}", str)))
            }
        }
    }
}

pub(crate) fn double(str: &str, nan_on_error: bool) -> Result<Type, ErrorInfo> {
    match str.parse() {
        Ok(num) => Ok(Type::Double(num)),
        Err(_) => {
            if nan_on_error {
                Ok(Type::Double(OrderedFloat::from(f64::NAN)))
            } else {
                Err((ErrorCode::FORG0001, format!("can't convert to decimal {:?}", str)))
            }
        }
    }
}