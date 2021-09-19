use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use crate::values::{QName, QNameResolved};
use crate::fns::Param;
use crate::parser::op::{Expr, Representation};
use crate::parser::errors::ErrorCode;
use ordered_float::OrderedFloat;
use bigdecimal::BigDecimal;
use crate::eval::helpers::sort_and_dedup;
use chrono::{NaiveTime, TimeZone, DateTime, Date, FixedOffset, Local, Utc, Timelike};
use num_integer::div_mod_floor;
use chrono::format::{DelayedFormat, StrftimeItems, Item};
use std::borrow::Borrow;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone, Hash)]
pub enum Type {
    Untyped(String),

    DateTime(DateTime<FixedOffset>),
    dateTimeStamp(),

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

    gYearMonth(),
    gYear(),
    gMonthDay(),
    gDay(),
    gMonth(),

    // TODO CharRef { representation: Representation, reference: u32 }, ?
    String(String),
    NormalizedString(String),
    Token(String),
    language(String),
    NMTOKEN(String),
    Name(String),
    NCName(String),
    ID(String),
    IDREF(String),
    ENTITY(String),

    Boolean(bool),
    base64Binary(),
    hexBinary(),
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
    time: NaiveTime,
    offset: Tz::Offset,
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

pub(crate) fn type_to_int(t: Type) -> i128 {
    match t {
        Type::Integer(num) => num,
        _ => panic!("can't convert to int {:?}", t)
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
    Node { sequence: isize, name: QName, attributes: Vec<Node>, children: Vec<Node> },
    Attribute { sequence: isize, name: QName, value: String },
    NodeText { sequence: isize, content: String },
    NodeComment { sequence: isize, content: String },
    NodePI { sequence: isize, target: QName, content: String },
}

fn node_to_number(node: &Node) -> &isize {
    match node {
        Node::Node { sequence, .. } => sequence,
        Node::Attribute { sequence, .. } => sequence,
        Node::NodeText { sequence, .. } => sequence,
        Node::NodeComment { sequence, .. } => sequence,
        Node::NodePI { sequence, .. } => sequence,
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
            Node:: NodePI { target, content, .. } => {
                write!(f, "<?")?;
                write(f, target);
                write!(f, "{:?}?>", content)?;
            },
            Node:: NodeComment { content, ..} => {
                write!(f, "<!--{}-->", content)?;
            },
            Node:: NodeText { content, ..} => {
                write!(f, "{}", content)?;
            },
            Node:: Attribute { name, value, .. } => {
                write!(f, "@")?;
                write(f, name);
                write!(f, "={:?}", value)?;
            },
            Node::Node { name, attributes, children, .. } => {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Object {
    // workaround
    ForBinding { name: QNameResolved, values: Box<Object> },
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

    Function { parameters: Vec<Param>, body: Box<Expr> },
    FunctionRef { name: QNameResolved, arity: usize },

    Return(Box<Object>),
}

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

impl PartialOrd for Object {
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
            typed_value_of_node(node, &mut result);
            let str = result.join("");
            Ok(Object::Atomic(Type::Untyped(str)))
        },
        Object::Array(items) => atomization_of_vec(items),
        Object::Sequence(items) => atomization_of_vec(items),
        Object::Empty => Ok(obj), // or it can be XPST0005?
        _ => todo!()
    }
}

pub(crate) fn typed_value_of_node(node: Node, result: &mut Vec<String>) {
    match node {
        Node::Node { children, .. } => {
            for child in children {
                typed_value_of_node(child, result);
            }
        }
        Node::Attribute { value, .. } => {
            // Object::Atomic(Type::Untyped(value))
            result.push(value)
        }
        Node::NodeText { content, .. } => {
            // result.push(Object::Atomic(Type::Untyped(content)))
            result.push(content)
        }
        Node::NodeComment { content, .. } => {
            // result.push(Object::Atomic(Type::String(content)))
            result.push(content)
        }
        Node::NodePI { content, .. } => {
            // result.push(Object::Atomic(Type::String(content)))
            result.push(content)
        }
    }
}