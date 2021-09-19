use crate::eval::{Object, Type, RangeIterator};
use crate::serialization::node_to_string;
use crate::parser::op::Representation;

pub fn object_to_string_xml(object: &Object) -> String {
    _object_to_string(object, false)
}

pub fn object_to_string(object: &Object) -> String {
    _object_to_string(object, true)
}

fn _object_to_string(object: &Object, ref_resolving: bool) -> String {
    match object {
        Object::ForBinding { values, .. } => {
            _object_to_string(values, ref_resolving)
        },
        Object::Range { min, max } => {
            let (it, count) = RangeIterator::create(*min, *max);

            let mut buf = Vec::with_capacity(count);
            for item in it {
                buf.push(_object_to_string(&item, ref_resolving));
            }

            buf.join(" ")
        }
        Object::CharRef { reference, representation } => {
            if ref_resolving {
                String::from(ref_to_char(*reference))
            } else {
                match representation {
                    Representation::Hexadecimal => {
                        format!("&#x{:X}", reference)
                    }
                    Representation::Decimal => {
                        format!("&#{}", reference)
                    }
                }
            }
        },
        Object::EntityRef(reference) => {
            match reference.as_str() {
                "lt" => String::from("<"),
                "gt" => String::from(">"),
                "amp" => String::from("&"),
                "quot" => String::from("\""),
                "apos" => String::from("'"),
                _ => panic!("unexpected {:?}", reference)
            }
        },
        Object::Atomic(Type::Untyped(str)) => str.clone(),
        Object::Atomic(Type::AnyURI(str)) => str.clone(),
        Object::Atomic(Type::String(str)) => str.clone(),
        Object::Atomic(Type::Integer(number)) => number.to_string(),
        Object::Atomic(Type::Decimal(number)) => number.to_string(),
        Object::Atomic(Type::Float(number)) => {
            if number.is_nan() {
                String::from("NaN")
            } else if number.is_infinite() {
                if number.is_sign_positive() {
                    String::from("INF")
                } else {
                    String::from("-INF")
                }
            } else {
                number.to_string()
            }
        },
        Object::Atomic(Type::Double(number)) => {
            if number.is_nan() {
                String::from("NaN")
            } else if number.is_infinite() {
                if number.is_sign_positive() {
                    String::from("INF")
                } else {
                    String::from("-INF")
                }
            } else {
                number.to_string()
            }
        },
        Object::Atomic(Type::DateTime(dt)) => {
            dt.to_rfc3339()
        }
        Object::Atomic(Type::Date(date)) => {
            date.format("%Y-%m-%d").to_string()
        }
        Object::Atomic(Type::Time(time)) => {
            time.format("%H:%M:%S").to_string()
        }
        Object::Sequence(items) => {
            let mut buf = String::new();
            for item in items {
                let str = _object_to_string(item, ref_resolving);
                buf.push_str(str.as_str());
            }
            buf
        },
        Object::Node(node) => {
            node_to_string(node)
        },
        _ => panic!("TODO object_to_string {:?}", object)
    }
}

fn ref_to_string(representation: Representation, code: u32) -> String {
     match representation {
        Representation::Hexadecimal => { format!("&#x{:X};", code) }
        Representation::Decimal => { format!("&#{};", code) }
    }
}

pub(crate) fn ref_to_char(code: u32) -> char {
    char::from_u32(code).unwrap()
}