use crate::eval::{Object, Environment, Type, eval_statements, object_to_iterator, comparison, EvalResult};
use crate::parser::parse;
use crate::values::{resolve_element_qname, QName};
use crate::serialization::object_to_string;
use crate::fns::object_to_bool;
use crate::parser::errors::ErrorCode;
use crate::serialization::to_xml::object_to_xml;


pub(crate) fn eval_on_spec<'a>(spec: &str, input: &str) -> EvalResult<'a> {
    match spec {
        "XQ10" | "XQ10+" | "XP30+ XQ10+" | "XQ30+" | "XP30+ XQ30+" | "XQ31+" | "XP31+ XQ31+" => {
            eval(input)
        }
        _ => panic!("unsupported spec {}", spec)
    }
}

pub(crate) fn eval<'a>(input: &str) -> EvalResult<'a> {
    println!("script: {:?}", input);

    let parsed = parse(input);
    if parsed.is_ok() {
        let program = parsed.unwrap();

        // println!("{:#?}", program);

        let env = Environment::create();
        eval_statements(program, env)

    } else {
        // println!("error: {:#?}", parsed);

        let msg = match parsed {
            Err(error) => {
                let code = error.as_ref();
                String::from(code)
            }
            _ => "err".to_string() // format!("error {:?}", parsed)
        };
        Err((ErrorCode::TODO, msg))
    }
}

pub(crate) fn check_assert<'a>(result: &'a EvalResult<'a>, check: &str) {
    let (check_env, check_result) = eval_assert(result, check).unwrap();
    match object_to_bool(&check_result) {
        Ok(check_result) => {
            if !check_result {
                let (env, obj) = result.as_ref().unwrap();
                assert_eq!(format!("{:?}", obj), check);
            }
        },
        Err((code, msg)) => assert_eq!(format!("error {:?} {}", code, msg), "")
    }
}

pub(crate) fn bool_check_assert<'a>(result: &'a EvalResult<'a>, check: &str) -> bool {
    let (check_env, check_result) = eval_assert(result, check).unwrap();
    match object_to_bool(&check_result) {
        Ok(check_result) => !check_result,
        Err((code, msg)) => {
            assert_eq!(format!("error {:?} {}", code, msg), "");
            panic!()
        }
    }
}

fn eval_assert<'a, 'b>(result: &'a EvalResult<'a>, check: &str) -> EvalResult<'b> {
    let (_, result) = result.as_ref().unwrap();

    let parsed = parse(check);
    if parsed.is_ok() {
        let program = parsed.unwrap();

        let mut env = Environment::create();

        let name = resolve_element_qname(&QName::local_part("result"), &env);
        env.set(name, result.clone());

        eval_statements(program, env)
    } else {
        todo!()
    }
}

pub(crate) fn check_assert_eq<'a>(result: &'a EvalResult<'a>, check: &str) {
    let (env, obj) = result.as_ref().unwrap();

    match eval(check) {
        Ok((expected_env, expected_obj)) => {
            match comparison::eq((&expected_env, &expected_obj), (env, obj)) {
                Ok(v) => if !v { assert_eq!(&expected_obj, obj) },
                Err(e) => panic!("Error {:?}", e)
            }
        },
        Err(e) => panic!("Error {:?}", e)
    }
}

pub(crate) fn bool_check_assert_eq<'a>(result: &'a EvalResult<'a>, check: &str) -> bool {
    let (env, obj) = result.as_ref().unwrap();
    let (expected_env, expected_obj) = eval(check).unwrap();
    match comparison::eq((&expected_env, &expected_obj), (env, obj)) {
        Ok(v) => !v,
        Err(code) => panic!("Error {:?}", code)
    }
}

pub(crate) fn check_assert_count<'a>(result: &'a EvalResult<'a>, check: &str) {
    let (env, obj) = result.as_ref().unwrap();
    let actual = object_to_iterator(obj).len();
    let expected: usize = check.parse().unwrap();
    assert_eq!(expected, actual)
}

pub(crate) fn bool_check_assert_count<'a>(result: &'a EvalResult<'a>, check: &str) -> bool {
    let (env, obj) = result.as_ref().unwrap();
    let actual = object_to_iterator(obj).len();
    let expected: usize = check.parse().unwrap();
    expected == actual
}

pub(crate) fn check_assert_deep_eq<'a>(result: &'a EvalResult<'a>, check: &str) {
    let (env, obj) = result.as_ref().unwrap();
    let (expected_env, expected_obj) = eval(check).unwrap();
    if comparison::deep_eq((&expected_env, &expected_obj), (env, obj)).unwrap() {
        assert_eq!(&expected_obj, obj);
    }
}

pub(crate) fn bool_check_assert_deep_eq<'a>(result: &'a EvalResult<'a>, check: &str) -> bool {
    let (env, obj) = result.as_ref().unwrap();
    let (expected_env, expected_obj) = eval(check).unwrap();
    comparison::deep_eq((&expected_env, &expected_obj), (env, obj)).unwrap()
}

pub(crate) fn check_assert_permutation<'a>(result: &'a EvalResult<'a>, check: &str) {
    println!("result: {:?}\ncheck: {:?}", result.as_ref().unwrap().1, check);
    todo!()
}

