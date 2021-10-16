use crate::eval::{Object, EvalResult, atomization, Type, string_to_double, Environment, relax, ErrorInfo};
use crate::parser::errors::ErrorCode;
use ordered_float::OrderedFloat;
use bigdecimal::{BigDecimal, ToPrimitive, FromPrimitive, Zero};
use crate::parser::op::OperatorArithmetic;

// XS_DOUBLE    XS_DOUBLE       > DOUBLE_DOUBLE
// XS_DOUBLE    XS_FLOAT        > DOUBLE_FLOAT
// XS_DOUBLE    XS_DECIMAL      > DOUBLE_DECIMAL
// XS_DOUBLE    XS_INTEGER*     > DOUBLE_DECIMAL

// XS_FLOAT     XS_DOUBLE       > FLOAT_DOUBLE
// XS_FLOAT     XS_FLOAT        > FLOAT_FLOAT
// XS_FLOAT     XS_DECIMAL      > FLOAT_DECIMAL
// XS_FLOAT     XS_INTEGER*     > FLOAT_DECIMAL
// XS_DECIMAL   XS_DOUBLE       > DECIMAL_DOUBLE
// XS_DECIMAL   XS_FLOAT        > DECIMAL_FLOAT
// XS_DECIMAL   XS_DECIMAL      > DECIMAL_DECIMAL
// XS_DECIMAL   XS_INTEGER*     > DECIMAL_DECIMAL
// XS_DATE_TIME XS_DATE_TIME    > DATETIME_DATETIME
// XS_DATE_TIME XS_DURATION     > DATETIME_DURATION
// XS_DURATION  XS_DATE_TIME    > DURATION_DATETIME
// XS_DURATION  XS_DURATION     > DURATION_DURATION
// XS_DURATION  XS_DOUBLE       > DURATION_NUMERIC
// XS_DURATION  XS_FLOAT        > DURATION_NUMERIC
// XS_DURATION  XS_DECIMAL      > DURATION_NUMERIC
// XS_DURATION  XS_INTEGER      > DURATION_NUMERIC

// XS_DOUBLE    XS_DURATION     > NUMERIC_DURATION

// XS_FLOAT     XS_DURATION     > NUMERIC_DURATION
// XS_DECIMAL   XS_DURATION     > NUMERIC_DURATION
// XS_INTEGER*  XS_DURATION     > NUMERIC_DURATION

type OperandReturn = Result<Box<dyn Operand>, ErrorCode>;
trait Operand {
    fn add(&self, rhs: &dyn Operand) -> OperandReturn;
    fn rev_add(&self, rhs: &dyn Operand) -> OperandReturn;
    fn sub(&self, rhs: &dyn Operand) -> OperandReturn;
    fn rev_sub(&self, rhs: &dyn Operand) -> OperandReturn;
    fn mul(&self, rhs: &dyn Operand) -> OperandReturn;
    fn rev_mul(&self, rhs: &dyn Operand) -> OperandReturn;
    fn div(&self, rhs: &dyn Operand) -> OperandReturn;
    fn rev_div(&self, rhs: &dyn Operand) -> OperandReturn;

    fn idiv(&self, rhs: &dyn Operand) -> OperandReturn;
    fn rev_idiv(&self, rhs: &dyn Operand) -> OperandReturn;

    fn remainder(&self, rhs: &dyn Operand) -> OperandReturn;
    fn rev_remainder(&self, rhs: &dyn Operand) -> OperandReturn;

    fn negative(&self) -> OperandReturn;

    fn is_zero(&self) -> bool;

    fn level(&self) -> u8;

    fn to_integer(&self) -> i128;
    fn to_decimal(&self) -> Result<BigDecimal, ErrorCode>;
    fn to_float(&self) -> f32;
    fn to_double(&self) -> f64;

    fn to_atomic(&self) -> Object;
}

struct VDouble {
    number: f64,
}

