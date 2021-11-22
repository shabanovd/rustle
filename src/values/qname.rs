use core::fmt;
use crate::eval::{Environment, DynamicContext, EvalResult, Object, Type};
use crate::namespaces::{Namespace, SCHEMA};
use std::cmp::Ordering;
use crate::eval::expression::Expression;

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct QNameResolved {
    pub url: String,
    pub local_part: String,
}

impl QNameResolved {
    pub(crate) fn is_same(&self, qname: &QName) -> bool {
        if self.local_part == qname.local_part {
            if let Some(url) = &qname.url {
                &self.url == url
            } else {
                self.url == ""
            }
        } else {
            false
        }
    }

    pub(crate) fn is_same_qn(&self, qname: &QN) -> bool {
        self.local_part == qname.local_part && self.url == qname.url
    }
}

impl PartialEq<QName> for QNameResolved {
    fn eq(&self, other: &QName) -> bool {
        self.is_same(other)
    }
}

impl<'a> PartialEq<QN<'a>> for QNameResolved {
    fn eq(&self, other: &QN) -> bool {
        self.is_same_qn(other)
    }
}

pub fn resolve_element_qname(qname: &QName, env: &Box<Environment>) -> QNameResolved {
    resolve_qname(qname, env, &env.namespaces.default_for_element)
}

pub fn resolve_function_qname(qname: &QName, env: &Box<Environment>) -> QNameResolved {
    resolve_qname(qname, env, &env.namespaces.default_for_function)
}

fn resolve_qname(qname: &QName, env: &Box<Environment>, default: &String) -> QNameResolved {
    if let Some(url) = &qname.url {
        QNameResolved { url: url.clone(), local_part: qname.local_part.clone() }
    } else {
        if let Some(prefix) = &qname.prefix {
            if let Some(ns) = env.namespaces.by_prefix(prefix) {
                QNameResolved {
                    url: ns.uri.clone(),
                    local_part: qname.local_part.clone(),
                }
            } else {
                todo!("error?")
            }
        } else {
            QNameResolved {
                url: default.clone(),
                local_part: qname.local_part.clone(),
            }
        }
    }
}

pub struct QN<'a> {
    pub prefix: &'a str,
    pub url: &'a str,
    pub local_part: &'a str,
}

impl<'a> QN<'a> {
    pub const fn full(prefix: &'a str, local_part: &'a str, url: &'a str) -> Self {
        QN { prefix, url, local_part }
    }
}

impl<'a> Into<QName> for QN<'a> {
    fn into(self) -> QName {
        QName::full(self.prefix.to_string(), self.local_part.to_string(), self.url.to_string())
    }
}

impl<'a> PartialEq<QNameResolved> for QN<'a> {
    fn eq(&self, other: &QNameResolved) -> bool {
        other.is_same_qn(self)
    }
}

#[derive(Eq, PartialEq, Clone, Hash)]
pub struct QName {
    pub prefix: Option<String>,
    pub url: Option<String>,
    pub local_part: String,
}

impl QName {
    pub fn wildcard() -> Self {
        QName {
            prefix: Some(String::from("*")),
            url: Some(String::from("*")),
            local_part: String::from("*"),
        }
    }

    pub fn full<S: Into<String>>(prefix: S, local_part: S, url: S) -> Self {
        QName {
            prefix: Some(prefix.into()),
            url: Some(url.into()),
            local_part: local_part.into(),
        }
    }

    pub fn ns<N>(ns: &N, local_part: String) -> Self where N: Namespace {
        QName {
            prefix: Some(ns.prefix()),
            url: Some(ns.uri()),
            local_part: local_part.into(),
        }
    }

    pub fn new(prefix: String, local_part: String) -> Self {
        if prefix.len() == 0 {
            QName {
                prefix: None,
                url: None,
                local_part,
            }
        } else {
            QName {
                prefix: Some(prefix),
                url: None,
                local_part,
            }
        }
    }

    pub fn local_part<S: Into<String>>(local_part: S) -> Self {
        QName {
            prefix: None,
            url: None,
            local_part: local_part.into(),
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

    pub(crate) fn cmp(&self, other: &QName) -> Ordering {
        if self.local_part == other.local_part && self.url == other.url {
            Ordering::Equal
        } else {
            self.local_part.cmp(&other.local_part)
        }
    }

    fn len(&self) -> usize {
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

impl PartialEq<QNameResolved> for QName {
    fn eq(&self, other: &QNameResolved) -> bool {
        other.is_same(self)
    }
}

impl Expression for QName {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        Ok((env, Object::Atomic( Type::QName { local_part: self.local_part.clone(), url: self.url.clone(), prefix: self.prefix.clone() } ) ))
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}

impl fmt::Debug for QName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(prefix) = &self.prefix {
            if let Some(url) = &self.url {
                write!(f, "QN{{{}:{} {}}}", prefix, self.local_part, url)
            } else {
                write!(f, "QN{{{}:{}}}", prefix, self.local_part)
            }
        } else if let Some(url) = &self.url {
            write!(f, "QN{{{} {}}}", self.local_part, url)
        } else {
            write!(f, "QN{{{}}}", self.local_part)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Name {
    pub name: String,
}

impl Name {
    pub(crate) fn boxed(name: String) -> Box<dyn Expression> {
        Box::new(Name { name })
    }
}

impl Expression for Name {
    fn eval<'a>(&self, env: Box<Environment>, context: &DynamicContext) -> EvalResult {
        Ok((env, Object::Atomic( Type::String(self.name.clone()) ) ))
    }

    fn predicate<'a>(&self, env: Box<Environment>, context: &DynamicContext, value: Object) -> EvalResult {
        todo!()
    }
}