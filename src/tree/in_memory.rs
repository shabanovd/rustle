use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fs;
use std::rc::Rc;
use std::sync::Mutex;
use linked_hash_map::LinkedHashMap;
use xmlparser::{ElementEnd, Token};
use crate::eval::{Axis, ErrorInfo, INS};
use crate::namespaces::XML;
use crate::tree::dln::DLN;
use crate::tree::{NodeType, Reference, XMLNode, XMLTreeReader, XMLTreeWriter};
use crate::values::QName;

#[derive(Clone)]
pub struct InMemoryXMLTree {
    id: usize,
    storage: Option<Rc<Mutex<Box<dyn XMLTreeWriter>>>>,

    // required during build only
    stack: Vec<(DLN, usize)>,
    namespaces: LinkedHashMap<String,String>,

    // state
    items: BTreeMap<DLN, Box<dyn XMLNode>>,
}

impl InMemoryXMLTree {
    fn instance(id: usize) -> Box<dyn XMLTreeWriter> {
        let mut namespaces = LinkedHashMap::with_capacity(21);
        namespaces.insert("xml".to_string(), "http://www.w3.org/XML/1998/namespace".to_string());

        Box::new(InMemoryXMLTree {
            id,
            storage: None,
            stack: Vec::with_capacity(21),
            namespaces,
            items: BTreeMap::new()
        })
    }

    pub fn create(id: usize) -> Rc<Mutex<Box<dyn XMLTreeWriter>>> {
        let tree = InMemoryXMLTree::instance(id);

        let rf = Rc::new(Mutex::new(tree));
        let clone = rf.clone();
        {
            let mut instance = rf.lock().unwrap();
            instance.init(clone);
        }

        rf
    }

    fn resolve(&self, mut name: QName) -> QName {
        if name.url.is_none() {
            if let Some(prefix) = &name.prefix {
                if let Some(url) = self.namespaces.get(prefix) {
                    name.url = Some(url.clone());
                }
            }
        }
        name
    }

    pub fn load(id: usize, path: &str) -> Rc<Mutex<Box<dyn XMLTreeWriter>>> {
        let data = fs::read_to_string(path).unwrap();
        InMemoryXMLTree::from_str(id, data.as_str())
    }

    pub fn from_str(id: usize, data: &str) -> Rc<Mutex<Box<dyn XMLTreeWriter>>> {
        let rf = InMemoryXMLTree::create(id);
        {
            let mut tree = rf.lock().unwrap();
            tree.start_document();
            for token in xmlparser::Tokenizer::from(data) {
                match token.unwrap() {
                    Token::Declaration { .. } => {
                        // TODO
                    }
                    Token::ProcessingInstruction { target, content, .. } => {
                        let target = QName::local_part(target.as_str());
                        let content = if let Some(data) = content {
                            data.as_str().to_string()
                        } else {
                            String::new()
                        };
                        tree.pi(target, content);
                    }
                    Token::Comment { text, .. } => {
                        tree.comment(text.as_str().to_string());
                    },
                    Token::DtdStart { .. } => panic!(),
                    Token::EmptyDtd { .. } => panic!(),
                    Token::EntityDeclaration { .. } => panic!(),
                    Token::DtdEnd { .. } => panic!(),
                    Token::ElementStart { prefix, local, .. } => {
                        let name = QName::new(prefix.as_str().to_string(), local.as_str().to_string());
                        tree.start_element(name);
                    }
                    Token::Attribute { prefix, local, value, .. } => {
                        let name = QName::new(prefix.as_str().to_string(), local.as_str().to_string());
                        tree.attribute(name, value.as_str().to_string());
                    }
                    Token::ElementEnd { end, .. } => {
                        match end {
                            ElementEnd::Open => {}
                            ElementEnd::Close(prefix, local) => {
                                tree.end_element();
                            },
                            ElementEnd::Empty => {
                                tree.end_element();
                            }
                        }
                    }
                    Token::Text { text } => {
                        tree.text(text.as_str().to_string());
                    },
                    Token::Cdata { text, .. } => panic!()
                }
            }
            tree.end_document();
        }

        rf
    }

    pub(crate) fn as_writer(self) -> Box<dyn XMLTreeWriter> {
        Box::new(self)
    }

    fn pop(&mut self) -> Option<Reference> {
        match self.stack.pop() {
            Some((id, ns_len)) => {
                while self.namespaces.len() > ns_len {
                    self.namespaces.pop_back();
                }
                Some(self.reference(id, None))
            },
            None => None
        }
    }

