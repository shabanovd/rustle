use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use crate::values::{QName, QNameResolved};
use crate::fns::Param;
use crate::parser::op::{Representation};
use crate::parser::errors::ErrorCode;
use ordered_float::OrderedFloat;
use bigdecimal::BigDecimal;
use crate::eval::helpers::sort_and_dedup;
use chrono::{NaiveTime, TimeZone, DateTime, Date, FixedOffset, Local, Timelike};
use num_integer::div_mod_floor;
use chrono::format::{DelayedFormat, StrftimeItems, Item};
use std::borrow::Borrow;
use crate::eval::expression::Expression;
use std::fmt::{Debug, Formatter};

#[allow(dead_code)]
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone, Hash)]
pub enum Type {
    Untyped(String),

    DateTime(DateTime<FixedOffset>),
    DateTimeStamp(),

    Date(Date<FixedOffset>),
    Time(Time<FixedOffset>),

    Duration { positive: bool, years: u32, months: u32, days: u32, hours: u32, minutes: u32, seconds: u32, microseconds: u32 },
    YearMonthDuration  { positive: bool, years: u32, months: u32 },
    DayTimeDuration { positive: bool, days: u32, hours: u32, minutes: u32, seconds: u32, microseconds: u32 },

    Integer(i128),
    Decimal(BigDecimal),
    Float(OrderedFloat<f32>),
    Double(OrderedFloat<f64>),

    // nonPositiveInteger(),
    // negativeInteger(),
    // long(),
    // int(),
    // short(),
    // byte(),

    // nonNegativeInteger(),
    // unsignedLong(),
    // unsignedInt(),
    // unsignedShort(),
    // unsignedByte(),

    // positiveInteger(),

    GYearMonth(),
    GYear(),
    GMonthDay(),
    GDay(),
    GMonth(),

    // TODO CharRef { representation: Representation, reference: u32 }, ?
    String(String),
    NormalizedString(String),
    Token(String),
    Language(String),
    NMTOKEN(String),
    Name(String),
    NCName(String),
    ID(String),
    IDREF(String),
    ENTITY(String),

    Boolean(bool),
    Base64Binary(),
    HexBinary(),
    AnyURI(String),
    QName { url: Option<String>, prefix: Option<String>, local_part: String },
    NOTATION(),
}

impl PartialOrd for Time<FixedOffset> {
    fn partial_cmp(&self, other: &Time<FixedOffset>) -> Option<Ordering> {
        self.time.partial_cmp(&other.time)
    }
}

impl Ord for Time<FixedOffset> {
    fn cmp(&self, other: &Time<FixedOffset>) -> Ordering {
        self.time.cmp(&other.time)
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct Time<Tz: TimeZone> {
    pub time: NaiveTime,
    pub offset: Tz::Offset,
}

impl Time<FixedOffset> {
    #[inline]
    pub fn now() -> Time<FixedOffset> {
        let now = Local::now();
        Time { time: now.time(), offset: TimeZone::from_offset(now.offset()) }
    }

    #[inline]
    pub fn from(dt: DateTime<Local>) -> Time<FixedOffset> {
        Time { time: dt.time(), offset: TimeZone::from_offset(dt.offset()) }
    }

    #[inline]
    pub fn from_utc(time: NaiveTime) -> Time<FixedOffset> {
        Time { time, offset: FixedOffset::east(0) }
    }

    pub fn hms(&self) -> (u32, u32, u32, u32) {
        let (mins, sec) = div_mod_floor(self.time.num_seconds_from_midnight(), 60);
        let (hour, min) = div_mod_floor(mins, 60);
        (hour, min, sec, 0)
    }

    #[inline]
    pub fn format<'a>(&self, fmt: &'a str) -> DelayedFormat<StrftimeItems<'a>> {
        self.format_with_items(StrftimeItems::new(fmt))
    }

    #[inline]
    pub fn format_with_items<'a, I, B>(&self, items: I) -> DelayedFormat<I>
        where
            I: Iterator<Item = B> + Clone,
            B: Borrow<Item<'a>>,
    {
        DelayedFormat::new_with_offset(None, Some(self.time), &self.offset, items)
    }
}

pub(crate) fn object_to_qname(t: Object) -> QName {
    match t {
        Object::Atomic(Type::QName { prefix, url, local_part }) =>
                       QName { prefix, url, local_part },
        _ => panic!("can't convert to QName {:?}", t)
    }
}

