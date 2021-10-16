use std::cmp::Ordering;
use std::rc::Rc;
use std::sync::Mutex;
use dyn_clone::DynClone;
use crate::values::QName;

mod dln;
mod in_memory;

pub use dln::DLN;
pub use in_memory::InMemoryXMLTree;
use crate::eval::Environment;

#[derive(Clone)]
pub struct Reference {
    pub storage: Option<Rc<Mutex<Box<dyn XMLTreeWriter>>>>,
    pub storage_id: usize,
    pub id: DLN,
    pub attr_name: Option<QName>
}

impl Reference {
    pub fn name(&self, env: &Box<Environment>) -> Option<QName> {
        todo!()
    }

    pub fn to_string(&self, env: &Box<Environment>) -> Result<String, String> {
        todo!()
    }

    pub fn to_xml(&self, env: &Box<Environment>) -> Result<String, String> {
        todo!()
    }

    pub fn to_typed_value(&self, env: &Box<Environment>) -> Result<String, String> {
        if let Some(storage) = &self.storage {
            let storage = storage.lock().unwrap();
            storage.as_reader().typed_value_of_node(&self)
        } else {
            panic!("internal error")
        }
    }

    pub fn cmp(&self, other: &Reference) -> Ordering {
        todo!()
    }

    pub(crate) fn attributes(&self, env: &Box<Environment>) -> Vec<Reference> {
        todo!()
    }

    pub(crate) fn children(&self, env: &Box<Environment>) -> Vec<Reference> {
        todo!()
    }
}

impl PartialEq for Reference {
    fn eq(&self, other: &Self) -> bool {
        todo!()
    }
}

pub trait XMLNode: DynClone {
    fn id(&self) -> DLN;

    fn name(&self) -> Option<QName>;

    fn typed_value(&self) -> String;

    fn dump(&self) -> String;

    // checks
    // fn after(&self, other: DLN, is_following: bool) -> bool;
    // fn before(&self, other: DLN, is_preceding: bool) -> bool;
    //
    // fn is_descendant_of(&self, ancestor: DLN) -> bool;
    // fn is_descendant_or_self_of(&self, ancestor: DLN) -> bool;
    //
    // fn is_child_of(&self, parent: DLN) -> bool;
}

dyn_clone::clone_trait_object!(XMLNode);

pub trait XMLStorage {
    fn reader(&self) -> Option<Box<dyn XMLTreeReader>>;
    fn writer(&self) -> Option<Box<dyn XMLTreeWriter>>;
}

pub trait XMLTreeWriter: DynClone {
    fn id(&self) -> usize;

    fn as_reader(&self) -> Box<&dyn XMLTreeReader>;

    fn start_document(&mut self) -> Reference;

    fn end_document(&mut self) -> Option<Reference>;

    fn start_element(&mut self, name: QName) -> Reference;

    fn attribute(&mut self, name: QName, value: String) -> Reference;

    fn end_element(&mut self) -> Option<Reference>;

    fn pi(&mut self, target: QName, content: String) -> Reference;

    fn text(&mut self, content: String) -> Reference;

    fn comment(&mut self, content: String) -> Reference;

    fn dump(&self) -> String;
}

dyn_clone::clone_trait_object!(XMLTreeWriter);

pub trait XMLTreeReader: DynClone {
    fn name(&self, pointer: &Reference) -> Option<QName>;

    fn to_string(&self, rf: &Reference) -> Result<String, String>;

    fn to_xml(&self, rf: &Reference) -> Result<String, String>;

    fn typed_value_of_node(&self, rf: &Reference) -> Result<String, String>;

    fn cmp(&self, other: Box<&dyn XMLTreeReader>, left: &Reference, right: &Reference) -> Ordering;

    // fn get_parent(&self) -> Box<dyn XMLNode>;
    // fn get_child(&self, child_pos: usize) -> Box<dyn XMLNode>;
    //
    // fn next_sibling(&self) -> Box<dyn XMLNode>;
    // fn preceding_sibling(&self) -> Box<dyn XMLNode>;
}

dyn_clone::clone_trait_object!(XMLTreeReader);