    fn add_namespace(&mut self, name: &QName, value: &String) {
        if let Some(prefix) = &name.prefix {
            if prefix == "xmlns" {
                self.namespaces.insert(name.local_part.clone(), value.clone());
            }
        } else if name.local_part == "xmlns" {
            self.namespaces.insert("".to_string(), value.clone());
        }
    }

    fn reference(&self, id: DLN, attr_name: Option<QName>) -> Reference {
        if let Some(storage) = self.storage.clone() {
            Reference { storage, id, attr_name }
        } else {
            panic!("internal error")
        }
    }

    fn last_id(&self) -> usize {
        if let Some((k,v)) = self.items.last_key_value() {
            k.get_level_id(0)
        } else {
            0
        }
    }

    fn next_top_id(&mut self) -> DLN {
        let next_id = self.last_id() + 1;
        let id = DLN::level_id(next_id);
        self.stack.push((id.clone(), self.namespaces.len()));
        id
    }

    fn prepare_child(&mut self) -> DLN {
        let next = if let Some((id, ns_len)) = self.stack.last() {
            id.zero_child()
        } else {
            self.next_top_id().zero_child()
        };
        self.stack.push((next.clone(), self.namespaces.len()));
        next
    }

    fn next_sibling(&mut self) -> DLN {
        let next = if let Some((id, ns_len)) = self.stack.pop() {
            id.next_sibling()
        } else {
            self.next_top_id().next_sibling()
        };
        self.stack.push((next.clone(), self.namespaces.len()));
        next
    }

    fn children(&self, rf: Reference, include_self: bool, all: bool) -> Vec<Reference> {
        let mut result = vec![];
        let level = rf.id.count_levels() + 1;

        for (k, v) in self.items.range(&rf.id..) {
            if k == &rf.id {
                if include_self {
                    result.push(self.reference(k.clone(), None));
                }
            } else if k.start_with(&rf.id) {
                if all || k.count_levels() == level {
                    result.push(self.reference(k.clone(), None));
                }
            } else {
                break;
            }
        }
        result
    }

    fn dump(&self) -> String {
        let mut buf = String::with_capacity(100_100);
        buf.push_str(format!("storage: {}\n", self.id).as_str());
        for (_, node) in &self.items {
            buf.push_str(node.dump().as_str());
            buf.push_str("\n");
        }
        buf
    }
}

impl XMLTreeReader for InMemoryXMLTree {
    fn name(&self, rf: &Reference) -> Option<QName> {
        if let Some(attr_name) = &rf.attr_name {
            // if let Some(uri) = &attr_name.url {
            //     if uri == &*XML.uri {
            //         Some(QName::local_part(attr_name.local_part.clone()))
            //     } else {
            //         Some(attr_name.clone())
            //     }
            // } else {
            Some(attr_name.clone())
            // }
        } else {
            if let Some(node) = self.items.get(&rf.id) {
                node.name()
            } else {
                // TODO raise error?
                None
            }
        }
    }

    fn target(&self, rf: &Reference) -> Option<QName> {
        if let Some(attr_name) = &rf.attr_name {
            None
        } else {
            if let Some(node) = self.items.get(&rf.id) {
                node.target()
            } else {
                // TODO raise error?
                None
            }
        }
    }

    fn content(&self, rf: &Reference) -> Option<String> {
        if let Some(attr_name) = &rf.attr_name {
            None
        } else {
            if let Some(node) = self.items.get(&rf.id) {
                node.content()
            } else {
                // TODO raise error?
                None
            }
        }
    }

    fn to_string(&self, rf: &Reference) -> Result<String, String> {
        todo!()
    }

    fn to_xml(&self, rf: &Reference) -> Result<String, String> {
        // println!("to_xml {}", self.dump());
        let mut ids: Vec<(usize, &DLN, &Box<dyn XMLNode>, usize)> = vec![];

        let mut namespaces = LinkedHashMap::with_capacity(21);
        let mut count = 0;

        let mut buf = String::new();
        for (k,node) in self.items.range(&rf.id..) {
            // println!("{} {:?}", k, node);

            // closing base on id level
            while let Some((c, last, n, ns_len)) = ids.last() {
                if k.start_with(last) {
                    if c + 1 == count {
                        buf.push_str(n.to_xml_start_children().as_str());
                    }
                    break;
                } else {
                    while namespaces.len() > *ns_len {
                        namespaces.pop_back();
                    }

                    if c + 1 == count {
                        buf.push_str(n.to_xml_close_empty().as_str());
                    } else {
                        buf.push_str(n.to_xml_close().as_str());
                    }
                    ids.pop();
                }
            }

            if k.start_with(&rf.id) {
                buf.push_str(node.to_xml_open(&mut namespaces).as_str());

                ids.push((count, k, node, namespaces.len()));
                count += 1;
            } else {
                break;
            }
        }

        // closing all open
        while let Some((c, _, node, ns_len)) = ids.pop() {
            while namespaces.len() > ns_len {
                namespaces.pop_back();
            }

            if c + 1 == count {
                buf.push_str(node.to_xml_close_empty().as_str());
            } else {
                buf.push_str(node.to_xml_close().as_str());
            }
        }

        Ok(buf)
    }

