use crate::eval::{ErrorInfo, Object};
use crate::values::*;
use crate::eval::expression::{NodeTest, Expression};
use crate::eval::sequence_type::ItemType::AtomicOrUnionType;
use crate::tree::Reference;

#[derive(Debug, Clone)]
pub enum ItemType {
    SequenceEmpty,
    Item,
    AtomicOrUnionType(QName),

    AnyKind,
    Node(Box<dyn NodeTest>),
    // Document(Option<Box<ItemType>>),
    // Element,
    // Attribute,
    // Text,
    // Comment,
    // NamespaceNode,
    // PI,

    SchemaAttribute(QName),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum OccurrenceIndicator {
    ExactlyOne,
    ZeroOrOne, // ?
    ZeroOrMore, // *
    OneOrMore, // +
}

#[derive(Clone, Debug)]
pub struct SequenceType {
    pub(crate) item_type: ItemType,
    pub(crate) occurrence_indicator: OccurrenceIndicator
}

impl SequenceType {
    pub fn cascade(&self, obj: Object) -> Result<Object, ErrorInfo> {
        todo!()
    }

    pub fn is_castable(&self, obj: &Object) -> Result<bool, ErrorInfo> {
        let result = match &self.item_type {
            AtomicOrUnionType(name) => {
                match obj {
                    Object::Empty => {
                        self.occurrence_indicator == OccurrenceIndicator::ZeroOrMore
                            || self.occurrence_indicator == OccurrenceIndicator::ZeroOrOne
                    },
                    Object::Atomic(Str(..)) => name == &*XS_STRING,
                    Object::Atomic(NormalizedString(..)) => name == &*XS_STRING,
                    Object::Atomic(Integer(..)) => name == &*XS_INTEGER,
                    Object::Atomic(Decimal{..}) => name == &*XS_DECIMAL,
                    Object::Atomic(Float{..}) => name == &*XS_FLOAT,
                    Object::Atomic(Double{..}) => name == &*XS_DOUBLE,
                    Object::Atomic(Untyped(..)) => name == &*XS_UNTYPED_ATOMIC,
                    _ => panic!("TODO: {:?}", obj)
                }
            },
            _ => panic!("TODO: {:?}", self.item_type)
        };
        Ok(result)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct AnyKindTest { }

impl AnyKindTest {
    pub(crate) fn boxed() -> Box<dyn NodeTest> {
        Box::new(AnyKindTest { })
    }
}

impl NodeTest for AnyKindTest {
    fn test_node(&self, rf: &Reference) -> bool {
        true
    }
}

#[derive(Clone, Debug)]
pub(crate) struct DocumentTest {
    child: Option<Box<dyn NodeTest>>
}

impl DocumentTest {
    pub(crate) fn boxed(child: Option<Box<dyn NodeTest>>) -> Box<dyn NodeTest> {
        Box::new(DocumentTest { child })
    }
}

impl NodeTest for DocumentTest {
    fn test_node(&self, rf: &Reference) -> bool {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct TextTest {
}

impl TextTest {
    pub(crate) fn boxed() -> Box<dyn NodeTest> {
        Box::new(TextTest { })
    }
}

impl NodeTest for TextTest {
    fn test_node(&self, rf: &Reference) -> bool {
        rf.is_text()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct CommentTest {
}

impl CommentTest {
    pub(crate) fn boxed() -> Box<dyn NodeTest> {
        Box::new(CommentTest { })
    }
}

impl NodeTest for CommentTest {
    fn test_node(&self, rf: &Reference) -> bool {
        rf.is_comment()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct NamespaceNodeTest {
}

impl NamespaceNodeTest {
    pub(crate) fn boxed() -> Box<dyn NodeTest> {
        Box::new(NamespaceNodeTest { })
    }
}

impl NodeTest for NamespaceNodeTest {
    fn test_node(&self, rf: &Reference) -> bool {
        rf.is_namespace()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PITest {
    content: Option<Box<dyn Expression>>
}

impl PITest {
    pub(crate) fn boxed(content: Option<Box<dyn Expression>>) -> Box<dyn NodeTest> {
        Box::new(PITest { content })
    }
}

impl NodeTest for PITest {
    fn test_node(&self, rf: &Reference) -> bool {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ElementTest {
    name: Option<QName>,
    type_annotation: Option<QName>,
}

impl ElementTest {
    pub(crate) fn boxed(name: Option<QName>, type_annotation: Option<QName>) -> Box<dyn NodeTest> {
        Box::new(ElementTest { name, type_annotation })
    }
}

impl NodeTest for ElementTest {
    fn test_node(&self, rf: &Reference) -> bool {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct AttributeTest {
    name: Option<QName>,
    type_annotation: Option<QName>,
}

impl AttributeTest {
    pub(crate) fn boxed(name: Option<QName>, type_annotation: Option<QName>) -> Box<dyn NodeTest> {
        Box::new(AttributeTest { name: None, type_annotation: None })
    }
}

impl NodeTest for AttributeTest {
    fn test_node(&self, rf: &Reference) -> bool {
        if let Some(rf_name) = &rf.attr_name {
            if let Some(name) = &self.name {
                if rf_name == name {
                    if let Some(type_annotation) = &self.type_annotation {
                        todo!()
                    } else {
                        true
                    }
                } else {
                    false
                }
            } else {
                true
            }
        } else {
            false
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SchemaElementTest {
    name: QName
}

impl SchemaElementTest {
    pub(crate) fn boxed(name: QName) -> Box<dyn NodeTest> {
        Box::new(SchemaElementTest { name })
    }
}

impl NodeTest for SchemaElementTest {
    fn test_node(&self, rf: &Reference) -> bool {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SchemaAttributeTest {
    name: QName
}

impl SchemaAttributeTest {
    pub(crate) fn boxed(name: QName) -> Box<dyn NodeTest> {
        Box::new(SchemaAttributeTest { name })
    }
}

impl NodeTest for SchemaAttributeTest {
    fn test_node(&self, rf: &Reference) -> bool {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct NameTest { pub(crate) name: QName }

impl NameTest {
    pub(crate) fn boxed(name: QName) -> Box<dyn NodeTest> {
        Box::new(NameTest { name })
    }
}

impl NodeTest for NameTest {
    fn test_node(&self, rf: &Reference) -> bool {
        if let Some(name) = rf.name() {
            (self.name.local_part == "*" || self.name.local_part == name.local_part)
                && (self.name.url == Some(String::from("*")) || self.name.url == name.url)
        } else {
            false
        }
    }
}