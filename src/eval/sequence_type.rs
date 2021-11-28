use crate::eval::{Environment, ErrorInfo, Object, Type};
use crate::values::*;
use crate::eval::expression::{NodeTest, Expression};
use crate::tree::Reference;
use crate::namespaces::{Namespace, SCHEMA};
use std::collections::HashMap;
use bigdecimal::Zero;
use nom::bytes::complete::is_a;
use crate::parser::errors::ErrorCode;

pub const XS_ANY_SIMPLE_TYPE: QN = QN::full("xs", "anySimpleType", SCHEMA.uri);
pub const XS_ANY_ATOMIC_TYPE: QN = QN::full("xs", "anyAtomicType", SCHEMA.uri);
pub const XS_STRING: QN = QN::full("xs", "string", SCHEMA.uri);
pub const XS_BOOLEAN: QN = QN::full("xs", "boolean", SCHEMA.uri);

pub const XS_NUMERIC: QN = QN::full("xs", "numeric", SCHEMA.uri);

pub const XS_INTEGER: QN = QN::full("xs", "integer", SCHEMA.uri);
pub const XS_NON_POSITIVE_INTEGER: QN = QN::full("xs", "nonPositiveInteger", SCHEMA.uri);
pub const XS_NEGATIVE_INTEGER: QN = QN::full("xs", "negativeInteger", SCHEMA.uri);
pub const XS_LONG: QN = QN::full("xs", "long", SCHEMA.uri);
pub const XS_INT: QN = QN::full("xs", "int", SCHEMA.uri);
pub const XS_SHORT: QN = QN::full("xs", "short", SCHEMA.uri);
pub const XS_BYTE: QN = QN::full("xs", "byte", SCHEMA.uri);
pub const XS_NON_NEGATIVE_INTEGER: QN = QN::full("xs", "nonNegativeInteger", SCHEMA.uri);
pub const XS_UNSIGNED_LONG: QN = QN::full("xs", "unsignedLong", SCHEMA.uri);
pub const XS_UNSIGNED_INT: QN = QN::full("xs", "unsignedInt", SCHEMA.uri);
pub const XS_UNSIGNED_SHORT: QN = QN::full("xs", "unsignedShort", SCHEMA.uri);
pub const XS_UNSIGNED_BYTE: QN = QN::full("xs", "unsignedByte", SCHEMA.uri);
pub const XS_POSITIVE_INTEGER: QN = QN::full("xs", "positiveInteger", SCHEMA.uri);

pub const XS_DECIMAL: QN = QN::full("xs", "decimal", SCHEMA.uri);
pub const XS_FLOAT: QN = QN::full("xs", "float", SCHEMA.uri);
pub const XS_DOUBLE: QN = QN::full("xs", "double", SCHEMA.uri);

pub const XS_DATE_TIME: QN = QN::full("xs", "dateTime", SCHEMA.uri);
pub const XS_TIME: QN = QN::full("xs", "time", SCHEMA.uri);
pub const XS_DATE: QN = QN::full("xs", "date", SCHEMA.uri);

pub const XS_G_YEAR_MONTH: QN = QN::full("xs", "gYearMonth", SCHEMA.uri);
pub const XS_G_YEAR: QN = QN::full("xs", "gYear", SCHEMA.uri);
pub const XS_G_MONTH_DAY: QN = QN::full("xs", "gMonthDay", SCHEMA.uri);
pub const XS_G_DAY: QN = QN::full("xs", "gDay", SCHEMA.uri);
pub const XS_G_MONTH: QN = QN::full("xs", "gMonth", SCHEMA.uri);

pub const XS_HEX_BINARY: QN = QN::full("xs", "hexBinary", SCHEMA.uri);
pub const XS_BASE64_BINARY: QN = QN::full("xs", "base64Binary", SCHEMA.uri);

pub const XS_ANY_URI: QN = QN::full("xs", "anyURI", SCHEMA.uri);

pub const XS_QNAME: QN = QN::full("xs", "QName", SCHEMA.uri);

pub const XS_NORMALIZED_STRING: QN = QN::full("xs", "normalizedString", SCHEMA.uri);
pub const XS_TOKEN: QN = QN::full("xs", "token", SCHEMA.uri);
pub const XS_LANGUAGE: QN = QN::full("xs", "language", SCHEMA.uri);
pub const XS_NMTOKEN: QN = QN::full("xs", "NMTOKEN", SCHEMA.uri);
pub const XS_NAME: QN = QN::full("xs", "Name", SCHEMA.uri);
pub const XS_NCNAME: QN = QN::full("xs", "NCName", SCHEMA.uri);
pub const XS_NOTATION: QN = QN::full("xs", "NOTATION", SCHEMA.uri);

pub const XS_DURATION: QN = QN::full("xs", "duration", SCHEMA.uri);
pub const XS_YEAR_MONTH_DURATION: QN = QN::full("xs", "yearMonthDuration", SCHEMA.uri);
pub const XS_DAY_TIME_DURATION: QN = QN::full("xs", "dayTimeDuration", SCHEMA.uri);
pub const XS_DATE_TIME_STAMP: QN = QN::full("xs", "dateTimeStamp", SCHEMA.uri);

pub const XS_UNTYPED: QN = QN::full("xs", "untyped", SCHEMA.uri);
pub const XS_UNTYPED_ATOMIC: QN = QN::full("xs", "untypedAtomic", SCHEMA.uri);

