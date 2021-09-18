use crate::eval::{Object, Environment, Type, eval_statements, object_to_iterator, comparison};
use crate::parser::parse;
use crate::value::{resolve_element_qname, QName};
use crate::serialization::object_to_string;
use crate::fns::object_to_bool;


pub(crate) fn eval_on_spec(spec: &str, input: &str) -> Result<Object, String> {
    match spec {
        "XQ10+" | "XP30+ XQ30+" | "XQ31+" | "XP31+ XQ31+" => {
            eval(input)
        }
        _ => panic!("unsupported spec {}", spec)
    }
}

pub(crate) fn eval(input: &str) -> Result<Object, String> {
    println!("script: {:?}", input);

    let parsed = parse(input);
    if parsed.is_ok() {
        let program = parsed.unwrap();

        println!("{:#?}", program);

        let env = Environment::new();
        let check = eval_statements(program, Box::new(env));
        match check {
            Ok(obj) => Ok(obj),
            Err((error_code, ..)) => {
                let code = error_code.as_ref();
                Err(String::from(code))
            }
        }
    } else {
        println!("error: {:#?}", parsed);

        let msg = match parsed {
            Err(error) => {
                let code = error.as_ref();
                String::from(code)
            }
            _ => format!("error {:?}", parsed)
        };
        Err(msg)
    }
}

pub(crate) fn check_assert(result: &Result<Object, String>, check: &str) {
    let check_result = eval_assert(result, check);
    let check_result = object_to_bool(&check_result);

    if !check_result {
        assert_eq!(format!("{:?}", result), check);
    }
}

pub(crate) fn bool_check_assert(result: &Result<Object, String>, check: &str) -> bool {
    let check_result = eval_assert(result, check);
    let check_result = object_to_bool(&check_result);

    !check_result
}

fn eval_assert(result: &Result<Object, String>, check: &str) -> Object {

    let parsed = parse(check);
    if parsed.is_ok() {
        let program = parsed.unwrap();

        let mut env = Box::new(Environment::new());

        let name = resolve_element_qname(QName::local_part("result"), &env);
        env.set(name, result.as_ref().unwrap().clone());

        match eval_statements(program, env) {
            Ok(obj) => obj,
            Err((code, msg)) => {
                panic!("error {:?} {:?}", code, msg);
            }
        }
    } else {
        panic!("error {:?}", parsed);
    }
}

pub(crate) fn check_assert_eq(result: &Result<Object, String>, check: &str) {
    let expected = eval(check).unwrap();
    match comparison::eq(&expected, &result.as_ref().unwrap()) {
        Ok(v) => if !v { assert_eq!(expected, result.as_ref().unwrap().clone()) },
        Err(code) => panic!("Error {:?}", code)
    }
}

pub(crate) fn bool_check_assert_eq(result: &Result<Object, String>, check: &str) -> bool {
    let expected = eval(check).unwrap();
    match comparison::eq(&expected, result.as_ref().unwrap()) {
        Ok(v) => !v,
        Err(code) => panic!("Error {:?}", code)
    }
}

pub(crate) fn check_assert_count(result: &Result<Object, String>, check: &str) {
    let actual = object_to_iterator(&result.as_ref().unwrap()).len();
    let expected: usize = check.parse().unwrap();
    assert_eq!(expected, actual)
}

pub(crate) fn bool_check_assert_count(result: &Result<Object, String>, check: &str) -> bool {
    let actual = object_to_iterator(&result.as_ref().unwrap()).len();
    let expected: usize = check.parse().unwrap();
    expected == actual
}

pub(crate) fn check_assert_deep_eq(result: &Result<Object, String>, check: &str) {
    let expected = eval(check).unwrap();
    assert_eq!(expected, result.as_ref().unwrap().clone());
}

pub(crate) fn bool_check_assert_deep_eq(result: &Result<Object, String>, check: &str) -> bool {
    let expected = eval(check).unwrap();
    expected == result.as_ref().unwrap().clone()
}