    fn typed_value_of_node(&self, rf: &Reference) -> Result<String, String> {
        if let Some(name) = &rf.attr_name {
            if let Some(node) = self.items.get(&rf.id) {
                if let Some(value) = node.attribute_value(name) {
                    Ok(value)
                } else {
                    Err("IO error".to_string()) // TODO better error message
                }
            } else {
                Err("IO error".to_string()) // TODO better error message
            }
        } else {
            let mut result = vec![];
            for (k, v) in self.items.range(&rf.id..) {
                if k.start_with(&rf.id) {
                    result.push(v.typed_value());
                } else {
                    break;
                }
            }
            let data = result.join("");
            Ok(data)
        }
    }

    fn first(&self) -> Option<Reference> {
        if let Some((k, v)) = self.items.first_key_value() {
            Some(self.reference(k.clone(), None))
        } else {
            None
        }
    }

    fn attribute_value(&self, rf: &Reference, name: &QName) -> Option<String> {
        if let Some(attr_name) = &rf.attr_name {
            if attr_name != name {
                return None;
            }
            if let Some(node) = self.items.get(&rf.id) {
                return node.attribute_value(name);
            }
        }
        None
    }

    fn root(&self, rf: &Reference) -> Option<Reference> {
        let top = rf.id.get_level_id(0);
        let id = DLN::level_id(top);
        if let Some(..) = self.items.get(&id) {
            Some(self.reference(id, None))
        } else {
            None
        }
    }

    fn parent(&self, rf: &Reference) -> Option<Reference> {
        if rf.attr_name.is_some() {
            Some(self.reference(rf.id.clone(), None))
        } else if let Some(id) = rf.id.parent() {
            if let Some(..) = self.items.get(&id) {
                Some(self.reference(id, None))
            } else {
                None
            }
        } else {
            None
        }
    }

    fn forward(&self, rf: &Reference, initial_node_sequence: &Option<INS>, axis: &Axis) -> Vec<Reference> {
        println!("forward {:?} {:?} {}", initial_node_sequence, axis, rf.id);
        // println!("{}", self.dump());

        let (rf, all) = if let Some(initial_node) = initial_node_sequence {
            match initial_node {
                INS::Root |
                INS::RootDescendantOrSelf => {
                    if let Some(doc) = self.root(rf) {
                        (doc, initial_node == &INS::RootDescendantOrSelf)
                    } else {
                        panic!("XPDY0050")
                    }
                },
                INS::DescendantOrSelf => {
                    (rf.clone(), true)
                }
            }
        } else {
            (rf.clone(), false)
        };

        if rf.attr_name.is_some() {
            let mut result = Vec::with_capacity(1);
            if axis == &Axis::ForwardSelf || axis == &Axis::ForwardDescendantOrSelf {
                result.push(self.reference(rf.id.clone(), rf.attr_name.clone()));
            }
            return result;
        }

        match axis {
            Axis::ForwardSelf => {
                if all {
                    let mut rfs = Vec::with_capacity(17);
                    let level = rf.id.count_levels() + 1;

                    for (k, node) in self.items.range(&rf.id..) {
                        // println!("{} {:?} {}", k, k, k.start_with(&rf.id));
                        if k.start_with(&rf.id) {
                            if all || k.count_levels() == level {
                                rfs.push(self.reference(k.clone(), None))
                            }
                        } else {
                            break;
                        }
                    }
                    rfs
                } else {
                    let mut rfs = Vec::with_capacity(1);
                    rfs.push(rf);
                    rfs
                }
            }
            Axis::ForwardAttribute => {
                if all {
                    let mut rfs = Vec::with_capacity(17);
                    let level = rf.id.count_levels() + 1;

                    for (k, node) in self.items.range(&rf.id..) {
                        // println!("{} {:?} {}", k, k, k.start_with(&rf.id));
                        if k.start_with(&rf.id) {
                            if all || k.count_levels() == level {
                                if let Some(names) = node.get_attributes() {
                                    for name in names {
                                        rfs.push(self.reference(k.clone(), Some(name)))
                                    }
                                }
                            }
                        } else {
                            break;
                        }
                    }
                    rfs
                } else {
                    if let Some(node) = self.items.get(&rf.id) {
                        if let Some(names) = node.get_attributes() {
                            let mut rfs = Vec::with_capacity(names.len());
                            for name in names {
                                rfs.push(self.reference(rf.id.clone(), Some(name)))
                            }
                            rfs
                        } else {
                            vec![]
                        }
                    } else {
                        vec![]
                    }
                }
            }
            Axis::ForwardChild => self.children(rf, false, all),
            Axis::ForwardDescendant => self.children(rf, false, true),
            Axis::ForwardDescendantOrSelf => self.children(rf, true, true),
            Axis::ForwardFollowing => todo!(),
            Axis::ForwardFollowingSibling => todo!(),
            _ => panic!("internal error")
        }
    }