lazy_static! {
    pub static ref QNameToTypes: HashMap<QNameResolved, Types> = {
        let mut map: HashMap<QNameResolved, Types> = HashMap::new();

        for (qn, t) in [
            (XS_UNTYPED_ATOMIC, Types::Untyped),
            (XS_STRING, Types::String),
            (XS_NORMALIZED_STRING, Types::NormalizedString),

            (XS_BOOLEAN, Types::Boolean),

            (XS_ANY_URI, Types::AnyURI),

            (XS_NUMERIC, Types::Numeric),
            (XS_INTEGER, Types::Integer),
            (XS_DECIMAL, Types::Decimal),
            (XS_FLOAT, Types::Float),
            (XS_DOUBLE, Types::Double),

            (XS_NON_POSITIVE_INTEGER, Types::Integer),
            (XS_NEGATIVE_INTEGER, Types::Integer), // TODO negativeInteger,
            (XS_LONG, Types::Integer), // TODO long,
            (XS_INT, Types::Integer), // TODO int,
            (XS_SHORT, Types::Integer), // TODO short,
            (XS_BYTE, Types::Integer), // TODO byte,

            (XS_NON_NEGATIVE_INTEGER, Types::Integer), // TODO nonNegativeInteger,
            (XS_UNSIGNED_LONG, Types::Integer), // TODO unsignedLong,
            (XS_UNSIGNED_INT, Types::Integer), // TODO unsignedInt,
            (XS_UNSIGNED_SHORT, Types::Integer), // TODO unsignedShort,
            (XS_UNSIGNED_BYTE, Types::Integer), // TODO unsignedByte,

            (XS_POSITIVE_INTEGER, Types::Integer), // TODO positiveInteger,

            (XS_DATE_TIME, Types::DateTime),
            (XS_DATE_TIME_STAMP, Types::DateTimeStamp),

            (XS_DATE, Types::Date),
            (XS_TIME, Types::Time),

            (XS_DURATION, Types::Duration),
            (XS_YEAR_MONTH_DURATION, Types::YearMonthDuration),
            (XS_DAY_TIME_DURATION, Types::DayTimeDuration),

            (XS_G_YEAR_MONTH, Types::GYearMonth),
            (XS_G_YEAR, Types::GYear),
            (XS_G_MONTH_DAY, Types::GMonthDay),
            (XS_G_DAY, Types::GDay),
            (XS_G_MONTH, Types::GMonth),

            (XS_NAME, Types::Name),
            (XS_NCNAME, Types::NCName),
            (XS_QNAME, Types::QName),

            (XS_TOKEN, Types::Token),
            (XS_LANGUAGE, Types::Language),
            (XS_NMTOKEN, Types::NMTOKEN),

            // TODO (XS_ID, Types::ID),
            // TODO (XS_IDREF, Types::IDREF),
            // TODO (XS_ENTITY, Types::ENTITY),

            (XS_BASE64_BINARY, Types::Base64Binary),
            (XS_HEX_BINARY, Types::HexBinary),

            // TODO (XS_NOTATION, Types::NOTATION),
        ] {
            map.insert(qn.into(), t);
        }

        map
    };
}

pub(crate) fn FN_ARRAY() -> (Vec<SequenceType>, SequenceType) {
    (
        [SequenceType::exactly_one(ItemType::AtomicOrUnionType(XS_INTEGER.into()))].to_vec(),
        SequenceType::zero_or_more(ItemType::Item)
    )
}

#[derive(Debug, Clone)]
pub enum ItemType {
    None,
    SequenceEmpty,
    Item,
    AnyAtomicType,
    AtomicOrUnionType(QName),

    AnyKind,
    Array(Option<Box<SequenceType>>),
    Map(Option<(Box<SequenceType>, Box<SequenceType>)>),
    Node(Box<dyn NodeTest>),
    // Document(Option<Box<ItemType>>),
    // Element,
    // Attribute,
    // Text,
    // Comment,
    // NamespaceNode,
    // PI,

    SchemaAttribute(QName),

    Function { args: Option<Vec<SequenceType>>, st: Option<Box<SequenceType>> }
}

impl ItemType {
    pub(crate) fn node() -> Self {
        ItemType::Node(AnyKindTest::boxed())
    }

    pub(crate) fn element() -> Self {
        ItemType::Node(ElementTest::boxed(None, None))
    }

    pub(crate) fn element_ns<N, S>(nc: &N, local_part: S) -> Self
        where N: Namespace, S: Into<String>
    {
        ItemType::Node(ElementTest::boxed(Some(QName { prefix: Some(nc.prefix()), url: Some(nc.uri()), local_part: local_part.into() }), None))
    }

