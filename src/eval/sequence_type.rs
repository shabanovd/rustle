use crate::parser::op::{ItemType, OccurrenceIndicator};
use crate::eval::{Object, Type};
use crate::parser::errors::ErrorCode;
use crate::values::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SequenceType {
    pub(crate) item_type: ItemType,
    pub(crate) occurrence_indicator: OccurrenceIndicator
}

impl SequenceType {
    pub fn cascade(&self, obj: Object) -> Result<Object, (ErrorCode, String)> {
        todo!()
    }

    pub fn is_castable(&self, obj: &Object) -> Result<bool, (ErrorCode, String)> {
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
                    _ => panic!("TODO: {:?}", obj)
                }
            },
            _ => panic!("TODO: {:?}", self.item_type)
        };
        Ok(result)
    }
}