impl Operand for VDouble {
    fn add(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            let other = rhs.to_double();

            let number = self.number + other;

            Ok(Box::new(VDouble { number }))
        } else {
            rhs.add(self)
        }
    }

    fn rev_add(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            let other = rhs.to_double();

            let number = self.number + other;

            Ok(Box::new(VDouble { number }))
        } else {
            Err(ErrorCode::FOAR0002)
        }
    }

    fn sub(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            let other = rhs.to_double();

            let number = self.number - other;

            Ok(Box::new(VDouble { number }))
        } else {
            rhs.sub(self)
        }
    }

    fn rev_sub(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            let other = rhs.to_double();

            let number = other - self.number;

            Ok(Box::new(VDouble { number }))
        } else {
            Err(ErrorCode::FOAR0002)
        }
    }

    fn mul(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            let other = rhs.to_double();

            let number = self.number * other;

            Ok(Box::new(VDouble { number }))
        } else {
            rhs.rev_mul(self)
        }
    }

    fn rev_mul(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            let other = rhs.to_double();

            let number = self.number * other;

            Ok(Box::new(VDouble { number }))
        } else {
            Err(ErrorCode::FOAR0002)
        }
    }

    fn div(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            let other = rhs.to_double();

            let number = self.number / other;

            Ok(Box::new(VDouble { number }))
        } else {
            rhs.rev_div(self)
        }
    }

    fn rev_div(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            let other = rhs.to_double();

            let number = other / self.number;

            Ok(Box::new(VDouble { number }))
        } else {
            Err(ErrorCode::FOAR0002)
        }
    }

    fn idiv(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            let other = rhs.to_double();

            let number = (self.number / other) as i128;

            Ok(Box::new(VInteger { number }))
        } else {
            rhs.rev_idiv(self)
        }
    }

    fn rev_idiv(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            let other = rhs.to_double();

            let number = (other / self.number) as i128;

            Ok(Box::new(VInteger { number }))
        } else {
            Err(ErrorCode::FOAR0002)
        }
    }

    fn remainder(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            let other = rhs.to_double();

            let number = self.number % other;

            Ok(Box::new(VDouble { number }))
        } else {
            rhs.rev_remainder(self)
        }
    }

    fn rev_remainder(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            let other = rhs.to_double();

            let number = other % self.number;

            Ok(Box::new(VDouble { number }))
        } else {
            Err(ErrorCode::FOAR0002)
        }
    }

    fn negative(&self) -> OperandReturn {
        let number = -self.number;
        Ok(Box::new(VDouble { number }))
    }

    fn is_zero(&self) -> bool {
        self.number.is_zero()
    }

    fn level(&self) -> u8 { 4 }

    fn to_integer(&self) -> i128 { panic!("internal error") }
    fn to_decimal(&self) -> Result<BigDecimal, ErrorCode> { Err(ErrorCode::FOAR0002) }
    fn to_float(&self) -> f32 { panic!("internal error") }
    fn to_double(&self) -> f64 { self.number }

    fn to_atomic(&self) -> Object {
        Object::Atomic(Type::Double(OrderedFloat::from(self.number)))
    }
}

struct VFloat {
    number: f32,
}

