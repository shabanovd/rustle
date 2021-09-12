use crate::eval::{Object, Type, NumberCase};
use rust_decimal::Decimal;
use crate::serialization::object_to_string;
use crate::parser::parse_duration::string_to_dt_duration;
use std::cmp::Ordering;

pub(crate) fn cmp(left: &Object, right: &Object) -> Option<Ordering> {
    if left == right {
        Some(Ordering::Equal)
    } else {
        // xs:string or xs:anyURI => xs:string
        if let Some(l_str) = object_to_string_if_string(left) {
            if let Some(r_str) = object_to_string_if_string(right) {
                return Some(l_str.cmp(&r_str));
            } else {
                return None;
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
                                Some(left_num.cmp(&right_num))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    },
                    NT::Decimal |
                    NT::Double => {
                        if let Some(left_num) = object_to_decimal(left) {
                            if let Some(right_num) = object_to_decimal(right) {
                                Some(left_num.cmp(&right_num))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    },
                }
            }
        }
        None
    }
}

pub(crate) fn eq(left: &Object, right: &Object) -> bool {
    match cmp(left, right) {
        Some(v) => v == Ordering::Equal,
        _ => false
    }
}

pub(crate) fn ls_or_eq(left: &Object, right: &Object) -> bool {
    match cmp(left, right) {
        Some(v) => v == Ordering::Equal || v == Ordering::Less,
        _ => false
    }
}

pub(crate) fn ls(left: &Object, right: &Object) -> bool {
    match cmp(left, right) {
        Some(v) => v == Ordering::Less,
        _ => false
    }
}

pub(crate) fn gr_or_eq(left: &Object, right: &Object) -> bool {
    match cmp(left, right) {
        Some(v) => v == Ordering::Equal || v == Ordering::Greater,
        _ => false
    }
}

pub(crate) fn gr(left: &Object, right: &Object) -> bool {
    match cmp(left, right) {
        Some(v) => v == Ordering::Greater,
        _ => false
    }
}

pub(crate) fn general_eq(left: &Object, right: &Object) -> bool {
    match left {
        Object::Atomic(lt) => {
            match right {
                Object::Atomic(Type::Untyped(rs)) => {
                    match lt {
                        Type::Untyped(ls) => {
                            ls == rs
                        }
                        Type::DayTimeDuration { .. } => {
                            match string_to_dt_duration(rs) {
                                Ok(rd) => lt == &rd,
                                Err(e) => panic!("error") // TODO: Err(..)
                            }
                        }
                        Type::Integer(..) |
                        Type::Decimal {..} |
                        Type::Float {..} |
                        Type::Decimal {..} => {
                            if let Ok(num) = Decimal::from_scientific(rs) {
                                let rv = Object::Atomic(Type::Double { number: Some(num), case: NumberCase::Normal });
                                eq(left, &rv)
                            } else {
                                panic!("error")
                            }
                        }
                        _ => panic!("error")
                    }
                }
                Object::Atomic(..) => {
                    eq(left, right)
                },
                Object::Sequence(items) => {
                    for item in items {
                        if eq(left, item) {
                            return true;
                        }
                    }
                    false
                }
                _ => panic!("error")
            }
        }
        Object::Sequence(left_items) => {
            match right {
                Object::Atomic(..) => {
                    for item in left_items {
                        if eq(left, item) {
                            return true;
                        }
                    }
                    false
                },
                Object::Sequence(right_items) => {
                    for left_item in left_items {
                        for right_item in right_items {
                            if eq(left_item, right_item) {
                                return true;
                            }
                        }
                    }
                    false
                }
                _ => panic!("error")
            }
        }
        _ => panic!("error")
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