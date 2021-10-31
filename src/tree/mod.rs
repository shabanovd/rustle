use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use std::sync::Mutex;
use dyn_clone::DynClone;
use crate::values::QName;

mod dln;
mod in_memory;

pub use dln::DLN;
pub use in_memory::InMemoryXMLTree;
use crate::eval::{Axis, Environment, INS};

#[derive(Clone)]
pub struct Reference {
    pub storage: Rc<Mutex<Box<dyn XMLTreeWriter>>>,
    pub id: DLN,
    pub attr_name: Option<QName>
}

impl Reference {
    pub fn name(&self) -> Option<QName> {
        let storage = self.storage.lock().unwrap();
        storage.as_reader().name(self)
    }

    pub fn xml_tree_id(&self) -> usize {
        let storage = self.storage.lock().unwrap();
        storage.id()
    }

    pub fn is_namespace(&self) -> bool {
        let storage = self.storage.lock().unwrap();
        storage.as_reader().is_namespace(self)
    }

    pub fn is_text(&self) -> bool {
        let storage = self.storage.lock().unwrap();
        storage.as_reader().is_text(self)
    }

    pub fn is_comment(&self) -> bool {
        let storage = self.storage.lock().unwrap();
        storage.as_reader().is_comment(self)
    }

    pub fn to_string(&self, env: &Box<Environment>) -> Result<String, String> {
        todo!()
    }

    pub fn to_xml(&self, env: &Box<Environment>) -> Result<String, String> {
        let storage = self.storage.lock().unwrap();
        storage.as_reader().to_xml(self, env)
    }

    pub fn to_typed_value(&self) -> Result<String, String> {
        let storage = self.storage.lock().unwrap();
        storage.as_reader().typed_value_of_node(self)
    }

    pub fn cmp(&self, other: &Reference) -> Ordering {
        let self_storage_id = self.storage.lock().unwrap().id();
        let other_storage_id = other.storage.lock().unwrap().id();
        let cmp = self_storage_id.cmp(&other_storage_id);
        if cmp == Ordering::Equal {
            let cmp = self.id.cmp(&other.id);
            if cmp == Ordering::Equal {
                if let Some(self_attr_name) = &self.attr_name {
                    if let Some(other_attr_name) = &other.attr_name {
                        self_attr_name.cmp(other_attr_name)
                    } else {
                        Ordering::Greater
                    }
                } else {
                    if let Some(..) = &other.attr_name {
                        Ordering::Less
                    } else {
                        Ordering::Equal
                    }
                }
            } else {
                cmp
            }
        } else {
            cmp
        }
    }

    pub(crate) fn attribute_value(&self, name: &QName) -> Option<String> {
        let storage = self.storage.lock().unwrap();
        storage.as_reader().attribute_value(&self, name)
    }

    pub(crate) fn root(&self) -> Option<Reference> {
        let storage = self.storage.lock().unwrap();
        storage.as_reader().root(&self)
    }

    pub(crate) fn attributes(&self) -> Option<Vec<Reference>> {
        let storage = self.storage.lock().unwrap();
        storage.as_reader().attributes(&self)
    }

    pub(crate) fn parent(&self) -> Option<Reference> {
        let storage = self.storage.lock().unwrap();
        storage.as_reader().parent(&self)
    }

    pub(crate) fn forward(&self, initial_node_sequence: &Option<INS>, axis: &Axis) -> Vec<Reference> {
        let storage = self.storage.lock().unwrap();
        storage.as_reader().forward(&self, initial_node_sequence, axis)
    }

    pub(crate) fn dump(&self) -> String {
        let storage = self.storage.lock().unwrap();
        storage.as_reader().dump(&self)
    }
}

impl Debug for Reference {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.dump())
    }

}

impl PartialEq for Reference {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

pub enum NodeType {
    Document,
    Element,

}

pub trait XMLNode: DynClone {
    fn id(&self) -> DLN;

    fn name(&self) -> Option<QName>;

    fn typed_value(&self) -> String;

    fn attribute_value(&self, name: &QName) -> Option<String>;
    fn add_attribute(&mut self, name: QName, value: String) -> bool;
    fn get_attributes(&self) -> Option<Vec<QName>>;

    // tests
    fn is_text(&self) -> bool;
    fn is_comment(&self) -> bool;

    fn to_xml_open(&self, env: &Box<Environment>) -> String;
    fn to_xml_start_children(&self) -> String;
    fn to_xml_close_empty(&self) -> String;
    fn to_xml_close(&self) -> String;

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
    fn init(&mut self, rf: Rc<Mutex<Box<dyn XMLTreeWriter>>>);

    fn id(&self) -> usize;

    fn as_reader(&self) -> Box<&dyn XMLTreeReader>;

    fn link_node(&mut self, rf: &Reference) -> Reference;

    fn start_document(&mut self) -> Reference;

    fn end_document(&mut self) -> Option<Reference>;

    fn start_element(&mut self, name: QName) -> Reference;

    fn attribute(&mut self, name: QName, value: String) -> Reference;

    fn end_element(&mut self) -> Option<Reference>;

    fn pi(&mut self, target: QName, content: String) -> Reference;

    fn ns(&mut self, prefix: String, url: String) -> Reference;

    fn text(&mut self, content: String) -> Reference;

    fn comment(&mut self, content: String) -> Reference;
}

dyn_clone::clone_trait_object!(XMLTreeWriter);

pub trait XMLTreeReader: DynClone {
    fn name(&self, pointer: &Reference) -> Option<QName>;

    fn to_string(&self, rf: &Reference) -> Result<String, String>;

    fn to_xml(&self, rf: &Reference, env: &Box<Environment>) -> Result<String, String>;

    fn typed_value_of_node(&self, rf: &Reference) -> Result<String, String>;

    fn first(&self) -> Option<Reference>;

    fn attribute_value(&self, rf: &Reference, name: &QName) -> Option<String>;

    // navigation
    fn root(&self, rf: &Reference) -> Option<Reference>;
    fn parent(&self, rf: &Reference) -> Option<Reference>;
    fn forward(&self, rf: &Reference, initial_node_sequence: &Option<INS>, axis: &Axis) -> Vec<Reference>;
    fn attributes(&self, rf: &Reference) -> Option<Vec<Reference>>;

    // tests
    fn is_namespace(&self, rf: &Reference) -> bool;
    fn is_text(&self, rf: &Reference) -> bool;
    fn is_comment(&self, rf: &Reference) -> bool;

    fn cmp(&self, other: Box<&dyn XMLTreeReader>, left: &Reference, right: &Reference) -> Ordering;

    // fn get_parent(&self) -> Box<dyn XMLNode>;
    // fn get_child(&self, child_pos: usize) -> Box<dyn XMLNode>;
    //
    // fn next_sibling(&self) -> Box<dyn XMLNode>;
    // fn preceding_sibling(&self) -> Box<dyn XMLNode>;

    fn dump(&self, rf: &Reference) -> String;
}

dyn_clone::clone_trait_object!(XMLTreeReader);