use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use crate::values::{Double, Integer, QName, QNameResolved, Str, Untyped};
use crate::fns::Param;
use crate::parser::op::{Representation};
use crate::parser::errors::ErrorCode;
use bigdecimal::BigDecimal;
use crate::eval::helpers::sort_and_dedup;
use crate::eval::expression::Expression;
use std::fmt::{Debug, Formatter};
use crate::eval::{Environment, ErrorInfo};
use crate::tree::Reference;
use crate::values::Value;

pub(crate) fn object_to_qname(t: Object) -> QName {
    match t {
        Object::Atomic(Str(str)) => {
            if str.contains(":") {
                todo!()
            }
            QName::local_part(str.as_str())
        }
        Object::Atomic(QName { prefix, url, local_part }) =>
                       QName { prefix, url, local_part },
        _ => panic!("can't convert to QName {:?}", t)
    }
}

pub fn string_to_double(string: &String) -> Result<Object, ErrorCode> {
    match string.trim().parse() {
        Ok(number) => {
            Ok(Object::Atomic(Double::boxed(number)))
        },
        Err(..) => Err(ErrorCode::FORG0001)
    }
}

pub fn string_to_decimal(string: &String) -> Result<BigDecimal, ErrorCode> {
    match string.trim().parse() {
        Ok(num) => Ok(num),
        Err(..) => Err(ErrorCode::FORG0001)
    }
}

#[derive(Clone)]
pub enum Object {
    Range { min: i128, max: i128 },

    Error { code: String },
    CharRef { representation: Representation, reference: u32 },
    EntityRef(String),

    Nothing,

    Empty,
    Sequence(Vec<Object>),

    Atomic(Box<dyn Value>),
    Node(Reference),

    Array(Vec<Object>),
    Map(HashMap<Box<dyn Value>, Object>),

    Function { parameters: Vec<Param>, body: Box<dyn Expression> },
    FunctionRef { name: QNameResolved, arity: usize },

    Return(Box<Object>),
}

impl Object {
    pub(crate) fn atomic(value: dyn Value) -> Object {
        Object::Atomic(Box::new(value))
    }
}

impl<'a> PartialEq<Self> for Object {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Object::Range { min: l_min, max: l_max } => {
                match other {
                    Object::Range { min: r_min, max: r_max } => l_min == r_min && l_max == r_max,
                    _ => false
                }
            }
            Object::Error { code: l_code } => {
                match other {
                    Object::Error { code: r_code } => l_code == r_code,
                    _ => false
                }
            }
            Object::CharRef { representation: l_representation, reference: l_reference } => {
                match other {
                    Object::CharRef { representation: r_representation, reference: r_reference } =>
                        l_representation == r_representation && l_reference == r_reference,
                    _ => false
                }
            }
            Object::EntityRef(l_ref) => {
                match other {
                    Object::EntityRef(r_ref) => l_ref == r_ref,
                    _ => false
                }
            }
            Object::Nothing => {
                match other {
                    Object::Nothing => true,
                    _ => false
                }
            }
            Object::Empty => {
                match other {
                    Object::Empty => true,
                    _ => false
                }
            }
            Object::Sequence(l_items) => {
                match other {
                    Object::Sequence(r_items) => l_items == r_items,
                    _ => false
                }
            }
            Object::Atomic(l_type) => {
                match other {
                    Object::Atomic(r_type) => l_type == r_type,
                    _ => false
                }
            }
            Object::Node(l_node) => {
                match other {
                    Object::Node(r_node) => l_node == r_node,
                    _ => false
                }
            }
            Object::Array(l_items) => {
                match other {
                    Object::Array(r_items) => l_items == r_items,
                    _ => false
                }
            }
            Object::Map(l_entries) => {
                match other {
                    Object::Map(r_entries) => l_entries == r_entries,
                    _ => false
                }
            }
            Object::Function { .. } => {
                false
            }
            Object::FunctionRef { name: l_name, arity: l_arity } => {
                match other {
                    Object::FunctionRef { name: r_name, arity: r_arity } => l_name == r_name && l_arity == r_arity,
                    _ => false
                }
            }
            Object::Return(l_item) => {
                match other {
                    Object::Return(r_item) => l_item == r_item,
                    _ => false
                }
            }
        }
    }
}

