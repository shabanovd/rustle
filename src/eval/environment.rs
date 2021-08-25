use std::collections::HashMap;
use crate::eval::Object;
use crate::fns::FunctionsRegister;
use crate::namespaces::*;

#[derive(Clone)]
pub struct Environment<'a> {
    pub namespaces: Namespaces<'a>,
    vars: HashMap<String, Object>,
    pub functions: FunctionsRegister<'a>,
}

impl<'a> Environment<'a> {
    pub fn new() -> Self {
        Environment {
            namespaces: Namespaces::new(),
            vars: HashMap::new(),
            functions: FunctionsRegister::new(),
        }
    }

    pub fn set(&mut self, key: String, value: Object) {
        self.vars.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<Object> {
        self.vars.get(key).map(|val| val.clone())
    }
}