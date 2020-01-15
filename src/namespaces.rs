use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub struct Namespace<'a> {
    pub prefix: &'a str,
    pub url: &'a str,
}

// namespaces
pub const XML: Namespace = Namespace { prefix: "xml", url: "http://www.w3.org/XML/1998/namespace" };
pub const SCHEMA: Namespace = Namespace { prefix: "xs", url: "http://www.w3.org/2001/XMLSchema" };
pub const SCHEMA_INSTANCE: Namespace = Namespace { prefix: "xsi", url: "http://www.w3.org/2001/XMLSchema-instance" };
pub const XPATH_FUNCTIONS: Namespace = Namespace { prefix: "fns", url: "http://www.w3.org/2005/xpath-functions" };
pub const XPATH_MAP: Namespace = Namespace { prefix: "map", url: "http://www.w3.org/2005/xpath-functions/map" };
pub const XPATH_ARRAY: Namespace = Namespace { prefix: "array", url: "http://www.w3.org/2005/xpath-functions/array" };
pub const XPATH_MATH: Namespace = Namespace { prefix: "math", url: "http://www.w3.org/2005/xpath-functions/math" };
pub const XQUERY_LOCAL: Namespace = Namespace { prefix: "local", url: "http://www.w3.org/2005/xquery-local-functions" };
pub const XQT_ERROR: Namespace = Namespace { prefix: "err", url: "http://www.w3.org/2005/xqt-errors" };
// http://www.w3.org/2012/xquery

pub struct Namespaces<'a> {
    prefixes: HashMap<&'a str, Namespace<'a>>
}

impl<'a> Namespaces<'a> {
    pub fn new() -> Self {
        let mut instance = Namespaces {
            prefixes: HashMap::new(),
        };

        instance.add(XML);
        instance.add(SCHEMA);
        instance.add(SCHEMA_INSTANCE);
        instance.add(XPATH_FUNCTIONS);
        instance.add(XPATH_MAP);
        instance.add(XPATH_ARRAY);
        instance.add(XPATH_MATH);
        instance.add(XQUERY_LOCAL);

        instance
    }

    pub fn add(&mut self, ns: Namespace<'a>) {
        self.prefixes.insert(&ns.prefix, ns);
    }
}