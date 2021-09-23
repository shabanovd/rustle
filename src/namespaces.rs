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
pub const XPATH_FUNCTIONS: Namespace = Namespace { prefix: "fn", url: "http://www.w3.org/2005/xpath-functions" }; // fns ?
pub const XPATH_MAP: Namespace = Namespace { prefix: "map", url: "http://www.w3.org/2005/xpath-functions/map" };
pub const XPATH_ARRAY: Namespace = Namespace { prefix: "array", url: "http://www.w3.org/2005/xpath-functions/array" };
pub const XPATH_MATH: Namespace = Namespace { prefix: "math", url: "http://www.w3.org/2005/xpath-functions/math" };
pub const XQUERY_LOCAL: Namespace = Namespace { prefix: "local", url: "http://www.w3.org/2005/xquery-local-functions" };
pub const XQT_ERROR: Namespace = Namespace { prefix: "err", url: "http://www.w3.org/2005/xqt-errors" };
// http://www.w3.org/2012/xquery

#[derive(Clone)]
pub struct Namespaces<'a> {
    prefixes: HashMap<&'a str, Namespace<'a>>,
    pub default_for_element: &'a str,
    pub default_for_function: &'a str,
}

impl<'a> Namespaces<'a> {
    pub fn new() -> Self {
        let mut instance = Namespaces {
            prefixes: HashMap::new(),
            default_for_element: "",
            default_for_function: XPATH_FUNCTIONS.url,
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

    pub fn default_for_element(&self) -> String {
        String::from(self.default_for_element)
    }

    pub fn default_for_function(&self) -> String {
        String::from(self.default_for_function)
    }

    pub fn add(&mut self, ns: Namespace<'a>) {
        self.prefixes.insert(&ns.prefix, ns);
    }

    pub fn by_prefix(&self, prefix: &str) -> Option<&Namespace> {
        self.prefixes.get(prefix)
    }
}