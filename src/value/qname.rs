use core::fmt;
use crate::eval::Environment;

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct QNameResolved {
    pub url: String,
    pub local_part: String,
}

pub fn resolve_element_qname(qname: QName, env: &Box<Environment>) -> QNameResolved {
    if qname.url.trim().is_empty() {
        if let Some(ns) = env.namespaces.by_prefix(qname.prefix) {
            QNameResolved {
                url: String::from(ns.url),
                local_part: qname.local_part,
            }
        } else {
            QNameResolved {
                url: env.namespaces.default_for_element(),
                local_part: qname.local_part,
            }
        }
    } else {
        QNameResolved {
            url: qname.url,
            local_part: qname.local_part
        }
    }
}

pub fn resolve_function_qname(qname: QName, env: &Box<Environment>) -> QNameResolved {
    if qname.url.trim().is_empty() {
        if let Some(ns) = env.namespaces.by_prefix(qname.prefix) {
            QNameResolved {
                url: String::from(ns.url),
                local_part: qname.local_part,
            }
        } else {
            QNameResolved {
                url: env.namespaces.default_for_function(),
                local_part: qname.local_part,
            }
        }
    } else {
        QNameResolved {
            url: qname.url,
            local_part: qname.local_part
        }
    }
}

#[derive(Eq, PartialEq, Clone, Hash)]
pub struct QName {
    pub prefix: String,
    pub url: String,
    pub local_part: String,
}

impl QName {
    pub fn new(prefix: String, local_part: String) -> Self {
        QName {
            prefix,
            url: String::from("" ),
            local_part,
        }
    }

    pub fn local_part(local_part: &str) -> Self {
        QName {
            prefix: String::from("" ),
            url: String::from("" ),
            local_part: String::from( local_part ),
        }
    }

    // pub fn from_string(str: String) -> Self {
    //     // TODO fix it by paring string
    //     QName::local_part(str.as_str())
    // }

    pub fn len(&self) -> usize {
        if self.prefix.len() != 0 {
            self.prefix.len() + 1 + self.local_part.len()
        } else {
            self.local_part.len()
        }
    }

    pub fn string(&self) -> String {
        let mut str = String::with_capacity(self.len());
        if self.prefix.len() != 0 {
            str.push_str(self.prefix.as_str());
            str.push_str(":");
        }
        str.push_str(self.local_part.as_str());
        str
    }
}

impl fmt::Debug for QName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.prefix.is_empty() {
            write!(f, "QName {{ {} }}", self.local_part)
        } else {
            write!(f, "QName {{ {}:{} }}", self.prefix, self.local_part)
        }
    }
}