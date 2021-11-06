
//     TODO CharRef { representation: Representation, reference: u32 }, ?

use crate::values::Value;

#[derive(Debug, Clone)]
pub struct Untyped(pub String);

impl Untyped {
    pub(crate) fn boxed(data: String) -> Box<dyn Value> {
        Box::new(Untyped(data))
    }
}

impl Value for Untyped {

}

#[derive(Debug, Clone)]
pub struct Str(pub String);

impl Str {
    pub(crate) fn boxed(data: String) -> Box<dyn Value> {
        Box::new(Str(data))
    }
}

impl Value for Str {

}

#[derive(Debug, Clone)]
pub struct NormalizedString(pub String);

impl Value for NormalizedString {

}

#[derive(Debug, Clone)]
pub struct Token(pub String);

impl Value for Token {

}

#[derive(Debug, Clone)]
pub struct Language(pub String);

impl Value for Language {

}

#[derive(Debug, Clone)]
pub struct NMTOKEN(pub String);

impl Value for NMTOKEN {

}

#[derive(Debug, Clone)]
pub struct NCName(pub String);

impl NCName {
    pub(crate) fn boxed(name: String) -> Box<dyn Value> {
        Box::new(NCName(name))
    }
}

impl Value for NCName {

}

#[derive(Debug, Clone)]
pub struct ID(pub String);

impl Value for ID {

}

#[derive(Debug, Clone)]
pub struct IDREF(pub String);

impl Value for IDREF {

}

#[derive(Debug, Clone)]
pub struct ENTITY(pub String);

impl Value for ENTITY {

}

#[derive(Debug, Clone)]
pub struct AnyURI(pub String);

impl AnyURI {
    pub(crate) fn boxed(uri: String) -> Box<dyn Value> {
        Box::new(AnyURI(uri))
    }
}

impl Value for AnyURI {

}

#[derive(Debug, Clone)]
pub struct NOTATION(pub String);

impl Value for NOTATION {

}