impl Operand for VFloat {
    fn add(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            let other = rhs.to_float();

            let number = self.number + other;

            Ok(Box::new(VFloat { number }))
        } else {
            rhs.add(self)
        }
    }

    fn rev_add(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            let other = rhs.to_float();

            let number = self.number + other;

            Ok(Box::new(VFloat { number }))
        } else {
            Err(ErrorCode::FOAR0002)
        }
    }

    fn sub(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            let other = rhs.to_float();

            let number = self.number - other;

            Ok(Box::new(VFloat { number }))
        } else {
            rhs.rev_sub(self)
        }
    }

    fn rev_sub(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            let other = rhs.to_float();

            let number = other - self.number;

            Ok(Box::new(VFloat { number }))
        } else {
            Err(ErrorCode::FOAR0002)
        }
    }

    fn mul(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            let other = rhs.to_float();

            let number = self.number * other;

            Ok(Box::new(VFloat { number }))
        } else {
            rhs.rev_mul(self)
        }
    }

    fn rev_mul(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            let other = rhs.to_float();

            let number = self.number * other;

            Ok(Box::new(VFloat { number }))
        } else {
            Err(ErrorCode::FOAR0002)
        }
    }

    fn div(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            let other = rhs.to_float();

            let number = self.number / other;

            Ok(Box::new(VFloat { number }))
        } else {
            rhs.rev_div(self)
        }
    }

    fn rev_div(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            let other = rhs.to_float();

            let number = other / self.number;

            Ok(Box::new(VFloat { number }))
        } else {
            Err(ErrorCode::FOAR0002)
        }
    }

    fn idiv(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            let other = rhs.to_float();

            let number = (self.number / other) as i128;

            Ok(Box::new(VInteger { number }))
        } else {
            rhs.rev_idiv(self)
        }
    }

    fn rev_idiv(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            let other = rhs.to_float();

            let number = (other / self.number) as i128;

            Ok(Box::new(VInteger { number }))
        } else {
            Err(ErrorCode::FOAR0002)
        }
    }

    fn remainder(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            let other = rhs.to_float();

            let number = self.number % other;

            Ok(Box::new(VFloat { number }))
        } else {
            rhs.rev_remainder(self)
        }
    }

    fn rev_remainder(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            let other = rhs.to_float();

            let number = other % self.number;

            Ok(Box::new(VFloat { number }))
        } else {
            Err(ErrorCode::FOAR0002)
        }
    }

    fn negative(&self) -> OperandReturn {
        let number = -self.number;
        Ok(Box::new(VFloat { number }))
    }

    fn is_zero(&self) -> bool {
        self.number.is_zero()
    }

    fn level(&self) -> u8 { 3 }

    fn to_integer(&self) -> i128 { panic!("internal error") }
    fn to_decimal(&self) -> Result<BigDecimal, ErrorCode> { Err(ErrorCode::FOAR0002) }
    fn to_float(&self) -> f32 { self.number }
    fn to_double(&self) -> f64 { self.number as f64 }

    fn to_atomic(&self) -> Object {
        Object::Atomic(Type::Float(OrderedFloat::from(self.number)))
    }
}

struct VDecimal {
    number: BigDecimal,
}

