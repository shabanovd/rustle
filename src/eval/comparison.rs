use crate::eval::{Object, Type};
use rust_decimal::Decimal;

pub(crate) fn eq(left: Object, right: Object) -> bool {

    if left == right {
        true
    } else {
        // xs:string or xs:anyURI => xs:string
        // xs:integer, xs:decimal or xs:float => xs:float
        // xs:integer, xs:decimal, xs:float, or xs:double => xs:double
        if let Some(lnt) = object_to_number_type(&left) {
            if let Some(rnt) = object_to_number_type(&right) {
                let nt = if lnt > rnt { lnt } else { rnt };
                return match nt {
                    NT::Integer => {
                        if let Some(left_num) = object_to_i128(left) {
                            if let Some(right_num) = object_to_i128(right) {
                                left_num.eq(&right_num)
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    },
                    NT::Decimal |
                    NT::Double => {
                        if let Some(left_num) = object_to_decimal(left) {
                            if let Some(right_num) = object_to_decimal(right) {
                                left_num.eq(&right_num)
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    },
                }
            }
        }

        false
    }
}

#[derive(Eq, PartialEq, PartialOrd)]
pub enum NT {
    Integer = 1,
    Decimal = 2,
    Double  = 3,
}

fn object_to_number_type(obj: &Object) -> Option<NT> {
    match obj {
        Object::Atomic(Type::Integer(num)) => Some(NT::Integer),
        Object::Atomic(Type::Decimal(num)) => Some(NT::Decimal),
        Object::Atomic(Type::Double(num)) => Some(NT::Double),
        _ => None
    }
}

fn object_to_i128(obj: Object) -> Option<i128> {
    match obj {
        Object::Atomic(Type::Integer(num)) => Some(num),
        // Object::Atomic(Type::Decimal(num)) => Some(num),
        // Object::Atomic(Type::Double(num)) => Some(num),
        _ => None
    }
}

fn object_to_decimal(obj: Object) -> Option<Decimal> {
    match obj {
        Object::Atomic(Type::Integer(num)) => Some(Decimal::from(num)),
        Object::Atomic(Type::Decimal(num)) |
        Object::Atomic(Type::Double(num)) => Some(num),
        _ => None
    }
}