impl Debug for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Object::Range { min, max } => {
                f.debug_tuple("Range")
                    .field(min)
                    .field(max)
                    .finish()
            }
            Object::Error { code } => {
                f.debug_tuple("Error")
                    .field(code)
                    .finish()
            }
            Object::CharRef { representation, reference } => {
                let data = match representation {
                    Representation::Hexadecimal => { format!("&#x{:X};", reference) }
                    Representation::Decimal => { format!("&#{};", reference) }
                };

                f.debug_tuple("CharRef")
                    .field(&data)
                    .finish()
            }
            Object::EntityRef(code) => {
                f.debug_tuple("EntityRef")
                    .field(code)
                    .finish()
            }
            Object::Nothing => f.debug_struct("Nothing").finish(),
            Object::Empty => f.debug_struct("Empty").finish(),
            Object::Sequence(items) => {
                f.debug_tuple("Sequence")
                    .field(items)
                    .finish()
            }
            Object::Atomic(t) => {
                f.debug_tuple("Atomic")
                    .field(t)
                    .finish()
            }
            Object::Node(node) => {
                f.debug_tuple("Node")
                    .field(node)
                    .finish()
            }
            Object::Array(items) => {
                f.debug_tuple("Array")
                    .field(items)
                    .finish()
            }
            Object::Map(entries) => {
                f.debug_tuple("Map")
                    .field(entries)
                    .finish()
            }
            Object::Function { parameters, .. } => {
                f.debug_tuple("Function")
                    .field(parameters)
                    .finish()
            }
            Object::FunctionRef { name, arity } => {
                f.debug_struct("FunctionRef")
                    .field("name", name)
                    .field("arity", arity)
                    .finish_non_exhaustive()
            }
            Object::Return(_) => {
                f.debug_struct("Return").finish_non_exhaustive()
            }
        }
    }
}

impl Eq for Object {}

impl Ord for Object {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            Object::Atomic(t1) => {
                match other {
                    Object::Atomic(t2) => {
                        t1.cmp(t2)
                    },
                    _ => Ordering::Greater,
                }
            },
            Object::Node(n1) => {
                match other {
                    Object::Node(n2) => {
                        n1.cmp(n2)
                    },
                    _ => Ordering::Less,
                }
            },
            _ => Ordering::Less
        }
    }
}

impl PartialOrd<Self> for Object {
    fn partial_cmp(&self, other: &Object) -> Option<Ordering> {
        match self {
            Object::Atomic(t1) => {
                match other {
                    Object::Atomic(t2) => {
                        t1.partial_cmp(t2)
                    },
                    _ => None,
                }
            },
            Object::Node(n1) => {
                match other {
                    Object::Node(n2) => {
                        Some(n1.cmp(n2))
                    },
                    _ => None,
                }
            },
            _ => None
        }
    }
}

fn zero_or_one(items: &mut Vec<Object>) -> Result<Object, ErrorInfo> {
    sort_and_dedup(items);
    if items.len() == 1 {
        Ok(items.remove(0))
    } else if items.len() == 0 {
        Ok(Object::Empty)
    } else {
        Err((ErrorCode::XPTY0004, String::from("TODO")))
    }
}

pub fn atomization_of_vec(env: &Box<Environment>, items: Vec<Object>) -> Result<Object, ErrorInfo> {
    let mut result = Vec::with_capacity(items.len());
    for item in items {
        let value = atomization(env, item)?;
        match value {
            Object::Sequence(elements) => {
                for el in elements {
                    result.push(el);
                }
            }
            _ => result.push(value)
        }
    }
    zero_or_one(&mut result)
}

pub(crate) fn atomization(env: &Box<Environment>, obj: Object) -> Result<Object, ErrorInfo> {
    match obj {
        Object::Atomic(..) => Ok(obj),
        Object::Node(rf) => {
            match rf.to_typed_value() {
                Ok(data) => Ok(Object::Atomic(Untyped::boxed(data))),
                Err(msg) => Err((ErrorCode::TODO, msg))
            }

        },
        Object::Array(items) => atomization_of_vec(env, items),
        Object::Sequence(items) => atomization_of_vec(env, items),
        Object::Range { min, max } => {
            if min == max {
                Ok(Object::Atomic(Integer::boxed(min)))
            } else {
                Err((ErrorCode::XPTY0004, String::from("TODO")))
            }
        },
        Object::Empty => Ok(obj), // or it can be XPST0005?
        Object::Function { .. } |
        Object::FunctionRef { .. } |
        Object::Map(..) => Err((ErrorCode::FOTY0013, String::from("TODO"))),
        _ => todo!()
    }
}

pub(crate) fn sequence_atomization(env: &Box<Environment>, obj: Object) -> Result<Object, ErrorInfo> {
    match obj {
        Object::Range { .. } |
        Object::Array(..) |
        Object::Sequence(..) |
        Object::Atomic(..) => Ok(obj),
        Object::Node(rf) => {
            match rf.to_typed_value() {
                Ok(data) => Ok(Object::Atomic(Untyped::new(data))),
                Err(msg) => Err((ErrorCode::TODO, msg))
            }
        },
        Object::Empty => Ok(obj), // or it can be XPST0005?
        _ => todo!()
    }
}

pub(crate) enum ValueType {
    Typed,
    String,
    Absent,
    UntypedAtomic,
}