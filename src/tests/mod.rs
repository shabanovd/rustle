use crate::eval::{Object, Environment, Type, eval_statements, object_to_bool, object_to_iterator};
use crate::parser::parse;
use crate::value::{resolve_element_qname, QName};
use crate::serialization::object_to_string;


pub(crate) fn eval_on_spec(spec: &str, input: &str) -> Result<Object, String> {
    match spec {
        "XQ10+" => {
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

        let mut env = Environment::new();

        let (new_env, result) = eval_statements(program, Box::new(env));

        println!("result: {:?}", result);

        Ok(result)
    } else {
        println!("error: {:#?}", parsed);

        match parsed {
            Err(error) => {
                let code = error.as_ref();
                return Err(String::from(code))
            }
            _ => Err(format!("error {:?}", parsed))
        }
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

        let (_, check_result) = eval_statements(program, env);

        check_result

    } else {
        panic!("error {:?}", parsed);
    }
}

pub(crate) fn check_assert_eq(result: &Result<Object, String>, check: &str) {
    let expected = eval(check).unwrap();
    if !crate::eval::comparison::eq(&expected, &result.as_ref().unwrap()) {
        assert_eq!(expected, result.as_ref().unwrap().clone());
    }
}

pub(crate) fn bool_check_assert_eq(result: &Result<Object, String>, check: &str) -> bool {
    let expected = eval(check).unwrap();
    !crate::eval::comparison::eq(&expected, result.as_ref().unwrap())
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
