use crate::eval::{Object, Type};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

pub(crate) fn eq(left: Object, right: Object) -> bool {

    if left == right {
        true
    } else {
        // xs:string or xs:anyURI => xs:string
        // xs:integer, xs:decimal or xs:float => xs:float
        // xs:integer, xs:decimal, xs:float, or xs:double => xs:double
        if let Some((left_num, lnt)) = object_to_number(left) {
            if let Some((right_num, rnt)) = object_to_number(right) {

                return if lnt == rnt {
                    left_num.eq(&right_num)
                } else {
                    let nt = if lnt > rnt { lnt } else { rnt };
                    match nt {
                        NT::Integer => (left_num.is_zero() && right_num.is_zero()) || left_num.eq(&right_num),
                        NT::Decimal |
                        NT::Double => left_num.eq(&right_num),
                    }
                };
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

fn object_to_number(obj: Object) -> Option<(Decimal, NT)> {
    match obj {
        Object::Atomic(Type::Integer(num)) => Some((num, NT::Integer)),
        Object::Atomic(Type::Decimal(num)) => Some((num, NT::Decimal)),
        Object::Atomic(Type::Double(num)) => Some((num, NT::Double)),
        _ => None
    }
}