impl Operand for VDecimal {
    fn add(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            let other = rhs.to_decimal()?;

            let number = &self.number + other;

            Ok(Box::new(VDecimal { number }))
        } else {
            rhs.add(self)
        }
    }

    fn rev_add(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            let other = rhs.to_decimal()?;

            let number = &self.number + other;

            Ok(Box::new(VDecimal { number }))
        } else {
            Err(ErrorCode::FOAR0002)
        }
    }

    fn sub(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            let other = rhs.to_decimal()?;

            let number = &self.number - other;

            Ok(Box::new(VDecimal { number }))
        } else {
            rhs.rev_sub(self)
        }
    }

    fn rev_sub(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            let other = rhs.to_decimal()?;

            let number = other - &self.number;

            Ok(Box::new(VDecimal { number }))
        } else {
            Err(ErrorCode::FOAR0002)
        }
    }

    fn mul(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            let other = rhs.to_decimal()?;

            let number = &self.number * other;

            Ok(Box::new(VDecimal { number }))
        } else {
            rhs.rev_mul(self)
        }
    }

    fn rev_mul(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            let other = rhs.to_decimal()?;

            let number = &self.number * other;

            Ok(Box::new(VDecimal { number }))
        } else {
            Err(ErrorCode::FOAR0002)
        }
    }

    fn div(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            if rhs.is_zero() {
                Err(ErrorCode::FOAR0001)
            } else {
                let other = rhs.to_decimal()?;

                let number = &self.number / other;

                Ok(Box::new(VDecimal { number }))
            }
        } else {
            rhs.rev_div(self)
        }
    }

    fn rev_div(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            if self.is_zero() {
                Err(ErrorCode::FOAR0001)
            } else {
                let other = rhs.to_decimal()?;

                let number = other / &self.number;

                Ok(Box::new(VDecimal { number }))
            }
        } else {
            Err(ErrorCode::FOAR0002)
        }
    }

    fn idiv(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            if rhs.is_zero() {
                Err(ErrorCode::FOAR0001)
            } else {
                let other = rhs.to_decimal()?;

                let number = (&self.number / other).round(0).to_i128().unwrap();

                Ok(Box::new(VInteger { number }))
            }
        } else {
            rhs.rev_idiv(self)
        }
    }

    fn rev_idiv(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            if self.is_zero() {
                Err(ErrorCode::FOAR0001)
            } else {
                let other = rhs.to_decimal()?;

                let number = (other / &self.number).round(0).to_i128().unwrap();

                Ok(Box::new(VInteger { number }))
            }
        } else {
            Err(ErrorCode::FOAR0002)
        }
    }

    fn remainder(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            if rhs.is_zero() {
                Err(ErrorCode::FOAR0001)
            } else {
                let other = rhs.to_decimal()?;

                let number = &self.number % other;

                Ok(Box::new(VDecimal { number }))
            }
        } else {
            rhs.rev_div(self)
        }
    }

    fn rev_remainder(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            if self.is_zero() {
                Err(ErrorCode::FOAR0001)
            } else {
                let other = rhs.to_decimal()?;

                let number = other % &self.number;

                Ok(Box::new(VDecimal { number }))
            }
        } else {
            Err(ErrorCode::FOAR0002)
        }
    }

    fn negative(&self) -> OperandReturn {
        let number = -self.number.clone();
        Ok(Box::new(VDecimal { number }))
    }

    fn is_zero(&self) -> bool {
        self.number.is_zero()
    }

    fn level(&self) -> u8 { 2 }

    fn to_integer(&self) -> i128 { panic!("internal error") }
    fn to_decimal(&self) -> Result<BigDecimal, ErrorCode> { Ok(self.number.clone()) }
    fn to_float(&self) -> f32 { self.number.to_f32().unwrap() } // TODO: code it
    fn to_double(&self) -> f64 { self.number.to_f64().unwrap() } // TODO: code it

    fn to_atomic(&self) -> Object {
        Object::Atomic(Type::Decimal(self.number.normalized()))
    }
}

struct VInteger {
    number: i128
}

