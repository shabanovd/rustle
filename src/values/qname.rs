use core::fmt;
use crate::eval::{Environment, DynamicContext, EvalResult, Object, Type};
use crate::namespaces::SCHEMA;
use std::cmp::Ordering;
use crate::eval::expression::Expression;

lazy_static! {
    pub static ref XS_STRING: QName = QName::full("xs", "string", SCHEMA.url);
    pub static ref XS_INTEGER: QName = QName::full("xs", "integer", SCHEMA.url);
    pub static ref XS_DECIMAL: QName = QName::full("xs", "decimal", SCHEMA.url);
    pub static ref XS_FLOAT: QName = QName::full("xs", "float", SCHEMA.url);
    pub static ref XS_DOUBLE: QName = QName::full("xs", "double", SCHEMA.url);
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct QNameResolved {
    pub url: String,
    pub local_part: String,
}

pub fn resolve_element_qname(qname: &QName, env: &Box<Environment>) -> QNameResolved {
    resolve_qname(qname, env, env.namespaces.default_for_element)
}

pub fn resolve_function_qname(qname: &QName, env: &Box<Environment>) -> QNameResolved {
    resolve_qname(qname, env, env.namespaces.default_for_function)
}

fn resolve_qname(qname: &QName, env: &Box<Environment>, default: &str) -> QNameResolved {
    if let Some(url) = &qname.url {
        QNameResolved { url: url.clone(), local_part: qname.local_part.clone() }
    } else {
        if let Some(prefix) = &qname.prefix {
            if let Some(ns) = env.namespaces.by_prefix(prefix) {
                QNameResolved {
                    url: String::from(ns.url),
                    local_part: qname.local_part.clone(),
                }
            } else {
                todo!("error?")
            }
        } else {
            QNameResolved {
                url: String::from(default),
                local_part: qname.local_part.clone(),
            }
        }
    }
}

#[derive(Eq, PartialEq, Clone, Hash)]
pub struct QName {
    pub prefix: Option<String>,
    pub url: Option<String>,
    pub local_part: String,
}

impl QName {
    pub fn full(prefix: &str, local_part: &str, url: &str) -> Self {
        QName {
            prefix: Some(String::from(prefix)),
            url: Some(String::from(url)),
            local_part: String::from(local_part),
        }
    }

    pub fn new(prefix: String, local_part: String) -> Self {
        QName {
            prefix: Some(prefix),
            url: None,
            local_part,
        }
    }

    pub fn local_part(local_part: &str) -> Self {
        QName {
            prefix: None,
            url: None,
            local_part: String::from( local_part ),
        }
    }

    // pub fn from_string(str: String) -> Self {
    //     // TODO fix it by paring string
    //     QName::local_part(str.as_str())
    // }

    pub(crate) fn partial_cmp(&self, other: &QName) -> Option<Ordering> {
        if self.local_part == other.local_part && self.url == other.url {
            Some(Ordering::Equal)
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        if let Some(prefix) = &self.prefix {
            prefix.len() + 1 + self.local_part.len()
        } else {
            self.local_part.len()
        }
    }

    pub fn string(&self) -> String {
        let mut str = String::with_capacity(self.len());
        if let Some(prefix) = &self.prefix {
            str.push_str(prefix.as_str());
            str.push_str(":");
        }
        str.push_str(self.local_part.as_str());
        str
    }
}

impl Expression for QName {
    fn eval<'a>(&self, env: Box<Environment<'a>>, context: &DynamicContext) -> EvalResult<'a> {
        Ok((env, Object::Atomic( Type::QName { local_part: self.local_part.clone(), url: self.url.clone(), prefix: self.prefix.clone() } ) ))
    }

    fn debug(&self) -> String {
        todo!()
    }
}

impl fmt::Debug for QName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(prefix) = &self.prefix {
            write!(f, "QName {{ {}:{} }}", prefix, self.local_part)
        } else {
            write!(f, "QName {{ {} }}", self.local_part)
        }
    }
}