    fn attributes(&self, rf: &Reference) -> Option<Vec<Reference>> {
        if let Some(node) = self.items.get(&rf.id) {
            if let Some(names) = node.get_attributes() {
                let mut rfs = Vec::with_capacity(names.len());
                for name in names {
                    rfs.push(self.reference(rf.id.clone(), Some(name)));
                }
                Some(rfs)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn get_node(&self, rf: &Reference) -> Option<Box<dyn XMLNode>> {
        if let Some(node) = self.items.get(&rf.id) {
            Some(node.clone())
        } else {
            None
        }
    }

    fn get_type(&self, rf: &Reference) -> Option<NodeType> {
        if let Some(node) = self.items.get(&rf.id) {
            Some(node.get_type())
        } else {
            None
        }
    }

    fn is_namespace(&self, rf: &Reference) -> bool {
        if let Some(name) = &rf.attr_name {
            if let Some(uri) = &name.url {
                uri == &*XML.uri
            } else {
                false
            }
        } else {
            false
        }
    }

    fn is_text(&self, rf: &Reference) -> bool {
        if let Some(node) = self.items.get(&rf.id) {
            node.is_text()
        } else {
            false
        }
    }

    fn is_comment(&self, rf: &Reference) -> bool {
        if let Some(node) = self.items.get(&rf.id) {
            node.is_comment()
        } else {
            false
        }
    }

    fn dump(&self, rf: &Reference) -> String {
        if let Some(node) = self.items.get(&rf.id) {
            format!("{{ storage: {}; {} }}", self.id, node.dump())
        } else {
            "".to_string()
        }
    }
}

impl XMLTreeWriter for InMemoryXMLTree {
    fn init(&mut self, rf: Rc<Mutex<Box<dyn XMLTreeWriter>>>) {
        if self.storage.is_some() {
            panic!("internal error");
        }
        self.storage = Some(rf)
    }

    fn id(&self) -> usize {
        self.id
    }

    fn as_reader(&self) -> Box<&dyn XMLTreeReader> {
        Box::new(self)
    }

    fn link_node(&mut self, rf: &Reference) -> Reference {
        let other = rf.storage.lock().unwrap();
        let other = other.as_reader();
        if let Some(name) = &rf.attr_name {
            let value = other.typed_value_of_node(rf).unwrap();
            self.attribute(name.clone(), value)
        } else {
            let id = self.next_sibling();
            // self.prepare_child();

            let node = Box::new(LinkedNode { id: id.clone(), rf: rf.clone() });

            self.items.insert(id.clone(), node);

            self.reference(id, None)
        }
    }

    fn start_document(&mut self) -> Reference {
        let id = self.next_top_id();

        self.prepare_child();

        let node = Box::new(Document { id: id.clone() });
        self.items.insert(id.clone(), node);

        self.reference(id, None)
    }

    fn end_document(&mut self) -> Option<Reference> {
        self.pop()
    }

    fn start_element(&mut self, mut name: QName) -> Reference {
        let id = self.next_sibling();
        self.prepare_child();

        name = self.resolve(name);

        // TODO on element end check namespace?
        if let Some(prefix) = &name.prefix {
            if let Some(ns) = self.namespaces.get(prefix) {
                if let Some(url) = &name.url {
                    if url != ns {
                        todo!("mismatch ns urls {} vs {}", ns, url)
                    }
                }
                name.url = Some(ns.clone())
            }
        }

        let node = Element::new(id.clone(), name);

        self.items.insert(id.clone(), node);

        self.reference(id, None)
    }

    fn ns(&mut self, prefix: String, url: String) -> Reference {
        let name = QName::ns(&XML, prefix);
        self.attribute(name, url)
    }

    fn attribute(&mut self, mut name: QName, value: String) -> Reference {
        self.add_namespace(&name, &value);

        name = self.resolve(name);

        let size = self.stack.len();
        if size == 0 {
            let id = DLN::level_id(0);
            self.stack.push((id.clone(), self.namespaces.len()));

            let mut node: Box<dyn XMLNode> = Element::empty(id.clone());
            if node.add_attribute(name.clone(), value) {
                self.items.insert(id.clone(), node);
                return self.reference(id, Some(name));
            } else {
                panic!("internal error")
            }
        } else if size >= 2 {
            if let Some((id, ns_len)) = self.stack.get(size - 2) {
                if let Some(node) = self.items.get_mut(id) {
                    if node.add_attribute(name.clone(), value) {
                        return self.reference(id.clone(), Some(name));
                    } else {
                        todo!()
                    }
                }
            }
        }
        todo!()
    }

    fn end_element(&mut self) -> Option<Reference> {
        self.pop()
    }

    fn pi(&mut self, target: QName, content: String) -> Reference {
        let id = self.next_sibling();

        let node = Box::new(PI { id: id.clone(), target, content });

        self.items.insert(id.clone(), node);

        self.reference(id, None)
    }

    fn text(&mut self, content: String) -> Reference {
        let id = self.next_sibling();

        let node = Box::new(Text { id: id.clone(), content });

        self.items.insert(id.clone(), node);

        self.reference(id, None)
    }

    fn comment(&mut self, content: String) -> Reference {
        let id = self.next_sibling();

        let node = Box::new(Comment { id: id.clone(), content });

        self.items.insert(id.clone(), node);

        self.reference(id, None)
    }
}

#[derive(Debug, Clone)]
struct Document {
    id: DLN,
}

impl XMLNode for Document {
    fn id(&self) -> DLN {
        self.id.clone()
    }

    fn name(&self) -> Option<QName> {
        None
    }

    fn target(&self) -> Option<QName> {
        None
    }

    fn content(&self) -> Option<String> {
        None
    }

    fn typed_value(&self) -> String {
        String::new()
    }

    fn attribute_value(&self, name: &QName) -> Option<String> {
        None
    }

    fn add_attribute(&mut self, name: QName, value: String) -> bool {
        false
    }

    fn get_attributes(&self) -> Option<Vec<QName>> {
        None
    }

    fn get_type(&self) -> NodeType {
        NodeType::Document
    }

    fn is_text(&self) -> bool {
        false
    }

    fn is_comment(&self) -> bool {
        false
    }

    fn to_xml_open(&self, namespaces: &mut LinkedHashMap<String, String>) -> String {
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>".to_string()
    }

    fn to_xml_start_children(&self) -> String {
        String::new()
    }

    fn to_xml_close_empty(&self) -> String {
        String::new()
    }

    fn to_xml_close(&self) -> String {
        String::new()
    }

    fn dump(&self) -> String {
        format!("Document {{ id={} }}", self.id)
    }
}

#[derive(Debug, Clone)]
struct Element {
    id: DLN,
    name: Option<QName>,
    attributes: Option<LinkedHashMap<QName, Attribute>>,
}

impl Element {
    pub(crate) fn empty(id: DLN) -> Box<Self> {
        Box::new(Element { id, name: None, attributes: None })
    }

    pub(crate) fn new(id: DLN, name: QName) -> Box<Self> {
        Box::new(Element { id, name: Some(name), attributes: None })
    }
}

impl XMLNode for Element {
    fn id(&self) -> DLN {
        self.id.clone()
    }

    fn name(&self) -> Option<QName> {
        self.name.clone()
    }

    fn target(&self) -> Option<QName> {
        None
    }

    fn content(&self) -> Option<String> {
        None
    }

    fn typed_value(&self) -> String {
        String::new()
    }

    fn attribute_value(&self, name: &QName) -> Option<String> {
        if let Some(attributes) = &self.attributes {
            if let Some(attribute) = attributes.get(&name) {
                return Some(attribute.value.clone());
            }
        }
        None
    }

    fn add_attribute(&mut self, name: QName, value: String) -> bool {
        if self.attributes.is_none() {
            self.attributes = Some(LinkedHashMap::new());
        }

        // workaround: resolve namespace for element name where ns defined
        if let Some(name_prefix) = &name.prefix {
            if name_prefix == "xmlns" {
                if let Some(el_name) = &mut self.name {
                    if el_name.url.is_none() && el_name.prefix.is_some() {
                        if let Some(prefix) = &el_name.prefix {
                            if prefix == &name.local_part {
                                el_name.url = Some(value.clone());
                            }
                        }
                    }
                }
            }
        } else if name.local_part == "xmlns" {
            if let Some(el_name) = &mut self.name {
                if el_name.prefix.is_none() {
                    el_name.url = Some(value.clone());
                }
            }
        }

        if let Some(attributes) = &mut self.attributes {
            attributes.insert(name.clone(), Attribute { name, value } );
        }

        true
    }

    fn get_attributes(&self) -> Option<Vec<QName>> {
        if let Some(attributes) = &self.attributes {
            let mut result = Vec::with_capacity(attributes.len());
            for name in attributes.keys() {
                result.push(name.clone())
            }
            Some(result)
        } else {
            None
        }
    }

    fn get_type(&self) -> NodeType {
        NodeType::Element
    }

    fn is_text(&self) -> bool {
        false
    }

    fn is_comment(&self) -> bool {
        false
    }

    fn to_xml_open(&self, namespaces: &mut LinkedHashMap<String, String>) -> String {
        if let Some(name) = &self.name {
            let mut buf = String::new();
            buf.push_str("<");
            buf.push_str(name.string().as_str());

            if let Some(attrs) = &self.attributes {
                for (name, attr) in attrs {
                    if let Some(prefix) = &name.prefix {
                        if prefix == "xmlns" {
                            namespaces.insert(name.local_part.clone(), attr.value.clone());
                        }
                    } else if name.local_part == "xmlns" {
                        namespaces.insert("".to_string(), attr.value.clone());
                    }
                    buf.push_str(" ");
                    buf.push_str(name.string().as_str());
                    buf.push_str("=\"");
                    buf.push_str(escape_str_attribute(attr.value.as_str()).to_string().as_str());
                    buf.push_str("\"");
                }
            }

            if let Some(prefix) = &name.prefix {
                if namespaces.get(prefix).is_none() {
                    if let Some(url) = &name.url {
                        namespaces.insert(prefix.clone(), url.clone());
                        buf.push_str(" xmlns:");
                        buf.push_str(prefix.as_str());
                        buf.push_str("=\"");
                        buf.push_str(url.as_str());
                        buf.push_str("\"");
                    } else {
                        todo!("raise error")
                    }
                }
            }
            buf
        } else {
            String::new()
        }
    }

    fn to_xml_start_children(&self) -> String {
        ">".to_string()
    }

    fn to_xml_close_empty(&self) -> String {
        "/>".to_string()
    }

    fn to_xml_close(&self) -> String {
        if let Some(name) = &self.name {
            let mut buf = String::new();
            buf.push_str("</");
            buf.push_str(name.string().as_str());
            buf.push_str(">");
            buf
        } else {
            String::new()
        }
    }

    fn dump(&self) -> String {
        let attributes = if let Some(attrs) = &self.attributes {
            let mut buf = String::new();
            for (name, attr) in attrs {
                buf.push_str(" ");
                buf.push_str(format!("{:?}", name).as_str());
                buf.push_str("=");
                buf.push_str(attr.value.as_str());
            }
            buf
        } else {
            String::from("NONE")
        };
        format!("Element {{ id={}; name={:?}; attributes={}; }}",self.id, self.name, attributes)
    }
}

#[derive(Debug, Clone)]
struct Attribute {
    name: QName,
    value: String
}

impl XMLNode for Attribute {
    fn id(&self) -> DLN {
        panic!()
    }

    fn name(&self) -> Option<QName> {
        Some(self.name.clone())
    }

    fn target(&self) -> Option<QName> {
        None
    }

    fn content(&self) -> Option<String> {
        None
    }

    fn typed_value(&self) -> String {
        self.value.clone()
    }

    fn attribute_value(&self, name: &QName) -> Option<String> {
        if &self.name == name {
            Some(escape_str_attribute(self.value.as_str()).to_string())
        } else {
            None
        }
    }

    fn add_attribute(&mut self, name: QName, value: String) -> bool {
        false
    }

    fn get_attributes(&self) -> Option<Vec<QName>> {
        None
    }

    fn get_type(&self) -> NodeType {
        NodeType::Attribute
    }

    fn is_text(&self) -> bool {
        false
    }

    fn is_comment(&self) -> bool {
        false
    }

    fn to_xml_open(&self, namespaces: &mut LinkedHashMap<String, String>) -> String {
        todo!()
    }

    fn to_xml_start_children(&self) -> String {
        todo!()
    }

    fn to_xml_close_empty(&self) -> String {
        todo!()
    }

    fn to_xml_close(&self) -> String {
        todo!()
    }

    fn dump(&self) -> String {
        format!("Attribute {{ name={:?}; value={}; }}", self.name, self.value)
    }
}

#[derive(Debug, Clone)]
struct PI {
    id: DLN,
    target: QName,
    content: String
}

impl XMLNode for PI {
    fn id(&self) -> DLN {
        self.id.clone()
    }

    fn name(&self) -> Option<QName> {
        None
    }

    fn target(&self) -> Option<QName> {
        Some(self.target.clone())
    }

    fn content(&self) -> Option<String> {
        Some(self.content.clone())
    }

    fn typed_value(&self) -> String {
        self.content.clone()
    }

    fn attribute_value(&self, name: &QName) -> Option<String> {
        None
    }

    fn add_attribute(&mut self, name: QName, value: String) -> bool {
        false
    }

    fn get_attributes(&self) -> Option<Vec<QName>> {
        None
    }

    fn get_type(&self) -> NodeType {
        NodeType::PI
    }

    fn is_text(&self) -> bool {
        false
    }

    fn is_comment(&self) -> bool {
        false
    }

    fn to_xml_open(&self, namespaces: &mut LinkedHashMap<String, String>) -> String {
        todo!()
    }

    fn to_xml_start_children(&self) -> String {
        todo!()
    }

    fn to_xml_close_empty(&self) -> String {
        todo!()
    }

    fn to_xml_close(&self) -> String {
        todo!()
    }

    fn dump(&self) -> String {
        format!("PI {{ id={}; target={:?}; content={}; }}", self.id, self.target, self.content)
    }
}

#[derive(Debug, Clone)]
struct Text {
    id: DLN,
    content: String
}

impl XMLNode for Text {
    fn id(&self) -> DLN {
        self.id.clone()
    }

    fn name(&self) -> Option<QName> {
        None
    }

    fn target(&self) -> Option<QName> {
        None
    }

    fn content(&self) -> Option<String> {
        Some(self.content.clone())
    }

    fn typed_value(&self) -> String {
        self.content.clone()
    }

    fn attribute_value(&self, name: &QName) -> Option<String> {
        None
    }

    fn add_attribute(&mut self, name: QName, value: String) -> bool {
        false
    }

    fn get_attributes(&self) -> Option<Vec<QName>> {
        None
    }

    fn get_type(&self) -> NodeType {
        NodeType::Text
    }

    fn is_text(&self) -> bool {
        true
    }

    fn is_comment(&self) -> bool {
        false
    }

    fn to_xml_open(&self, namespaces: &mut LinkedHashMap<String, String>) -> String {
        escape_str_content(self.content.as_str()).to_string()
    }

    fn to_xml_start_children(&self) -> String {
        String::new()
    }

    fn to_xml_close_empty(&self) -> String {
        String::new()
    }

    fn to_xml_close(&self) -> String {
        String::new()
    }

    fn dump(&self) -> String {
        format!("Text {{ id={}; content={:?} }}", self.id, self.content)
    }
}

#[derive(Debug, Clone)]
struct Comment {
    id: DLN,
    content: String
}

impl XMLNode for Comment {
    fn id(&self) -> DLN {
        self.id.clone()
    }

    fn name(&self) -> Option<QName> {
        None
    }

    fn target(&self) -> Option<QName> {
        None
    }

    fn content(&self) -> Option<String> {
        Some(self.content.clone())
    }

    fn typed_value(&self) -> String {
        self.content.clone()
    }

    fn attribute_value(&self, name: &QName) -> Option<String> {
        None
    }

    fn add_attribute(&mut self, name: QName, value: String) -> bool {
        false
    }

    fn get_attributes(&self) -> Option<Vec<QName>> {
        None
    }

    fn get_type(&self) -> NodeType {
        NodeType::Comment
    }

    fn is_text(&self) -> bool {
        false
    }

    fn is_comment(&self) -> bool {
        true
    }

    fn to_xml_open(&self, namespaces: &mut LinkedHashMap<String, String>) -> String {
        "<!--".to_string()
    }

    fn to_xml_start_children(&self) -> String {
        "".to_string()
    }

    fn to_xml_close_empty(&self) -> String {
        "-->".to_string()
    }

    fn to_xml_close(&self) -> String {
        "-->".to_string()
    }

    fn dump(&self) -> String {
        format!("Comment {{ id={}; content={:?} }}", self.id, self.content)
    }
}

#[derive(Debug, Clone)]
struct LinkedNode {
    id: DLN,
    rf: Reference,
}

impl XMLNode for LinkedNode {
    fn id(&self) -> DLN {
        self.id.clone()
    }

    fn name(&self) -> Option<QName> {
        self.rf.name()
    }

    fn target(&self) -> Option<QName> {
        self.rf.target()
    }

    fn content(&self) -> Option<String> {
        self.rf.content()
    }

    fn typed_value(&self) -> String {
        self.rf.to_typed_value().unwrap()
    }

    fn attribute_value(&self, name: &QName) -> Option<String> {
        self.rf.attribute_value(name)
    }

    fn add_attribute(&mut self, name: QName, value: String) -> bool {
        todo!()
    }

    fn get_attributes(&self) -> Option<Vec<QName>> {
        todo!()
    }

    fn get_type(&self) -> NodeType {
        if let Some(node_type) = self.rf.get_type() {
            node_type
        } else {
            todo!("raise error")
        }
    }

    fn is_text(&self) -> bool {
        self.rf.is_text()
    }

    fn is_comment(&self) -> bool {
        self.rf.is_comment()
    }

    fn to_xml_open(&self, namespaces: &mut LinkedHashMap<String, String>) -> String {
        // println!("{} {:?}", self.id, self.rf);
        // println!("{}", self.rf.to_xml(env).unwrap());
        self.rf.to_xml().unwrap()
        // String::new()
    }

    fn to_xml_start_children(&self) -> String {
        String::new()
    }

    fn to_xml_close_empty(&self) -> String {
        String::new()
    }

    fn to_xml_close(&self) -> String {
        String::new()
    }

    fn dump(&self) -> String {
        format!("LinkedNode {{ id={}; rf={:?} }}", self.id, self.rf)
    }
}

enum Value {
    Char(char),
    Str(&'static str)
}

impl Value {
    fn dispatch_for_attribute(c: char) -> Value {
        match c {
            '<'  => Value::Str("&lt;"),
            '>'  => Value::Str("&gt;"),
            '"'  => Value::Str("&quot;"),
            '\'' => Value::Str("&apos;"),
            '&'  => Value::Str("&amp;"),
            '\n' => Value::Str("&#xA;"),
            '\r' => Value::Str("&#xD;"),
            _    => Value::Char(c)
        }
    }

    fn dispatch_for_content(c: char) -> Value {
        match c {
            '<'  => Value::Str("&lt;"),
            '>'  => Value::Str("&gt;"),
            '&'  => Value::Str("&amp;"),
            _    => Value::Char(c)
        }
    }

    fn dispatch_for_pcdata(c: char) -> Value {
        match c {
            '<'  => Value::Str("&lt;"),
            '&'  => Value::Str("&amp;"),
            _    => Value::Char(c)
        }
    }
}

enum Process<'a> {
    Borrowed(&'a str),
    Owned(String)
}

impl<'a> Process<'a> {
    fn process(&mut self, (i, next): (usize, Value)) {
        match next {
            Value::Str(s) => match *self {
                Process::Owned(ref mut o) => o.push_str(s),
                Process::Borrowed(b) => {
                    let mut r = String::with_capacity(b.len() + s.len());
                    r.push_str(&b[..i]);
                    r.push_str(s);
                    *self = Process::Owned(r);
                }
            },
            Value::Char(c) => match *self {
                Process::Borrowed(_) => {}
                Process::Owned(ref mut o) => o.push(c)
            }
        }
    }

    fn into_result(self) -> Cow<'a, str> {
        match self {
            Process::Borrowed(b) => Cow::Borrowed(b),
            Process::Owned(o) => Cow::Owned(o)
        }
    }
}

impl<'a> Extend<(usize, Value)> for Process<'a> {
    fn extend<I: IntoIterator<Item=(usize, Value)>>(&mut self, it: I) {
        for v in it.into_iter() {
            self.process(v);
        }
    }
}

fn escape_str(s: &str, dispatch: fn(char) -> Value) -> Cow<str> {
    let mut p = Process::Borrowed(s);
    p.extend(s.char_indices().map(|(ind, c)| (ind, dispatch(c))));
    p.into_result()
}

#[inline]
pub fn escape_str_attribute(s: &str) -> Cow<str> {
    escape_str(s, Value::dispatch_for_attribute)
}

#[inline]
pub fn escape_str_content(s: &str) -> Cow<str> {
    escape_str(s, Value::dispatch_for_content)
}

#[inline]
pub fn escape_str_pcdata(s: &str) -> Cow<str> {
    escape_str(s, Value::dispatch_for_pcdata)
}