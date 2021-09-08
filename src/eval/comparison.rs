use crate::eval::{Object, Type};
use rust_decimal::Decimal;
use crate::serialization::object_to_string;

pub(crate) fn eq(left: &Object, right: &Object) -> bool {

    if left == right {
        true
    } else {
        // xs:string or xs:anyURI => xs:string
        if let Some(l_str) = object_to_string_if_string(left) {
            if let Some(r_str) = object_to_string_if_string(right) {
                return l_str == r_str;
            } else {
                return false;
            }
        }

        // xs:integer, xs:decimal or xs:float => xs:float
        // xs:integer, xs:decimal, xs:float, or xs:double => xs:double
        if let Some(lnt) = object_to_number_type(left) {
            if let Some(rnt) = object_to_number_type(right) {
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

pub(crate) fn deep_eq(left: &Object, right: &Object) -> bool {
    if left == right {
        true
    } else {
        match left {
            Object::Atomic(..) => {
                match right {
                    Object::Atomic(..) => {
                        eq(left, right)
                    }
                    _ => false
                }
            }
            Object::Sequence(left_items) => {
                match right {
                    Object::Sequence(right_items) => {
                        println!("l: {:?} {:?}", left_items.len(), left_items);
                        println!("r: {:?} {:?}", right_items.len(), right_items);
                        if left_items.len() != right_items.len() {
                            false
                        } else {
                            let mut left_it = left_items.iter();
                            let mut right_it = right_items.iter();

                            loop {
                                if let Some(left_item) = left_it.next() {
                                    if let Some(right_item) = right_it.next() {
                                        if deep_eq(left_item, right_item) {
                                            return false;
                                        }
                                    } else {
                                        return false;
                                    }
                                } else {
                                    if let Some(right_item) = right_it.next() {
                                        return false;
                                    }
                                    return true;
                                }
                            }
                        }
                    },
                    _ => false
                }
            },
            _ => panic!("TODO {:?}", left)
        }
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
        Object::Atomic(Type::Integer(..)) => Some(NT::Integer),
        Object::Atomic(Type::Decimal { .. }) => Some(NT::Decimal),
        Object::Atomic(Type::Double { .. }) => Some(NT::Double),
        _ => None
    }
}

fn object_to_i128(obj: &Object) -> Option<i128> {
    match obj {
        Object::Atomic(Type::Integer(num)) => Some(*num),
        // Object::Atomic(Type::Decimal(num)) => Some(num),
        // Object::Atomic(Type::Double(num)) => Some(num),
        _ => None
    }
}

fn object_to_decimal(obj: &Object) -> Option<Decimal> {
    match obj {
        Object::Atomic(Type::Integer(num)) => Some(Decimal::from(*num)),
        Object::Atomic(Type::Decimal { number, case }) |
        Object::Atomic(Type::Double { number, case }) => *number,
        _ => None
    }
}

fn object_to_string_if_string(obj: &Object) -> Option<String> {
    match obj {
        Object::Atomic(Type::String(..)) |
        Object::Atomic(Type::NormalizedString(..)) |
        Object::CharRef {..} |
        Object::EntityRef(..) => {
            Some(object_to_string(obj))
        }
        _ => None
    }
}