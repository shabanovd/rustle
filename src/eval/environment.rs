use std::collections::HashMap;
use crate::eval::Object;

pub struct Environment {
    vars: HashMap<String, Object>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            vars: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: String, value: Object) {
        self.vars.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<Object> {
        self.vars.get(key).map(|val| val.clone())
    }
}