impl Operand for VInteger {
    fn add(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            let other = rhs.to_integer();

            match self.number.checked_add(other) {
                Some(number) => Ok(Box::new(VInteger { number })),
                None => Err(ErrorCode::FOAR0002)
            }
        } else {
            rhs.rev_add(self)
        }
    }

    fn rev_add(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            let other = rhs.to_integer();

            match self.number.checked_add(other) {
                Some(number) => Ok(Box::new(VInteger { number })),
                None => Err(ErrorCode::FOAR0002)
            }
        } else {
            Err(ErrorCode::FOAR0002)
        }
    }

    fn sub(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            let other = rhs.to_integer();

            match self.number.checked_sub(other) {
                Some(number) => Ok(Box::new(VInteger { number })),
                None => Err(ErrorCode::FOAR0002)
            }
        } else {
            rhs.rev_sub(self)
        }
    }

    fn rev_sub(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            let other = rhs.to_integer();

            match other.checked_sub(self.number) {
                Some(number) => Ok(Box::new(VInteger { number })),
                None => Err(ErrorCode::FOAR0002)
            }
        } else {
            Err(ErrorCode::FOAR0002)
        }
    }

    fn mul(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            let other = rhs.to_integer();

            match self.number.checked_mul(other) {
                Some(number) => Ok(Box::new(VInteger { number })),
                None => Err(ErrorCode::FOAR0002)
            }
        } else {
            rhs.rev_mul(self)
        }
    }

    fn rev_mul(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            let other = rhs.to_integer();

            match self.number.checked_mul(other) {
                Some(number) => Ok(Box::new(VInteger { number })),
                None => Err(ErrorCode::FOAR0002)
            }
        } else {
            Err(ErrorCode::FOAR0002)
        }
    }

    fn div(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            if rhs.is_zero() {
                Err(ErrorCode::FOAR0001)
            } else {
                let self_number = self.to_decimal()?;
                let other = rhs.to_decimal()?;

                let number = self_number / other;

                Ok(Box::new(VDecimal { number }))
            }
        } else {
            rhs.rev_div(self)
        }
    }

    fn rev_div(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            if rhs.is_zero() {
                Err(ErrorCode::FOAR0001)
            } else {
                let self_number = self.to_decimal()?;
                let other = rhs.to_decimal()?;

                let number = other / self_number;

                Ok(Box::new(VDecimal { number }))
            }
        } else {
            Err(ErrorCode::FOAR0002)
        }
    }

    fn idiv(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            if rhs.is_zero() {
                Err(ErrorCode::FOAR0001)
            } else {
                let self_number = self.to_decimal()?;
                let other = rhs.to_decimal()?;

                let number = (self_number / other).round(0).to_i128().unwrap();

                Ok(Box::new(VInteger { number }))
            }
        } else {
            rhs.rev_idiv(self)
        }
    }

    fn rev_idiv(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            if rhs.is_zero() {
                Err(ErrorCode::FOAR0001)
            } else {
                let self_number = self.to_decimal()?;
                let other = rhs.to_decimal()?;

                let number = (other / self_number).round(0).to_i128().unwrap();

                Ok(Box::new(VInteger { number }))
            }
        } else {
            Err(ErrorCode::FOAR0002)
        }
    }

    fn remainder(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            if rhs.is_zero() {
                Err(ErrorCode::FOAR0001)
            } else {
                let self_number = self.to_integer();
                let other = rhs.to_integer();

                let number = self_number % other;

                Ok(Box::new(VInteger { number }))
            }
        } else {
            rhs.rev_div(self)
        }
    }

    fn rev_remainder(&self, rhs: &dyn Operand) -> OperandReturn {
        if self.level() >= rhs.level() {
            if rhs.is_zero() {
                Err(ErrorCode::FOAR0001)
            } else {
                let self_number = self.to_integer();
                let other = rhs.to_integer();

                let number = other % self_number;

                Ok(Box::new(VInteger { number }))
            }
        } else {
            Err(ErrorCode::FOAR0002)
        }
    }

    fn negative(&self) -> OperandReturn {
        let number = -self.number;
        Ok(Box::new(VInteger { number }))
    }

    fn is_zero(&self) -> bool {
        self.number.is_zero()
    }

    fn level(&self) -> u8 { 1 }

    fn to_integer(&self) -> i128 { self.number }
    fn to_decimal(&self) -> Result<BigDecimal, ErrorCode> {
        match BigDecimal::from_i128(self.number) {
            Some(number) => Ok(number),
            None => Err(ErrorCode::FOAR0002)
        }
    }
    fn to_float(&self) -> f32 { self.number as f32 }
    fn to_double(&self) -> f64 { self.number as f64 }

    fn to_atomic(&self) -> Object {
        Object::Atomic(Type::Integer(self.number))
    }
}

struct VDateTime {
}

struct VDuration {
}

// TODO: delete
pub fn object_to_items(object: &Object) -> Vec<Object> {
    match object {
        // Object::ForBinding { values, ..} => {
        //     object_to_iterator(values)
        // },
        _ => vec![object.clone()]
    }
}

pub fn eval_arithmetic<'a>(env: Box<Environment<'a>>, operator: OperatorArithmetic, left: Object, right: Object) -> EvalResult<'a> {

    let mut current_env = env;
    let mut result = vec![];

    let it_left = object_to_items(&left);
    for l in it_left {

        let it_right = object_to_items(&right);
        for r in it_right {

            let (new_env, value) = eval_arithmetic_item(current_env, operator.clone(), l.clone(), r)?;
            current_env = new_env;

            result.push(value);
        }
    }

    relax(current_env, result)
}