    pub(crate) fn is_same(&self, env: &Environment, right: &ItemType) -> bool {
        match self {
            ItemType::None => {
                match right {
                    ItemType::None => true,
                    _ => false
                }
            }
            ItemType::SequenceEmpty => {
                match right {
                    ItemType::SequenceEmpty => true,
                    _ => false
                }
            }
            ItemType::Item => {
                match right {
                    ItemType::Item => true,
                    _ => false
                }
            }
            ItemType::AnyAtomicType => {
                match right {
                    ItemType::AnyAtomicType => true,
                    _ => false
                }
            }
            ItemType::AtomicOrUnionType(l_name) => {
                match right {
                    ItemType::AtomicOrUnionType(r_name) => {
                        let l_name = env.namespaces.resolve(l_name);
                        let r_name = env.namespaces.resolve(r_name);
                        l_name == r_name
                    },
                    _ => false
                }
            }
            ItemType::AnyKind => {
                match right {
                    ItemType::AnyKind => true,
                    _ => false
                }
            }
            ItemType::Array(l_st) => {
                match right {
                    ItemType::Array(r_st) => {
                        if let Some(l_st) = l_st {
                            if let Some(r_st) = r_st {
                                l_st.is_same(env, r_st)
                            } else {
                                false
                            }
                        } else {
                            r_st.is_none()
                        }
                    },
                    _ => false
                }
            }
            ItemType::Map(l_st) => {
                match right {
                    ItemType::Map(r_st) => {
                        if let Some((l_k, l_v)) = l_st {
                            if let Some((r_k, r_v)) = r_st {
                                l_k.is_same(env, r_k) && l_v.is_same(env, r_v)
                            } else {
                                false
                            }
                        } else {
                            r_st.is_none()
                        }
                    },
                    _ => false
                }
            }
            ItemType::Node(_) => {
                match right {
                    ItemType::Node(_) => todo!(),
                    _ => false
                }
            }
            ItemType::SchemaAttribute(_) => {
                match right {
                    ItemType::SchemaAttribute(_) => todo!(),
                    _ => false
                }
            }
            ItemType::Function { .. } => {
                match right {
                    ItemType::Function { .. } => todo!(),
                    _ => false
                }
            }
        }

    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OccurrenceIndicator {
    ExactlyOne,
    ZeroOrOne, // ?
    ZeroOrMore, // *
    OneOrMore, // +
}

#[derive(Debug, Clone)]
pub struct SequenceType {
    pub(crate) item_type: ItemType,
    pub(crate) occurrence_indicator: OccurrenceIndicator
}

impl SequenceType {
    pub(crate) const fn none() -> Self {
        SequenceType { item_type: ItemType::None, occurrence_indicator: OccurrenceIndicator::ExactlyOne }
    }

    pub(crate) const fn exactly_one(item_type: ItemType) -> Self {
        SequenceType { item_type, occurrence_indicator: OccurrenceIndicator::ExactlyOne }
    }

    pub(crate) const fn zero_or_one(item_type: ItemType) -> Self {
        SequenceType { item_type, occurrence_indicator: OccurrenceIndicator::ZeroOrOne }
    }

    pub(crate) const fn zero_or_more(item_type: ItemType) -> Self {
        SequenceType { item_type, occurrence_indicator: OccurrenceIndicator::ZeroOrMore }
    }

    pub(crate) const fn one_or_more(item_type: ItemType) -> Self {
        SequenceType { item_type, occurrence_indicator: OccurrenceIndicator::OneOrMore }
    }

    pub fn is_same(&self, env: &Environment, other: &SequenceType) -> bool {
        match &self.item_type {
            ItemType::None => {
                match &other.item_type {
                    ItemType::None => {
                        self.occurrence_indicator == other.occurrence_indicator
                    }
                    _ => false
                }
            }
            ItemType::AnyAtomicType => {
                match &other.item_type {
                    ItemType::AnyAtomicType => {
                        self.occurrence_indicator == other.occurrence_indicator
                    }
                    _ => false
                }
            }
            ItemType::SequenceEmpty => {
                match &other.item_type {
                    ItemType::SequenceEmpty => {
                        self.occurrence_indicator == other.occurrence_indicator
                    }
                    _ => false
                }
            }
            ItemType::Item => {
                match &other.item_type {
                    ItemType::Item => {
                        self.occurrence_indicator == other.occurrence_indicator
                    }
                    _ => false
                }
            }
            ItemType::AtomicOrUnionType(l_name) => {
                match &other.item_type {
                    ItemType::AtomicOrUnionType(r_name) => {
                        println!("{:?} vs {:?}", l_name, r_name);
                        env.namespaces.resolve(l_name) == env.namespaces.resolve(r_name) && self.occurrence_indicator == other.occurrence_indicator
                    }
                    _ => false
                }
            }
            ItemType::AnyKind => {
                match &other.item_type {
                    ItemType::AnyKind => {
                        self.occurrence_indicator == other.occurrence_indicator
                    }
                    _ => false
                }
            }
            ItemType::Map(l_st) => {
                match &other.item_type {
                    ItemType::Map(r_st) => {
                        if self.occurrence_indicator == other.occurrence_indicator {
                            if let Some((l_k, l_v)) = l_st {
                                if let Some((r_k, r_v)) = r_st {
                                    l_k.is_same(env, r_k) && l_v.is_same(env, r_v)
                                } else {
                                    false
                                }
                            } else {
                                r_st.is_none()
                            }
                        } else {
                            false
                        }
                    }
                    _ => false
                }
            }
            ItemType::Array(l_st) => {
                match &other.item_type {
                    ItemType::Array(r_st) => {
                        self.occurrence_indicator == other.occurrence_indicator
                    }
                    _ => false
                }
            }
            ItemType::Node(l_nt) => {
                match &other.item_type {
                    ItemType::Node(r_nt) => {
                        todo!()
                    }
                    _ => false
                }
            }
            ItemType::SchemaAttribute(l_name) => {
                match &other.item_type {
                    ItemType::SchemaAttribute(r_name) => {
                        env.namespaces.resolve(l_name) == env.namespaces.resolve(r_name) && self.occurrence_indicator == other.occurrence_indicator
                    }
                    _ => false
                }
            }
            ItemType::Function { args: l_args, st: l_st } => {
                match &other.item_type {
                    ItemType::Function { args: r_args, st: r_st } => {
                        // l_args == r_args && l_st == r_st && self.occurrence_indicator == other.occurrence_indicator
                        todo!()
                    }
                    _ => false
                }
            }
        }
    }

    pub fn is_not_same(&self, env: &Environment, other: &SequenceType) -> bool {
        !self.is_same(env, other)
    }

    pub fn cascade(&self, env: &Environment, obj: Object) -> Result<Object, ErrorInfo> {
        println!("cascade:\n st: {:#?}\n ob: {:#?}", self, obj);
        let is_array = false;
        let type_only = false;
        match &self.item_type {
            ItemType::Item => {
                match obj {
                    Object::Nothing => panic!("raise error?"),
                    Object::Empty => {
                        if type_only {
                            todo!()
                        } else {
                            if match self.occurrence_indicator {
                                OccurrenceIndicator::OneOrMore |
                                OccurrenceIndicator::ExactlyOne => false,
                                OccurrenceIndicator::ZeroOrOne |
                                OccurrenceIndicator::ZeroOrMore => true,
                            } {
                                Ok(Object::Empty)
                            } else {
                                panic!("raise error?")
                            }
                        }
                    }
                    Object::Range { min, max } => {
                        if type_only {
                            todo!()
                        } else {
                            if match self.occurrence_indicator {
                                OccurrenceIndicator::ExactlyOne |
                                OccurrenceIndicator::ZeroOrOne => min == max,
                                OccurrenceIndicator::ZeroOrMore |
                                OccurrenceIndicator::OneOrMore => true,
                            } {
                                Ok(obj)
                            } else {
                                panic!("raise error?")
                            }
                        }
                    }
                    Object::Atomic(_) |
                    Object::Node(_) |
                    Object::Array(_) |
                    Object::Map(_) => Ok(obj),
                    Object::Sequence(items) => {
                        if type_only {
                            todo!()
                        } else {
                            if match self.occurrence_indicator {
                                OccurrenceIndicator::ExactlyOne => items.len() == 1,
                                OccurrenceIndicator::ZeroOrOne => items.len() >= 0 && items.len() <= 1,
                                OccurrenceIndicator::ZeroOrMore => items.len() >= 0,
                                OccurrenceIndicator::OneOrMore => items.len() >= 1
                            } {
                                Ok(Object::Sequence(items))
                            } else {
                                panic!("raise error?")
                            }
                        }
                    }
                    Object::Error { .. } => todo!(),
                    Object::CharRef { .. } => todo!(),
                    Object::EntityRef(_) => todo!(),
                    Object::Function { .. } => todo!(),
                    Object::FunctionRef { .. } => todo!(),
                    Object::Return(_) => todo!(),
                }
            }
            ItemType::AtomicOrUnionType(original_qname) => {
                let name = env.namespaces.resolve(original_qname);
                if name.is_same_qn(&XS_NOTATION) || name.is_same_qn(&XS_ANY_ATOMIC_TYPE) || name.is_same_qn(&XS_ANY_SIMPLE_TYPE) {
                    return Err((ErrorCode::XPST0080, String::from("TODO")));
                } else if name.is_same_qn(&XS_ANY_ATOMIC_TYPE) {
                    match obj {
                        Object::Empty => {
                            if is_array {
                                Ok(obj)
                            } else if type_only {
                                todo!()
                            } else {
                                if match self.occurrence_indicator {
                                    OccurrenceIndicator::OneOrMore |
                                    OccurrenceIndicator::ExactlyOne => false,
                                    OccurrenceIndicator::ZeroOrOne |
                                    OccurrenceIndicator::ZeroOrMore => true,
                                } {
                                    Ok(obj)
                                } else {
                                    panic!("raise error?")
                                }
                            }
                        },
                        Object::Range { .. } |
                        Object::Atomic(_) => Ok(obj),
                        _ => todo!("{:?}", obj),
                    }
                } else {
                    match obj {
                        Object::Empty => {
                            if is_array {
                                Ok(obj)
                            } else if type_only {
                                todo!()
                            } else {
                                if match self.occurrence_indicator {
                                    OccurrenceIndicator::OneOrMore |
                                    OccurrenceIndicator::ExactlyOne => false,
                                    OccurrenceIndicator::ZeroOrOne |
                                    OccurrenceIndicator::ZeroOrMore => true,
                                } {
                                    Ok(obj)
                                } else {
                                    panic!("raise error?")
                                }
                            }
                        },
                        Object::Atomic(t) => {
                            if let Some(types) = QNameToTypes.get(&name) {
                                match t.convert(types.clone()) {
                                    Ok(t) => Ok(Object::Atomic(t)),
                                    Err(err) => Err(err)
                                }
                            } else {
                                todo!("handle custom types")
                            }
                        },
                        Object::Range { .. } => {
                            if match self.occurrence_indicator {
                                OccurrenceIndicator::ZeroOrOne |
                                OccurrenceIndicator::ExactlyOne => false,
                                OccurrenceIndicator::OneOrMore |
                                OccurrenceIndicator::ZeroOrMore => true,
                            } {
                                // name == XS_UNTYPED_ATOMIC || name == XS_STRING || name == XS_BOOLEAN
                                //     || name == XS_DOUBLE || name == XS_FLOAT || name == XS_DECIMAL || name == XS_INTEGER
                                todo!()
                            } else {
                                panic!("raise error?")
                            }
                        }
                        Object::Sequence(items) => {
                            if type_only || match self.occurrence_indicator {
                                OccurrenceIndicator::ZeroOrOne |
                                OccurrenceIndicator::ExactlyOne => false,
                                OccurrenceIndicator::OneOrMore |
                                OccurrenceIndicator::ZeroOrMore => true,
                            } {
                                let mut result = Vec::with_capacity(items.len());
                                for item in items {
                                    result.push(
                                        self.cascade(env, item)?
                                    );
                                }
                                Ok(Object::Sequence(result))
                            } else {
                                panic!("raise error?")
                            }
                        },
                        Object::Array(items) => {
                            if type_only {
                                todo!()
                            } else {
                                let mut result = Vec::with_capacity(items.len());
                                for item in items {
                                    result.push(
                                        self.cascade(env, item)?
                                    );
                                }
                                Ok(Object::Array(result))
                            }
                        }
                        Object::Map(items) => {
                            Err((ErrorCode::FOTY0013, String::from("TODO")))
                        }
                        Object::Node(rf) => {
                            let str = match rf.to_typed_value() {
                                Ok(str) => str,
                                Err(msg) => todo!("{}", msg)
                            };
                            self.cascade(env, Object::Atomic(Type::Untyped(str)))
                        }
                        _ => todo!("{:?}", obj),
                    }
                }
            },
            ItemType::Map(st) => {
                todo!()
            }
            ItemType::Array(st) => {
                match obj {
                    Object::Array(items) => {
                        if self.occurrence_indicator == OccurrenceIndicator::ExactlyOne {
                            if let Some(item_st) = st {
                                let mut result = Vec::with_capacity(items.len());
                                for item in items {
                                    result.push(
                                        item_st.cascade(env, item)?
                                    );
                                }
                                Ok(Object::Array(result))
                            } else {
                                Ok(Object::Array(items))
                            }
                        } else {
                            panic!("raise error?")
                        }
                    },
                    Object::Sequence(_) => {
                        todo!()
                    }
                    _ => panic!("raise error?")
                }
            }
            ItemType::Function { args, st } => {
                match obj {
                    Object::FunctionRef { name, arity } => {
                        if let Some(((fn_args, fn_st), body)) = env.get_function(&name, arity) {
                            println!("FN:\n {:?}\n {:?}", fn_args, fn_st);
                            if let Some(st) = st {
                                if st.is_not_same(env, &fn_st) {
                                    panic!("raise error?")
                                }
                            }
                            if let Some(args) = args {
                                if args.len() != fn_args.len() {
                                    panic!("raise error?")
                                }
                                for (st, fn_st) in args.into_iter().zip(fn_args.into_iter()) {
                                    if st.is_not_same(env, &fn_st) {
                                        panic!("raise error?")
                                    }
                                }
                            }
                            Ok(Object::FunctionRef { name: name.clone(), arity })
                        } else {
                            todo!("raise error?")
                        }
                    }
                    Object::Function { parameters, st: fn_st, body } => {
                        if let Some(st) = st {
                            if let Some(fn_st) = fn_st.as_ref() {
                                if st.is_not_same(env, fn_st) {
                                    panic!("raise error?")
                                }
                            } else {
                                panic!("raise error?")
                            }
                        }
                        if let Some(args) = args {
                            if args.len() != parameters.len() {
                                panic!("raise error?")
                            }
                            for (st, param) in args.into_iter().zip(parameters.clone().into_iter()) {
                                if let Some(fn_st) = &param.sequence_type {
                                    if st.is_not_same(env, fn_st) {
                                        panic!("raise error?")
                                    }
                                } else {
                                    todo!()
                                }
                            }
                        }
                        Ok(Object::Function { parameters, st: fn_st, body })
                    }
                    Object::Map(_) => {
                        todo!()
                    }
                    Object::Array(_) => {
                        todo!()
                    }
                    _ => panic!("TODO: {:?}", obj)
                }
            }
            _ => panic!("TODO: {:?}", self.item_type)
        }
    }

    // https://www.w3.org/TR/xpath-functions-31/#casting-from-primitive-to-primitive
    pub fn is_castable(&self, env: &Environment, obj: &Object) -> Result<bool, ErrorInfo> {
        self.is_castable_internal(env, obj, false, false)
    }

    fn is_castable_internal(&self, env: &Environment, obj: &Object, type_only: bool, is_array: bool) -> Result<bool, ErrorInfo> {
        println!("is_castable:\n st: {:#?}\n ob: {:#?}", self, obj);
        let result = match &self.item_type {
            ItemType::Item => {
                match obj {
                    Object::Nothing => panic!("raise error?"),
                    Object::Empty => {
                        if type_only {
                            todo!()
                        } else {
                            match self.occurrence_indicator {
                                OccurrenceIndicator::OneOrMore |
                                OccurrenceIndicator::ExactlyOne => false,
                                OccurrenceIndicator::ZeroOrOne |
                                OccurrenceIndicator::ZeroOrMore => true,
                            }
                        }
                    }
                    Object::Range { min, max } => {
                        if type_only {
                            todo!()
                        } else {
                            match self.occurrence_indicator {
                                OccurrenceIndicator::ExactlyOne |
                                OccurrenceIndicator::ZeroOrOne => min == max,
                                OccurrenceIndicator::ZeroOrMore |
                                OccurrenceIndicator::OneOrMore => true,
                            }
                        }
                    }
                    Object::Atomic(_) |
                    Object::Node(_) |
                    Object::Array(_) |
                    Object::Map(_) => true,
                    Object::Sequence(items) => {
                        if type_only {
                            todo!()
                        } else {
                            match self.occurrence_indicator {
                                OccurrenceIndicator::ExactlyOne => items.len() == 1,
                                OccurrenceIndicator::ZeroOrOne => items.len() >= 0 && items.len() <= 1,
                                OccurrenceIndicator::ZeroOrMore => items.len() >= 0,
                                OccurrenceIndicator::OneOrMore => items.len() >= 1
                            }
                        }
                    }
                    Object::Error { .. } => todo!(),
                    Object::CharRef { .. } => todo!(),
                    Object::EntityRef(_) => todo!(),
                    Object::Function { .. } => todo!(),
                    Object::FunctionRef { .. } => todo!(),
                    Object::Return(_) => todo!(),
                }
            }
            ItemType::AtomicOrUnionType(original_qname) => {
                let name = env.namespaces.resolve(original_qname);
                if name.is_same_qn(&XS_NOTATION) || name.is_same_qn(&XS_ANY_ATOMIC_TYPE) || name.is_same_qn(&XS_ANY_SIMPLE_TYPE) {
                    return Err((ErrorCode::XPST0080, String::from("TODO")));
                } else if name.is_same_qn(&XS_ANY_ATOMIC_TYPE) {
                    match obj {
                        Object::Empty => {
                            if is_array {
                                true
                            } else if type_only {
                                todo!()
                            } else {
                                match self.occurrence_indicator {
                                    OccurrenceIndicator::OneOrMore |
                                    OccurrenceIndicator::ExactlyOne => false,
                                    OccurrenceIndicator::ZeroOrOne |
                                    OccurrenceIndicator::ZeroOrMore => true,
                                }
                            }
                        },
                        Object::Range { .. } |
                        Object::Atomic(_) => true,
                        _ => todo!("{:?}", obj),
                    }
                } else {
                    match obj {
                        Object::Empty => {
                            if is_array {
                                true
                            } else if type_only {
                                todo!()
                            } else {
                                match self.occurrence_indicator {
                                    OccurrenceIndicator::OneOrMore |
                                    OccurrenceIndicator::ExactlyOne => false,
                                    OccurrenceIndicator::ZeroOrOne |
                                    OccurrenceIndicator::ZeroOrMore => true,
                                }
                            }
                        },
                        Object::Atomic(t) => {
                            match t {
                                Type::Untyped(str) |
                                Type::String(str) |
                                Type::NormalizedString(str) => {
                                    if let Some(types) = QNameToTypes.get(&name) {
                                        match types {
                                            Types::Untyped |
                                            Types::NormalizedString |
                                            Types::String |
                                            Types::AnyURI |
                                            Types::QName |
                                            Types::Boolean |
                                            Types::Integer |
                                            Types::Decimal |
                                            Types::Float |
                                            Types::Double |
                                            Types::DateTime |
                                            Types::DateTimeStamp |
                                            Types::Date |
                                            Types::Time |
                                            Types::Duration |
                                            Types::YearMonthDuration |
                                            Types::DayTimeDuration |
                                            Types::GYearMonth |
                                            Types::GYear |
                                            Types::GMonthDay |
                                            Types::GDay |
                                            Types::GMonth => {
                                                match t.convert(types.clone()) {
                                                    Ok(_) => true,
                                                    Err(_) => false
                                                }
                                            }
                                            Types::Base64Binary => {
                                                str.chars().all(
                                                    |c| (c >= 'A' && c <= 'Z')
                                                        || (c >= 'a' && c <= 'z')
                                                        || (c >= '0' && c <= '9')
                                                        || c == '+'
                                                        || c == '/'
                                                )
                                            }
                                            Types::HexBinary => {
                                                str.chars().all(
                                                    |c| (c >= 'A' && c <= 'F')
                                                        || (c >= 'a' && c <= 'f')
                                                        || (c >= '0' && c <= '9')
                                                ) && str.len() % 2 == 0
                                            }
                                            _ => todo!("{:?} {:?}", types, obj),
                                        }
                                    } else {
                                        todo!("handle custom types")
                                    }
                                }
                                Type::Integer(_) =>
                                    name == XS_UNTYPED_ATOMIC || name == XS_STRING || name == XS_BOOLEAN
                                        || name == XS_DOUBLE || name == XS_FLOAT || name == XS_DECIMAL || name == XS_INTEGER,
                                Type::Decimal { .. } =>
                                    name == XS_UNTYPED_ATOMIC || name == XS_STRING || name == XS_BOOLEAN
                                        || name == XS_DOUBLE || name == XS_FLOAT || name == XS_DECIMAL || name == XS_INTEGER,
                                Type::Float(num) =>
                                    name == XS_UNTYPED_ATOMIC || name == XS_STRING || name == XS_BOOLEAN
                                        || name == XS_DOUBLE || name == XS_FLOAT
                                        || ((num.is_zero() || num.is_normal()) && (name == XS_DECIMAL || name == XS_INTEGER)),
                                Type::Double(num) =>
                                    name == XS_UNTYPED_ATOMIC || name == XS_STRING || name == XS_BOOLEAN
                                        || name == XS_DOUBLE || name == XS_FLOAT
                                        || ((num.is_zero() || num.is_normal()) && (name == XS_DECIMAL || name == XS_INTEGER)),
                                Type::Duration { .. } => {
                                    name == XS_UNTYPED_ATOMIC || name == XS_STRING
                                        || name == XS_DURATION || name == XS_YEAR_MONTH_DURATION || name == XS_DAY_TIME_DURATION
                                }
                                Type::YearMonthDuration { .. } => {
                                    name == XS_UNTYPED_ATOMIC || name == XS_STRING
                                        || name == XS_DURATION || name == XS_YEAR_MONTH_DURATION || name == XS_DAY_TIME_DURATION
                                }
                                Type::DayTimeDuration { .. } => {
                                    name == XS_UNTYPED_ATOMIC || name == XS_STRING
                                        || name == XS_DURATION || name == XS_YEAR_MONTH_DURATION || name == XS_DAY_TIME_DURATION
                                }
                                Type::GYearMonth { .. } => {
                                    name == XS_UNTYPED_ATOMIC || name == XS_STRING || name == XS_G_YEAR_MONTH
                                }
                                Type::GYear { .. } => {
                                    name == XS_UNTYPED_ATOMIC || name == XS_STRING || name == XS_G_YEAR
                                }
                                Type::GMonthDay { .. } => {
                                    name == XS_UNTYPED_ATOMIC || name == XS_STRING || name == XS_G_MONTH_DAY
                                }
                                Type::GDay { .. } => {
                                    name == XS_UNTYPED_ATOMIC || name == XS_STRING || name == XS_G_DAY
                                }
                                Type::GMonth { .. } => {
                                    name == XS_UNTYPED_ATOMIC || name == XS_STRING || name == XS_G_MONTH
                                }
                                Type::DateTime { .. } => {
                                    name == XS_UNTYPED_ATOMIC || name == XS_STRING || name == XS_DATE_TIME
                                        || name == XS_TIME || name == XS_DATE
                                        || name == XS_G_YEAR_MONTH || name == XS_G_YEAR || name == XS_G_MONTH_DAY
                                        || name == XS_G_DAY || name == XS_G_MONTH
                                }
                                Type::Time { .. } => {
                                    name == XS_UNTYPED_ATOMIC || name == XS_STRING || name == XS_TIME
                                }
                                Type::Date { .. } => {
                                    name == XS_UNTYPED_ATOMIC || name == XS_STRING || name == XS_DATE_TIME
                                        || name == XS_DATE
                                        || name == XS_G_YEAR_MONTH || name == XS_G_YEAR || name == XS_G_MONTH_DAY
                                        || name == XS_G_DAY || name == XS_G_MONTH
                                }
                                Type::Boolean(_) => {
                                    name == XS_UNTYPED_ATOMIC || name == XS_STRING || name == XS_BOOLEAN
                                        || name == XS_DOUBLE || name == XS_FLOAT
                                        || name == XS_DECIMAL || name == XS_INTEGER
                                }
                                Type::Base64Binary(_) |
                                Type::HexBinary(_) => {
                                    name == XS_UNTYPED_ATOMIC || name == XS_STRING || name == XS_BASE64_BINARY || name == XS_HEX_BINARY
                                }
                                Type::AnyURI(_) => {
                                    name == XS_UNTYPED_ATOMIC || name == XS_STRING || name == XS_ANY_URI
                                }
                                Type::QName { .. } => {
                                    name == XS_UNTYPED_ATOMIC || name == XS_STRING || name == XS_QNAME
                                }
                                _ => todo!("{:?}", obj)
                            }
                        },
                        Object::Range { .. } => {
                            if match self.occurrence_indicator {
                                OccurrenceIndicator::ZeroOrOne |
                                OccurrenceIndicator::ExactlyOne => false,
                                OccurrenceIndicator::OneOrMore |
                                OccurrenceIndicator::ZeroOrMore => true,
                            } {
                                name == XS_UNTYPED_ATOMIC || name == XS_STRING || name == XS_BOOLEAN
                                    || name == XS_DOUBLE || name == XS_FLOAT || name == XS_DECIMAL || name == XS_INTEGER
                            } else {
                                false
                            }
                        }
                        Object::Sequence(items) => {
                            if type_only || match self.occurrence_indicator {
                                OccurrenceIndicator::ZeroOrOne |
                                OccurrenceIndicator::ExactlyOne => false,
                                OccurrenceIndicator::OneOrMore |
                                OccurrenceIndicator::ZeroOrMore => true,
                            } {
                                for item in items {
                                    if !self.is_castable_internal(env, item, false, false)? {
                                        return Ok(false);
                                    }
                                }
                                true
                            } else {
                                false
                            }
                        },
                        Object::Array(items) => {
                            if type_only {
                                todo!()
                            } else {
                                for item in items {
                                    if !self.is_castable_internal(env, item, false, true)? {
                                        return Ok(false);
                                    }
                                }
                                true
                            }
                        }
                        Object::Map(items) => {
                            return Err((ErrorCode::FOTY0013, String::from("TODO")));
                        }
                        Object::Node(rf) => {
                            let str = match rf.to_typed_value() {
                                Ok(str) => str,
                                Err(msg) => todo!("{}", msg)
                            };
                            self.is_castable_internal(env, &Object::Atomic(Type::Untyped(str)), false, false)?
                        }
                        _ => todo!("{:?}", obj),
                    }
                }
            },
            ItemType::Map(st) => {
                match obj {
                    Object::Map(items) => {
                        if let Some((k_st, v_st)) = st {
                            for (k, v) in items {
                                if k_st.occurrence_indicator == OccurrenceIndicator::ExactlyOne
                                    // TODO: optimize!!!
                                    && !k_st.is_castable_internal(env,&Object::Atomic(k.clone()), true, false)?
                                {
                                    return Ok(false);
                                }
                                if !v_st.is_castable_internal(env, v, false, false)? {
                                    return Ok(false);
                                }
                            }
                            true
                        } else {
                            true
                        }
                    }
                    _ => false
                }
            }
            ItemType::Array(st) => {
                match obj {
                    Object::Array(items) => {
                        if self.occurrence_indicator == OccurrenceIndicator::ExactlyOne {
                            if let Some(st) = st {
                                for item in items {
                                    if !st.is_castable_internal(env, item, false, true)? {
                                        return Ok(false);
                                    }
                                }
                            }
                            true
                        } else {
                            false
                        }
                    },
                    Object::Sequence(items) => {
                        if type_only || match self.occurrence_indicator {
                            OccurrenceIndicator::ExactlyOne => items.len() == 1,
                            OccurrenceIndicator::ZeroOrOne => items.len() >= 0 && items.len() <= 1,
                            OccurrenceIndicator::ZeroOrMore => items.len() >= 0,
                            OccurrenceIndicator::OneOrMore => items.len() >= 1,
                        } {
                            for item in items {
                                if !self.is_castable_internal(env, item, true, false)? {
                                    return Ok(false);
                                }
                            }
                            true
                        } else {
                            false
                        }
                    }
                    _ => false
                }
            }
            ItemType::Function { args, st } => {
                match obj {
                    Object::FunctionRef { name, arity } => {
                        if let Some(((fn_args, fn_st), body)) = env.get_function(name, *arity) {
                            println!("FN:\n {:?}\n {:?}", fn_args, fn_st);
                            if let Some(st) = st {
                                if st.is_not_same(env, &fn_st) {
                                    return Ok(false)
                                }
                            }
                            if let Some(args) = args {
                                if args.len() != fn_args.len() {
                                    return Ok(false)
                                }
                                for (st, fn_st) in args.into_iter().zip(fn_args.into_iter()) {
                                    if st.is_not_same(env, &fn_st) {
                                        return Ok(false)
                                    }
                                }
                            }
                            return Ok(true)
                        } else {
                            todo!("raise error?")
                        }
                    }
                    Object::Function { parameters, st: fn_st, .. } => {
                        if let Some(st) = st {
                            if let Some(fn_st) = fn_st {
                                if st.is_not_same(env, fn_st) {
                                    return Ok(false)
                                }
                            } else {
                                return Ok(false)
                            }
                        }
                        if let Some(args) = args {
                            if args.len() != parameters.len() {
                                return Ok(false)
                            }
                            for (st, param) in args.into_iter().zip(parameters.into_iter()) {
                                if let Some(fn_st) = &param.sequence_type {
                                    if st.is_not_same(env, fn_st) {
                                        return Ok(false)
                                    }
                                } else {
                                    todo!()
                                }
                            }
                        }
                        true
                    }
                    Object::Map(items) => {
                        todo!()
                    }
                    Object::Array(_) => {
                        if let Some(args) = args {
                            match args.as_slice() {
                                [SequenceType { item_type: ItemType::AtomicOrUnionType(arg_name), occurrence_indicator: OccurrenceIndicator::ExactlyOne }] => {
                                    let arg_name = env.namespaces.resolve(arg_name);
                                    if arg_name.is_same_qn(&XS_INTEGER) {
                                        if let Some(st) = st {
                                            match **st {
                                                SequenceType { item_type: ItemType::Item, occurrence_indicator: OccurrenceIndicator::ZeroOrMore } => {
                                                    true
                                                },
                                                _ => false
                                            }
                                        } else {
                                            false
                                        }
                                    } else {
                                        false
                                    }
                                }
                                _ => false
                            }
                        } else {
                            true
                        }
                    }
                    _ => panic!("TODO: {:?}", obj)
                }
            }
            _ => panic!("TODO: {:?}", self.item_type)
        };
        Ok(result)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct AnyKindTest { }

impl AnyKindTest {
    pub(crate) fn boxed() -> Box<dyn NodeTest> {
        Box::new(AnyKindTest { })
    }
}

impl NodeTest for AnyKindTest {
    fn test_node(&self, rf: &Reference) -> bool {
        true
    }
}

#[derive(Clone, Debug)]
pub(crate) struct DocumentTest {
    child: Option<Box<dyn NodeTest>>
}

impl DocumentTest {
    pub(crate) fn boxed(child: Option<Box<dyn NodeTest>>) -> Box<dyn NodeTest> {
        Box::new(DocumentTest { child })
    }
}

impl NodeTest for DocumentTest {
    fn test_node(&self, rf: &Reference) -> bool {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct TextTest {
}

impl TextTest {
    pub(crate) fn boxed() -> Box<dyn NodeTest> {
        Box::new(TextTest { })
    }
}

impl NodeTest for TextTest {
    fn test_node(&self, rf: &Reference) -> bool {
        rf.is_text()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct CommentTest {
}

impl CommentTest {
    pub(crate) fn boxed() -> Box<dyn NodeTest> {
        Box::new(CommentTest { })
    }
}

impl NodeTest for CommentTest {
    fn test_node(&self, rf: &Reference) -> bool {
        rf.is_comment()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct NamespaceNodeTest {
}

impl NamespaceNodeTest {
    pub(crate) fn boxed() -> Box<dyn NodeTest> {
        Box::new(NamespaceNodeTest { })
    }
}

impl NodeTest for NamespaceNodeTest {
    fn test_node(&self, rf: &Reference) -> bool {
        rf.is_namespace()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PITest {
    content: Option<Box<dyn Expression>>
}

impl PITest {
    pub(crate) fn boxed(content: Option<Box<dyn Expression>>) -> Box<dyn NodeTest> {
        Box::new(PITest { content })
    }
}

impl NodeTest for PITest {
    fn test_node(&self, rf: &Reference) -> bool {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ElementTest {
    name: Option<QName>,
    type_annotation: Option<QName>,
}

impl ElementTest {
    pub(crate) fn boxed(name: Option<QName>, type_annotation: Option<QName>) -> Box<dyn NodeTest> {
        Box::new(ElementTest { name, type_annotation })
    }
}

impl NodeTest for ElementTest {
    fn test_node(&self, rf: &Reference) -> bool {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct AttributeTest {
    name: Option<QName>,
    type_annotation: Option<QName>,
}

impl AttributeTest {
    pub(crate) fn boxed(name: Option<QName>, type_annotation: Option<QName>) -> Box<dyn NodeTest> {
        Box::new(AttributeTest { name: None, type_annotation: None })
    }
}

impl NodeTest for AttributeTest {
    fn test_node(&self, rf: &Reference) -> bool {
        if let Some(rf_name) = &rf.attr_name {
            if let Some(name) = &self.name {
                if rf_name == name {
                    if let Some(type_annotation) = &self.type_annotation {
                        todo!()
                    } else {
                        true
                    }
                } else {
                    false
                }
            } else {
                true
            }
        } else {
            false
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SchemaElementTest {
    name: QName
}

impl SchemaElementTest {
    pub(crate) fn boxed(name: QName) -> Box<dyn NodeTest> {
        Box::new(SchemaElementTest { name })
    }
}

impl NodeTest for SchemaElementTest {
    fn test_node(&self, rf: &Reference) -> bool {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SchemaAttributeTest {
    name: QName
}

impl SchemaAttributeTest {
    pub(crate) fn boxed(name: QName) -> Box<dyn NodeTest> {
        Box::new(SchemaAttributeTest { name })
    }
}

impl NodeTest for SchemaAttributeTest {
    fn test_node(&self, rf: &Reference) -> bool {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct NameTest { pub(crate) name: QName }

impl NameTest {
    pub(crate) fn boxed(name: QName) -> Box<dyn NodeTest> {
        Box::new(NameTest { name })
    }
}

impl NodeTest for NameTest {
    fn test_node(&self, rf: &Reference) -> bool {
        if let Some(name) = rf.name() {
            println!("{:?} vs {:?}", self.name.local_part, name.local_part);
            println!("{:?} vs {:?}", self.name.url, name.url);
            (self.name.local_part == "*" || self.name.local_part == name.local_part)
                && (self.name.url == Some(String::from("*")) || self.name.url == name.url)
        } else {
            false
        }
    }
}