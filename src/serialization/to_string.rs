use crate::eval::{Object, Type, Node, NumberCase};
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
        Object::Atomic(Type::String(str)) => str.clone(),
        Object::Atomic(Type::Integer(num)) => {
            num.to_string()
        },
        Object::Atomic(Type::Decimal { number, case }) |
        Object::Atomic(Type::Float { number, case }) |
        Object::Atomic(Type::Double { number, case }) => {
            match case {
                NumberCase::Normal => {
                    if let Some(num) = number {
                        num.to_string()
                    } else {
                        panic!("internal error")
                    }
                }
                NumberCase::NaN => String::from("NaN"),
                NumberCase::PlusInfinity => String::from("INF"),
                NumberCase::MinusInfinity => String::from("-INF"),
            }
        },
        Object::Sequence(items) => {
            let mut result = String::new();
            for item in items {
                let str = object_to_string(item);
                result.push_str(str.as_str());
            }
            result
        },
        Object::Node(node) => {
            node_to_string(node)
        }
        _ => panic!("TODO object_to_string {:?}", object)
    }
}

pub(crate) fn ref_to_string(representation: Representation, code: u32) -> String {
     match representation {
        Representation::Hexadecimal => { format!("&#x{:X};", code) }
        Representation::Decimal => { format!("&#{};", code) }
    }
}

pub(crate) fn ref_to_char(code: u32) -> char {
    char::from_u32(code).unwrap()
}