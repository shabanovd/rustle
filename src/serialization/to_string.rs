use crate::eval::{Object, Type, RangeIterator, Environment};
use crate::parser::op::Representation;

pub fn object_to_string_xml(env: &Box<Environment>, object: &Object) -> String {
    _object_to_string(env, object, false, " ")
}

pub fn object_to_string(env: &Box<Environment>, object: &Object) -> String {
    _object_to_string(env, object, true, " ")
}

pub fn _object_to_string(env: &Box<Environment>, object: &Object, ref_resolving: bool, sep: &str) -> String {
    match object {
        Object::Empty => String::new(),
        Object::Range { min, max } => {
            let (it, count) = RangeIterator::create(*min, *max);

            let mut buf = Vec::with_capacity(count);
            for item in it {
                buf.push(_object_to_string(env, &item, ref_resolving, sep));
            }

            buf.join(sep)
        }
        Object::CharRef { representation, reference } => {
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
        Object::Atomic(Type::Boolean(b)) => b.to_string(),
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
            let mut buf = Vec::with_capacity(items.len());
            for item in items {
                let data = _object_to_string(env, item, ref_resolving, " ");
                buf.push(data);
            }
            let data = buf.join(sep);
            data
        },
        Object::Node(rf) => {
            match rf.to_typed_value() {
                Ok(data) => data,
                Err(msg) => panic!(msg)
            }
        },
        _ => panic!("TODO object_to_string {:?}", object)
    }
}

pub(crate) fn ref_to_char(code: u32) -> char {
    char::from_u32(code).unwrap()
}