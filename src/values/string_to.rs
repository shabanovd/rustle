use crate::eval::ErrorInfo;
use crate::parser::errors::ErrorCode;
use crate::values::Type;

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

pub(crate) fn float(str: &str) -> Result<Type, ErrorInfo> {
    match str.parse() {
        Ok(num) => Ok(Type::Float(num)),
        Err(_) => Err((ErrorCode::FORG0001, format!("can't convert to decimal {:?}", str)))
    }
}

pub(crate) fn double(str: &str) -> Result<Type, ErrorInfo> {
    match str.parse() {
        Ok(num) => Ok(Type::Double(num)),
        Err(_) => Err((ErrorCode::FORG0001, format!("can't convert to decimal {:?}", str)))
    }
}