pub(crate) fn check_assert_permutation(result: &Result<Object, String>, check: &str) {
    println!("result: {:?}\ncheck: {:?}", result, check);
    todo!()
}

pub(crate) fn bool_check_assert_permutation(result: &Result<Object, String>, check: &str) -> bool {
    println!("result: {:?}\ncheck: {:?}", result, check);
    todo!()
}

pub(crate) fn check_assert_xml(result: &Result<Object, String>, check: &str) {
    let actual = object_to_string(result.as_ref().unwrap());
    assert_eq!(check, actual);
}

pub(crate) fn bool_check_assert_xml(result: &Result<Object, String>, check: &str) -> bool {
    let actual = object_to_string(result.as_ref().unwrap());
    check == actual
}

pub(crate) fn check_assert_type(result: &Result<Object, String>, check: &str) {
    if let Some(err) = _check_assert_type(result, check) {
        assert_eq!(err, format!("{:?}", result));
    }
}

pub(crate) fn bool_check_assert_type(result: &Result<Object, String>, check: &str) -> bool {
    _check_assert_type(result, check).is_some()
}

pub(crate) fn _check_assert_type(result: &Result<Object, String>, check: &str) -> Option<String> {
    let result = result.as_ref().unwrap();
    if check == "array(*)" {
        match result {
            Object::Array(..) => None,
            _ => {
                Some(String::from("not array(*)"))
            }
        }
    } else if check == "array(xs:string)" {
        match result {
            Object::Array(items) => {
                if items.len() == 0 {
                    Some(String::from("not array(xs:string)"))
                } else {
                    for item in items {
                        match item {
                            Object::Atomic(Type::String(..)) => {},
                            _ => {
                                return Some(String::from("not array(xs:string)"));
                            }
                        }
                    }
                    None
                }
            },
            _ => {
                Some(String::from("not array(xs:string)"))
            }
        }
    } else if check == "xs:integer" {
        match result {
            Object::Atomic(Type::Integer(..)) => None,
            _ => {
                Some(String::from("not xs:integer"))
            }
        }
    } else {
        todo!()
    }
}

pub(crate) fn check_assert_string_value(result: &Result<Object, String>, check: &str) {
    let actual = object_to_string(result.as_ref().unwrap());
    assert_eq!(check, actual);
}

pub(crate) fn bool_check_assert_string_value(result: &Result<Object, String>, check: &str) -> bool {
    let actual = object_to_string(result.as_ref().unwrap());
    check == actual
}

pub(crate) fn check_assert_empty(result: &Result<Object, String>) {
    if result.as_ref().unwrap() != &Object::Empty {
        assert_eq!("not empty", format!("{:?}", result));
    }
}

pub(crate) fn bool_check_assert_empty(result: &Result<Object, String>) -> bool {
    result.as_ref().unwrap() != &Object::Empty
}

pub(crate) fn check_assert_true(result: &Result<Object, String>) {
    if result.as_ref().unwrap() != &Object::Atomic(Type::Boolean(true)) {
        assert_eq!("not true", format!("{:?}", result));
    }
}

pub(crate) fn bool_check_assert_true(result: &Result<Object, String>) -> bool {
    result.as_ref().unwrap() != &Object::Atomic(Type::Boolean(true))
}

pub(crate) fn check_assert_false(result: &Result<Object, String>) {
    if result.as_ref().unwrap() != &Object::Atomic(Type::Boolean(false)) {
        assert_eq!("not false", format!("{:?}", result));
    }
}

pub(crate) fn bool_check_assert_false(result: &Result<Object, String>) -> bool {
    result.as_ref().unwrap() != &Object::Atomic(Type::Boolean(false))
}

pub(crate) fn check_error(result: &Result<Object, String>, expected_code: &str) {
    match result {
        Err(code) => {
            assert_eq!(*code, expected_code)
        },
        _ => {
            assert_eq!("not error", format!("{:?}", result));
        }
    }
}

pub(crate) fn bool_check_error(result: &Result<Object, String>, expected_code: &str) -> bool {
    match result {
        Err(code) => {
            code == expected_code
        },
        _ => {
            false
        }
    }
}