pub fn string_to_double(string: &String) -> Result<Object, ErrorCode> {
    match string.trim().parse() {
        Ok(number) => {
            Ok(Object::Atomic(Type::Double(number)))
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

#[derive(Eq, PartialEq, Clone, Hash)]
pub enum Node {
    Document { sequence: isize, children: Vec<Node> },
    Element { sequence: isize, name: QName, attributes: Vec<Node>, children: Vec<Node> },
    Attribute { sequence: isize, name: QName, value: String },
    Text { sequence: isize, content: String },
    Comment { sequence: isize, content: String },
    PI { sequence: isize, target: QName, content: String },
}

fn node_to_number(node: &Node) -> &isize {
    match node {
        Node::Document { sequence, .. } => sequence,
        Node::Element { sequence, .. } => sequence,
        Node::Attribute { sequence, .. } => sequence,
        Node::Text { sequence, .. } => sequence,
        Node::Comment { sequence, .. } => sequence,
        Node::PI { sequence, .. } => sequence,
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        node_to_number(self).cmp(node_to_number(other))
    }
}

impl PartialOrd<Self> for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        node_to_number(self).partial_cmp(node_to_number(other))
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let write = |f: &mut fmt::Formatter, qname: &QName| {
            if let Some(prefix) = &qname.prefix {
                write!(f, "{}:", prefix).unwrap();
            }
            write!(f, "{}", qname.local_part).unwrap();
        };

        match self {
            Node::Document { children, .. } => {
                for child in children {
                    write!(f, "{:?}", child)?;
                }
            },
            Node::PI { target, content, .. } => {
                write!(f, "<?")?;
                write(f, target);
                write!(f, "{:?}?>", content)?;
            },
            Node::Comment { content, ..} => {
                write!(f, "<!--{}-->", content)?;
            },
            Node::Text { content, ..} => {
                write!(f, "{}", content)?;
            },
            Node:: Attribute { name, value, .. } => {
                write!(f, "@")?;
                write(f, name);
                write!(f, "={:?}", value)?;
            },
            Node::Element { name, attributes, children, .. } => {
                write!(f, "<")?;

                write(f, name);

                if attributes.len() > 0 {
                    for attribute in attributes {
                        println!("attribute {:?}", attribute);
                        match attribute {
                            Node::Attribute { sequence, name, value } => {
                                write!(f, " ")?;
                                write(f, name);
                                write!(f, "={}", value)?;
                            },
                            _ => panic!("unexpected")
                        }
                    }
                }

                if children.len() == 0 {
                    write!(f, "/>")?;
                } else {
                    write!(f, ">").unwrap();
                    for child in children {
                        write!(f, "{:?}", child)?;
                    }
                    write!(f, "</")?;
                    write(f, name);
                }
            }
        }
        write!(f, "")
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

    Atomic(Type),
    Node(Node),

    Array(Vec<Object>),
    Map(HashMap<Type, Object>),

    Function { parameters: Vec<Param>, body: Box<dyn Expression> },
    FunctionRef { name: QNameResolved, arity: usize },

    Return(Box<Object>),
}

impl PartialEq<Self> for Object {
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
                    _ => Some(Ordering::Greater),
                }
            },
            Object::Node(n1) => {
                match other {
                    Object::Node(n2) => {
                        n1.partial_cmp(n2)
                    },
                    _ => Some(Ordering::Less),
                }
            },
            _ => Some(Ordering::Less)
        }
    }
}

fn zero_or_one(items: &mut Vec<Object>) -> Result<Object, crate::parser::errors::ErrorCode> {
    sort_and_dedup(items);
    if items.len() == 1 {
        Ok(items.remove(0))
    } else if items.len() == 0 {
        Ok(Object::Empty)
    } else {
        Err(ErrorCode::XPTY0004)
    }
}

pub fn atomization_of_vec(items: Vec<Object>) -> Result<Object, ErrorCode> {
    let mut result = Vec::with_capacity(items.len());
    for item in items {
        let value = atomization(item)?;
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

pub(crate) fn atomization(obj: Object) -> Result<Object, ErrorCode> {
    match obj {
        Object::Atomic(..) => Ok(obj),
        Object::Node(node) => {
            let mut result = vec![];
            let t = typed_value_of_node(node, &mut result);
            let str = result.join("");
            Ok(Object::Atomic(Type::Untyped(str)))
        },
        Object::Array(items) => atomization_of_vec(items),
        Object::Sequence(items) => atomization_of_vec(items),
        Object::Range { min, max } => {
            if min == max {
                Ok(Object::Atomic(Type::Integer(min)))
            } else {
                Err(ErrorCode::XPTY0004)
            }
        },
        Object::Empty => Ok(obj), // or it can be XPST0005?
        _ => todo!()
    }
}

pub(crate) fn sequence_atomization(obj: Object) -> Result<Object, ErrorCode> {
    match obj {
        Object::Range { .. } |
        Object::Array(..) |
        Object::Sequence(..) |
        Object::Atomic(..) => Ok(obj),
        Object::Node(node) => {
            let mut result = vec![];
            let t = typed_value_of_node(node, &mut result);
            let str = result.join("");
            Ok(Object::Atomic(Type::Untyped(str)))
        },
        Object::Empty => Ok(obj), // or it can be XPST0005?
        _ => todo!()
    }
}

pub(crate) enum Value {
    Typed,
    String,
    Absent,
    UntypedAtomic,
}

pub(crate) fn typed_value_of_node(node: Node, result: &mut Vec<String>) -> Value {
    match node {
        Node::Document { children, .. } => {
            for child in children {
                typed_value_of_node(child, result);
            }
            Value::Typed
        }
        Node::Element { children, .. } => {
            for child in children {
                typed_value_of_node(child, result);
            }
            Value::Typed
        }
        Node::Attribute { value, .. } => {
            // Object::Atomic(Type::Untyped(value))
            result.push(value);
            Value::Typed
        }
        Node::Text { content, .. } => {
            // result.push(Object::Atomic(Type::Untyped(content)))
            result.push(content);
            Value::UntypedAtomic
        }
        Node::Comment { content, .. } => {
            // result.push(Object::Atomic(Type::String(content)))
            result.push(content);
            Value::String
        }
        Node::PI { content, .. } => {
            // result.push(Object::Atomic(Type::String(content)))
            result.push(content);
            Value::String
        }
    }
}