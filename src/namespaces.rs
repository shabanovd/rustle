use std::collections::HashMap;
use crate::values::{QName, QNameResolved};

#[derive(Debug, PartialEq, Clone)]
pub struct Namespace {
    pub prefix: String,
    pub uri: String,
}

impl Namespace {
    fn new<S: Into<String>>(prefix: S, uri: S) -> Self {
        Namespace {
            prefix: prefix.into(),
            uri: uri.into(),
        }
    }
}

// namespaces
lazy_static! {
    pub static ref XML: Namespace = Namespace::new("xml", "http://www.w3.org/XML/1998/namespace");
    pub static ref SCHEMA: Namespace = Namespace::new("xs", "http://www.w3.org/2001/XMLSchema");
    pub static ref SCHEMA_INSTANCE: Namespace = Namespace::new("xsi", "http://www.w3.org/2001/XMLSchema-instance");
    pub static ref XPATH_FUNCTIONS: Namespace = Namespace::new("fn", "http://www.w3.org/2005/xpath-functions"); // fns ?
    pub static ref XPATH_MAP: Namespace = Namespace::new("map", "http://www.w3.org/2005/xpath-functions/map");
    pub static ref XPATH_ARRAY: Namespace = Namespace::new("array", "http://www.w3.org/2005/xpath-functions/array");
    pub static ref XPATH_MATH: Namespace = Namespace::new("math", "http://www.w3.org/2005/xpath-functions/math");
    pub static ref XQUERY_LOCAL: Namespace = Namespace::new("local", "http://www.w3.org/2005/xquery-local-functions");
    pub static ref XQT_ERROR: Namespace = Namespace::new("err", "http://www.w3.org/2005/xqt-errors");
    // http://www.w3.org/2012/xquery
}

lazy_static! {
    pub static ref NS: HashMap<String, Namespace> = {
        let mut map = HashMap::new();

        for ns in [
            &*XML,
            &*SCHEMA,
            &*SCHEMA_INSTANCE,
            &*XPATH_FUNCTIONS,
            &*XPATH_MAP,
            &*XPATH_ARRAY,
            &*XPATH_MATH,
            &*XQUERY_LOCAL,
            &*XQT_ERROR
        ] {
            map.insert(ns.prefix.clone(), ns.clone());
        }

        map
    };
}

#[derive(Clone)]
pub struct Namespaces {
    prefixes: HashMap<String, Namespace>,
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

        instance.add(&*XML);
        instance.add(&*SCHEMA);
        instance.add(&*SCHEMA_INSTANCE);
        instance.add(&*XPATH_FUNCTIONS);
        instance.add(&*XPATH_MAP);
        instance.add(&*XPATH_ARRAY);
        instance.add(&*XPATH_MATH);
        instance.add(&*XQUERY_LOCAL);

        instance
    }

    pub fn resolve(&self, mut name: QName) -> QNameResolved {
        if name.url.is_none() {
            if let Some(prefix) = &name.prefix {
                if let Some(ns) = self.prefixes.get(prefix) {
                    name.url = Some(ns.uri.clone())
                }
            }
        }
        let url = if let Some(url) = name.url { url } else { "".to_string() };
        QNameResolved { url, local_part: name.local_part }
    }

    pub fn default_for_element(&self) -> String {
        self.default_for_element.clone()
    }

    pub fn default_for_function(&self) -> String {
        self.default_for_function.clone()
    }

    pub fn add(&mut self, ns: &Namespace) {
        self.prefixes.insert(ns.prefix.clone(), ns.clone());
    }

    pub fn by_prefix(&self, prefix: &str) -> Option<&Namespace> {
        self.prefixes.get(prefix)
    }
}