pub(crate) fn bool_check_assert_permutation<'a>(result: &'a EvalResult<'a>, check: &str) -> bool {
    println!("result: {:?}\ncheck: {:?}", result.as_ref().unwrap().1, check);
    todo!()
}

pub(crate) fn check_assert_xml<'a>(result: &'a EvalResult<'a>, check: &str) {
    let (env, obj) = result.as_ref().unwrap();
    let (expected_env, expected_obj) = eval(check).unwrap();
    match comparison::deep_eq((&expected_env, &expected_obj), (env, obj)) {
        Ok(v) => {
            if !v {
                assert_eq!(object_to_xml(env, obj), check)
            }
        },
        Err(e) => assert_eq!(format!("error {:?}", e), check),
    }
}

pub(crate) fn bool_check_assert_xml<'a>(result: &'a EvalResult<'a>, check: &str) -> bool {
    let (expected_env, expected_obj) = eval(check).unwrap();
    let (env, obj) = result.as_ref().unwrap();
    comparison::deep_eq((&expected_env, &expected_obj), (env, obj)).unwrap()
}

pub(crate) fn check_assert_type<'a>(result: &'a EvalResult<'a>, check: &str) {
    if let Some(err) = _check_assert_type(result, check) {
        assert_eq!(err, format!("{:?}", result.as_ref().unwrap().1));
    }
}

pub(crate) fn bool_check_assert_type<'a>(result: &'a EvalResult<'a>, check: &str) -> bool {
    _check_assert_type(result, check).is_some()
}

pub(crate) fn _check_assert_type<'a>(result: &'a EvalResult<'a>, check: &str) -> Option<String> {
    let (env, result) = result.as_ref().unwrap();
    if check == "array(*)" {
        match result {
            Object::Array(..) => None,
            _ => Some(String::from("not array(*)"))
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
                            _ => return Some(format!("not xs:string: {:?}", item))
                        }
                    }
                    None
                }
            },
            _ => {
                Some(String::from("not array(xs:string)"))
            }
        }
    } else if check == "array(xs:string*)" {
        match result {
            Object::Array(items) => {
                if items.len() == 0 {
                    Some(format!("not {}", check))
                } else {
                    for item in items {
                        match item {
                            Object::Atomic(Type::String(..)) => {},
                            Object::Sequence(items) => {
                                for item in items {
                                    match item {
                                        Object::Atomic(Type::String(..)) => {},
                                        _ => return Some(format!("not xs:string: {:?}", item))
                                    }
                                }
                            },
                            _ => {
                                return Some(format!("not xs:string*: {:?}", item));
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
        todo!("{}", check)
    }
}

pub(crate) fn check_assert_string_value<'a>(result: &'a EvalResult<'a>, check: &str) {
    let (env, obj) = result.as_ref().unwrap();
    let actual = object_to_string(env, obj);
    assert_eq!(check, actual);
}

pub(crate) fn bool_check_assert_string_value<'a>(result: &'a EvalResult<'a>, check: &str) -> bool {
    let (env, obj) = result.as_ref().unwrap();
    let actual = object_to_string(env, obj);
    check == actual
}

pub(crate) fn check_assert_empty<'a>(result: &'a EvalResult<'a>) {
    let (env, obj) = result.as_ref().unwrap();
    if obj != &Object::Empty {
        assert_eq!("not empty", format!("{:?}", obj));
    }
}

pub(crate) fn bool_check_assert_empty<'a>(result: &'a EvalResult<'a>) -> bool {
    let (env, obj) = result.as_ref().unwrap();
    obj != &Object::Empty
}

pub(crate) fn check_assert_true<'a>(result: &'a EvalResult<'a>) {
    let (env, obj) = result.as_ref().unwrap();
    if obj != &Object::Atomic(Type::Boolean(true)) {
        assert_eq!("not true", format!("{:?}", obj));
    }
}

pub(crate) fn bool_check_assert_true<'a>(result: &'a EvalResult<'a>) -> bool {
    let (env, obj) = result.as_ref().unwrap();
    obj != &Object::Atomic(Type::Boolean(true))
}

pub(crate) fn check_assert_false<'a>(result: &'a EvalResult<'a>) {
    let (env, obj) = result.as_ref().unwrap();
    if obj != &Object::Atomic(Type::Boolean(false)) {
        assert_eq!("not false", format!("{:?}", obj));
    }
}

pub(crate) fn bool_check_assert_false<'a>(result: &'a EvalResult<'a>) -> bool {
    let (env, obj) = result.as_ref().unwrap();
    obj != &Object::Atomic(Type::Boolean(false))
}

pub(crate) fn check_error<'a>(result: &'a EvalResult<'a>, expected_code: &str) {
    match result {
        Err((code, msg)) => {
            if expected_code != "*" {
                let code = String::from(code.as_ref());
                assert_eq!(code, expected_code)
            }
        },
        _ => assert_eq!("not error", format!("{:?}", result.as_ref().unwrap().1))
    }
}

pub(crate) fn bool_check_error<'a>(result: &'a EvalResult<'a>, expected_code: &str) -> bool {
    match result {
        Err((code, msg)) => {
            if expected_code != "*" {
                let code = String::from(code.as_ref());
                code == expected_code
            } else {
                true
            }
        },
        _ => false
    }
}