pub fn eval_arithmetic_item<'a>(env: Box<Environment<'a>>, operator: OperatorArithmetic, left: Object, right: Object) -> EvalResult<'a> {
    let left = match atomization(&env, left) {
        Ok(v) => v,
        Err(e) => return Err(e)
    };
    let right = match atomization(&env, right) {
        Ok(v) => v,
        Err(e) => return Err(e)
    };

    let left_value = match into_type(left) {
        Ok(v) => v,
        Err(e) => return Err(e)
    };
    let right_value = match into_type(right) {
        Ok(v) => v,
        Err(e) => return Err(e)
    };

    let result = match operator {
        OperatorArithmetic::Plus => left_value.add(&*right_value),
        OperatorArithmetic::Minus => left_value.sub(&*right_value),
        OperatorArithmetic::Multiply => left_value.mul(&*right_value),
        OperatorArithmetic::Divide => left_value.div(&*right_value),
        OperatorArithmetic::IDivide => left_value.idiv(&*right_value),
        OperatorArithmetic::Mod => left_value.remainder(&*right_value),
    };

    match result {
        Ok(number) => {
            Ok((env, number.to_atomic()))
        },
        Err(code) => Err((code, String::from("TODO")))
    }
}

pub fn eval_unary(env: Box<Environment>, object: Object, sign_is_positive: bool) -> EvalResult {
    let object = match atomization(&env, object) {
        Ok(v) => v,
        Err(e) => return Err(e)
    };

    let value = match into_type(object) {
        Ok(v) => v,
        Err(e) => return Err(e)
    };

    if sign_is_positive {
        let obj = value.to_atomic();
        Ok((env, obj))
    } else {
        match value.negative() {
            Ok(number) => {
                Ok((env, number.to_atomic()))
            },
            Err(code) => Err((code, String::from("TODO")))
        }
    }
}

fn into_type(obj: Object) -> Result<Box<dyn Operand>, ErrorInfo> {
    match obj {
        Object::Atomic(t) => {
            match t {
                Type::Untyped(str) => {
                    match string_to_double(&str) {
                        Ok(Object::Atomic(Type::Double(number))) => {
                            Ok(Box::new(VDouble { number: number.into_inner() }))
                        },
                        _ => Err((ErrorCode::FORG0001, String::from("TODO")))
                    }
                }
                // Type::dateTime() => {}
                // Type::dateTimeStamp() => {}
                // Type::Date { .. } => {}
                // Type::Time { .. } => {}
                // Type::Duration { .. } => {}
                // Type::YearMonthDuration { .. } => {}
                // Type::DayTimeDuration { .. } => {}
                Type::Integer(number) => Ok(Box::new( VInteger { number } )),
                Type::Decimal(number) => Ok(Box::new( VDecimal { number } )),
                Type::Float(number) => Ok(Box::new( VFloat { number: number.into_inner() } )),
                Type::Double(number) => Ok(Box::new( VDouble { number: number.into_inner() } )),
                // Type::nonPositiveInteger() => {}
                // Type::negativeInteger() => {}
                // Type::long() => {}
                // Type::int() => {}
                // Type::short() => {}
                // Type::byte() => {}
                // Type::nonNegativeInteger() => {}
                // Type::unsignedLong() => {}
                // Type::unsignedInt() => {}
                // Type::unsignedShort() => {}
                // Type::unsignedByte() => {}
                // Type::positiveInteger() => {}
                // Type::gYearMonth() => {}
                // Type::gYear() => {}
                // Type::gMonthDay() => {}
                // Type::gDay() => {}
                // Type::gMonth() => {}
                // Type::String(_) => {}
                // Type::NormalizedString(_) => {}
                // Type::Token(_) => {}
                // Type::language(_) => {}
                // Type::NMTOKEN(_) => {}
                // Type::Name(_) => {}
                // Type::NCName(_) => {}
                // Type::ID(_) => {}
                // Type::IDREF(_) => {}
                // Type::ENTITY(_) => {}
                // Type::Boolean(_) => {}
                // Type::base64Binary() => {}
                // Type::hexBinary() => {}
                // Type::AnyURI(_) => {}
                // Type::QName() => {}
                // Type::NOTATION() => {}
                _ => Err((ErrorCode::XPTY0004, String::from("TODO")))
            }
        },
        _ => Err((ErrorCode::XPTY0004, String::from("TODO")))
    }
}