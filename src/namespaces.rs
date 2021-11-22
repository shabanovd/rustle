use std::collections::HashMap;
use crate::values::{QName, QNameResolved};

pub trait Namespace {
    fn prefix(&self) -> String;
    fn uri(&self) -> String;


    fn to_heap(&self) -> NS_heap {
        NS_heap { prefix: self.prefix(), uri: self.uri() }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct NS_heap {
    pub prefix: String,
    pub uri: String,
}

impl NS_heap {
    fn new<S: Into<String>>(prefix: S, uri: S) -> Self {
        NS_heap {
            prefix: prefix.into(),
            uri: uri.into(),
        }
    }
}

impl Namespace for NS_heap {
    fn prefix(&self) -> String {
        self.prefix.clone()
    }

    fn uri(&self) -> String {
        self.uri.clone()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct NS<'a> {
    pub prefix: &'a str,
    pub uri: &'a str,
}

impl<'a> NS<'a> {
    const fn new(prefix: &'a str, uri: &'a str) -> Self {
        NS { prefix, uri }
    }
}

impl<'a> Namespace for NS<'a> {
    fn prefix(&self) -> String {
        self.prefix.to_string()
    }

    fn uri(&self) -> String {
        self.uri.to_string()
    }
}

// namespaces
pub const XML: NS = NS::new("xml", "http://www.w3.org/XML/1998/namespace");
pub const SCHEMA: NS = NS::new("xs", "http://www.w3.org/2001/XMLSchema");
pub const SCHEMA_INSTANCE: NS = NS::new("xsi", "http://www.w3.org/2001/XMLSchema-instance");
pub const XPATH_FUNCTIONS: NS = NS::new("fn", "http://www.w3.org/2005/xpath-functions"); // fns ?
pub const XPATH_MAP: NS = NS::new("map", "http://www.w3.org/2005/xpath-functions/map");
pub const XPATH_ARRAY: NS = NS::new("array", "http://www.w3.org/2005/xpath-functions/array");
pub const XPATH_MATH: NS = NS::new("math", "http://www.w3.org/2005/xpath-functions/math");
pub const XQUERY_LOCAL: NS = NS::new("local", "http://www.w3.org/2005/xquery-local-functions");
pub const XQT_ERROR: NS = NS::new("err", "http://www.w3.org/2005/xqt-errors");
// http://www.w3.org/2012/xquery

lazy_static! {
    pub static ref NS_BY_PREFIX: HashMap<String, NS_heap> = {
        let mut map = HashMap::new();

        for ns in [
            &XML,
            &SCHEMA,
            &SCHEMA_INSTANCE,
            &XPATH_FUNCTIONS,
            &XPATH_MAP,
            &XPATH_ARRAY,
            &XPATH_MATH,
            &XQUERY_LOCAL,
            &XQT_ERROR
        ] {
            map.insert(ns.prefix(), ns.to_heap());
        }

        map
    };
}

#[derive(Clone)]
pub struct Namespaces {
    prefixes: HashMap<String, NS_heap>,
    pub default_for_element: String,
    pub default_for_function: String,
}

impl Namespaces {
    pub fn new() -> Self {
        let mut instance = Namespaces {
            prefixes: HashMap::new(),
            default_for_element: "".to_string(),
            default_for_function: XPATH_FUNCTIONS.uri.to_string(),
        };

        instance.add(&XML);
        instance.add(&SCHEMA);
        instance.add(&SCHEMA_INSTANCE);
        instance.add(&XPATH_FUNCTIONS);
        instance.add(&XPATH_MAP);
        instance.add(&XPATH_ARRAY);
        instance.add(&XPATH_MATH);
        instance.add(&XQUERY_LOCAL);

        instance
    }

    pub fn resolve(&self, mut name: &QName) -> QNameResolved {
        let url = if let Some(url) = &name.url {
            url.clone()
        } else if let Some(prefix) = &name.prefix {
            if let Some(ns) = self.prefixes.get(prefix) {
                ns.uri.clone()
            } else {
                todo!("raise error?")
            }
        } else {
            "".to_string()
        };
        QNameResolved { url, local_part: name.local_part.clone() }
    }

    pub fn default_for_element(&self) -> String {
        self.default_for_element.clone()
    }

    pub fn default_for_function(&self) -> String {
        self.default_for_function.clone()
    }

    pub fn add<T>(&mut self, ns: &T) where T: Namespace {
        self.prefixes.insert(ns.prefix(), ns.to_heap());
    }

    pub fn by_prefix(&self, prefix: &str) -> Option<&NS_heap> {
        self.prefixes.get(prefix)
    }
}