
// nonPositiveInteger(),
// negativeInteger(),
// long(),
// int(),
// short(),
// byte(),

// nonNegativeInteger(),
// unsignedLong(),
// unsignedInt(),
// unsignedShort(),
// unsignedByte(),

// positiveInteger(),

use bigdecimal::BigDecimal;
use ordered_float::OrderedFloat;
use crate::values::Value;

#[derive(Debug, Clone)]
pub struct Integer(pub i128);

impl Integer {
    pub(crate) fn boxed(num: i128) -> Box<dyn Value> {
        Box::new(Integer(num))
    }
}

impl Value for Integer {

}

#[derive(Debug, Clone)]
pub struct Decimal(pub BigDecimal);

impl Decimal {
    pub(crate) fn boxed(num: BigDecimal) -> Box<dyn Value> {
        Box::new(Decimal(num))
    }
}

impl Value for Decimal {

}

#[derive(Debug, Clone)]
pub struct Float(pub OrderedFloat<f32>);

impl Float {
    pub(crate) fn boxed(num: OrderedFloat<f32>) -> Box<dyn Value> {
        Box::new(Float(num))
    }
}

impl Value for Float {

}

#[derive(Debug, Clone)]
pub struct Double(pub OrderedFloat<f64>);

impl Double {
    pub(crate) fn boxed(num: OrderedFloat<f64>) -> Box<dyn Value> {
        Box::new(Double(num))
    }
}

impl Value for Double {

}