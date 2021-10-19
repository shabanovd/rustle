use crate::eval::{Environment, ErrorInfo, Object, Type};
use crate::values::*;
use crate::eval::expression::{NodeTest, Expression};
use crate::tree::Reference;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ItemType {
    SequenceEmpty,
    Item,
    AtomicOrUnionType(QName),

    AnyKind,
    Document(Option<Box<ItemType>>),
    Element,
    Attribute,
    Text,
    Comment,
    NamespaceNode,
    PI,

    SchemaAttribute(QName),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum OccurrenceIndicator {
    ExactlyOne,
    ZeroOrOne, // ?
    ZeroOrMore, // *
    OneOrMore, // +
}

#[derive(Clone, Debug, PartialEq, Eq)]
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
            ItemType::AtomicOrUnionType(name) => {
                match obj {
                    Object::Empty => {
                        self.occurrence_indicator == OccurrenceIndicator::ZeroOrMore
                            || self.occurrence_indicator == OccurrenceIndicator::ZeroOrOne
                    },
                    Object::Atomic(Type::String(..)) => name == &*XS_STRING,
                    Object::Atomic(Type::NormalizedString(..)) => name == &*XS_STRING,
                    Object::Atomic(Type::Integer(..)) => name == &*XS_INTEGER,
                    Object::Atomic(Type::Decimal{..}) => name == &*XS_DECIMAL,
                    Object::Atomic(Type::Float{..}) => name == &*XS_FLOAT,
                    Object::Atomic(Type::Double{..}) => name == &*XS_DOUBLE,
                    Object::Atomic(Type::Untyped(..)) => name == &*XS_UNTYPED_ATOMIC,
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
    fn test_node(&self, env: &Box<Environment>, rf: &Reference) -> bool {
        todo!()
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
    fn test_node(&self, env: &Box<Environment>, rf: &Reference) -> bool {
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
    fn test_node(&self, env: &Box<Environment>, rf: &Reference) -> bool {
        todo!()
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
    fn test_node(&self, env: &Box<Environment>, rf: &Reference) -> bool {
        todo!()
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
    fn test_node(&self, env: &Box<Environment>, rf: &Reference) -> bool {
        todo!()
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
    fn test_node(&self, env: &Box<Environment>, rf: &Reference) -> bool {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ElementTest {
}

impl ElementTest {
    pub(crate) fn boxed() -> Box<dyn NodeTest> {
        Box::new(ElementTest { })
    }
}

impl NodeTest for ElementTest {
    fn test_node(&self, env: &Box<Environment>, rf: &Reference) -> bool {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct AttributeTest {
}

impl AttributeTest {
    pub(crate) fn boxed() -> Box<dyn NodeTest> {
        Box::new(AttributeTest { })
    }
}

impl NodeTest for AttributeTest {
    fn test_node(&self, env: &Box<Environment>, rf: &Reference) -> bool {
        todo!()
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
    fn test_node(&self, env: &Box<Environment>, rf: &Reference) -> bool {
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
    fn test_node(&self, env: &Box<Environment>, rf: &Reference) -> bool {
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
    fn test_node(&self, env: &Box<Environment>, rf: &Reference) -> bool {
        if let Some(name) = rf.name(env) {
            (self.name.local_part == "*" || self.name.local_part == name.local_part)
                && (self.name.url == Some(String::from("*")) || self.name.url == name.url)
        } else {
            false
        }
    }
}