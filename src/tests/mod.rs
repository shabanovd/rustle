use crate::eval::{Object, Environment, Type, eval_statements, comparison, EvalResult, DynamicContext, Axis};
use crate::eval::helpers::relax;
use crate::parser::parse;
use crate::values::{resolve_element_qname, QName, QNameResolved};
use crate::serialization::object_to_string;
use crate::namespaces::NS;
use crate::parser::errors::ErrorCode;
use crate::serialization::to_xml::object_to_xml;
use crate::tree::InMemoryXMLTree;

// (sources, namespaces)
pub(crate) fn eval_on_spec(
    spec: &str,
    sources_namespaces: Option<(Vec<(&str, &str)>, Vec<(&str, &str)>)>,
    input: &str
) -> EvalResult {
    match spec {
        "XQ10" | "XP20 XQ10" | "XQ10 XP20" |
        "XQ10+" | "XP20+ XQ10+" | "XP30+ XQ10+" |
        "XQ30+" | "XP30+ XQ30+" |
        "XQ31+" | "XP31+ XQ31+" => {
            eval(sources_namespaces, input)
        }
        _ => panic!("unsupported spec {}", spec)
    }
}

pub(crate) fn eval(
    sources_namespaces: Option<(Vec<(&str, &str)>, Vec<(&str, &str)>)>,
    input: &str
) -> EvalResult {
    println!("script: {:?}", input);

    let parsed = parse(input);
    if parsed.is_ok() {
        let program = parsed.unwrap();

        // println!("{:#?}", program);

        let mut env = Environment::create();

        let mut context = DynamicContext::nothing();

        if let Some(sources_namespaces) = sources_namespaces {
            let (sources, namespaces) = sources_namespaces;
            for (name, path) in sources {
                let tree = InMemoryXMLTree::load(env.next_id(), path);
                if name == "." {
                    let writer = tree.lock().unwrap();
                    if let Some(rf) = writer.as_reader().first() {
                        context.item = Object::Node(rf);
                        context.position = Some(1)
                    }
                } else if name.starts_with("$") {
                    let writer = tree.lock().unwrap();
                    if let Some(rf) = writer.as_reader().first() {
                        let var_name = QNameResolved { url: "".to_string(), local_part: name[1..].to_string() };
                        env.set_variable(var_name, Object::Node(rf));
                    }
                } else {
                    panic!("unknown source name {}", name);
                }
            }

            for (prefix, uri) in namespaces {
                env.namespaces.add(&NS::new(prefix, uri));
            }
        }

        eval_statements(program, env, &context)

    } else {
        // println!("error: {:#?}", parsed);

        let e = match parsed {
            Err(code) => {
                let msg = String::from(code.as_ref());
                (code, msg)
            }
            _ => (ErrorCode::TODO, "err".to_string())
        };
        Err(e)
    }
}

pub(crate) fn check_assert(result: &EvalResult, check: &str) {
    let (_, check_result) = eval_assert(result, check).unwrap();
    match check_result.to_bool() {
        Ok(check_result) => {
            if !check_result {
                let (_, obj) = result.as_ref().unwrap();
                assert_eq!(format!("{:?}", obj), check);
            }
        },
        Err((code, msg)) => assert_eq!(format!("error {:?} {}", code, msg), "")
    }
}

pub(crate) fn bool_check_assert(result: &EvalResult, check: &str) -> bool {
    let (_, check_result) = eval_assert(result, check).unwrap();
    match check_result.to_bool() {
        Ok(check_result) => !check_result,
        Err((code, msg)) => {
            assert_eq!(format!("error {:?} {}", code, msg), "");
            panic!()
        }
    }
}

fn eval_assert(result: &EvalResult, check: &str) -> EvalResult {
    let (_, result) = result.as_ref().unwrap();

    let parsed = parse(check);
    if parsed.is_ok() {
        let program = parsed.unwrap();

        let mut env = Environment::create();

        let name = resolve_element_qname(&QName::local_part("result"), &env);
        env.set_variable(name, result.clone());

        eval_statements(program, env, &DynamicContext::nothing())
    } else {
        todo!()
    }
}

pub(crate) fn check_assert_eq(result: &EvalResult, check: &str) {
    let (env, obj) = result.as_ref().unwrap();

    match eval(None, check) {
        Ok((expected_env, expected_obj)) => {
            match comparison::eq((&expected_env, &expected_obj), (env, obj)) {
                Ok(v) => if !v { assert_eq!(&expected_obj, obj) },
                Err(e) => panic!("Error {:?}", e)
            }
        },
        Err(e) => panic!("Error {:?}", e)
    }
}

pub(crate) fn bool_check_assert_eq(result: &EvalResult, check: &str) -> bool {
    let (env, obj) = result.as_ref().unwrap();
    let (expected_env, expected_obj) = eval(None, check).unwrap();
    match comparison::eq((&expected_env, &expected_obj), (env, obj)) {
        Ok(v) => !v,
        Err(code) => panic!("Error {:?}", code)
    }
}

pub(crate) fn check_assert_count(result: &EvalResult, check: &str) {
    let (_, obj) = result.as_ref().unwrap();
    let actual = obj.as_ref_into_iter().count();
    let expected: usize = check.parse().unwrap();
    assert_eq!(expected, actual)
}

pub(crate) fn bool_check_assert_count(result: &EvalResult, check: &str) -> bool {
    let (_, obj) = result.as_ref().unwrap();
    let actual = obj.as_ref_into_iter().count();
    let expected: usize = check.parse().unwrap();
    expected == actual
}

pub(crate) fn check_assert_deep_eq(result: &EvalResult, check: &str) {
    let (env, obj) = result.as_ref().unwrap();
    let (expected_env, expected_obj) = eval(None, check).unwrap();
    if comparison::deep_eq((&expected_env, &expected_obj), (env, obj)).unwrap() {
        assert_eq!(&expected_obj, obj);
    }
}

pub(crate) fn bool_check_assert_deep_eq(result: &EvalResult, check: &str) -> bool {
    let (env, obj) = result.as_ref().unwrap();
    let (expected_env, expected_obj) = eval(None, check).unwrap();
    comparison::deep_eq((&expected_env, &expected_obj), (env, obj)).unwrap()
}

pub(crate) fn check_assert_permutation(result: &EvalResult, check: &str) {
    println!("result: {:?}\ncheck: {:?}", result.as_ref().unwrap().1, check);
    todo!()
}

pub(crate) fn bool_check_assert_permutation(result: &EvalResult, check: &str) -> bool {
    println!("result: {:?}\ncheck: {:?}", result.as_ref().unwrap().1, check);
    todo!()
}

pub(crate) fn check_assert_xml(result: &EvalResult, check: &str) {
    if !bool_check_assert_xml(result, check) {
        let (env, obj) = result.as_ref().unwrap();
        assert_eq!(object_to_xml(env, obj), check)
    }
}

pub(crate) fn bool_check_assert_xml(result: &EvalResult, check: &str) -> bool {
    let (env, obj) = result.as_ref().unwrap();

    // TODO: refactor

    let tmp_env = Environment::create();

    let mut items = vec![];

    let tree = InMemoryXMLTree::from_str(0, format!("<doc>{}</doc>", check).as_str());
    let (tmp_env, expected) = {
        let writer = tree.lock().unwrap();
        let reader = writer.as_reader();
        let rf = reader.first().unwrap();
        for rf in reader.forward(&rf, &None, &Axis::ForwardChild) {
            for rf in reader.forward(&rf, &None, &Axis::ForwardChild) {
                items.push(Object::Node(rf))
            }
        }
        relax(tmp_env, items).unwrap()
    };

    match &expected {
        Object::Node(l_rf) => {
            let expect = l_rf.to_xml().unwrap();
            println!("expect: {:?}", expect);
            match obj {
                Object::Node(r_rf) => {
                    let result = r_rf.to_xml().unwrap();
                    println!("result: {:?}", result);
                    // l_rf.deep_eq(r_rf)
                    expect == result
                }
                _ => panic!()
            }
        }
        _ => panic!()
    }
}

pub(crate) fn check_serialization_matches(result: &EvalResult, check: &str) {
    todo!()
}

pub(crate) fn bool_check_serialization_matches(result: &EvalResult, check: &str) -> bool {
    todo!()
}

pub(crate) fn check_assert_serialization_error(result: &EvalResult, check: &str) {
    todo!()
}

pub(crate) fn bool_check_assert_serialization_error(result: &EvalResult, check: &str) -> bool {
    todo!()
}

pub(crate) fn check_assert_type(result: &EvalResult, check: &str) {
    if let Some(err) = _check_assert_type(result, check) {
        assert_eq!(err, format!("{:?}", result.as_ref().unwrap().1));
    }
}

pub(crate) fn bool_check_assert_type(result: &EvalResult, check: &str) -> bool {
    _check_assert_type(result, check).is_some()
}

pub(crate) fn _check_assert_type(result: &EvalResult, check: &str) -> Option<String> {
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
            _ => Some(String::from("not array(xs:string)"))
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
            _ => Some(String::from("not array(xs:string)"))
        }
    } else if check == "xs:boolean" {
        match result {
            Object::Atomic(Type::Boolean(..)) => None,
            _ => Some(String::from("not xs:boolean"))
        }
    } else if check == "xs:integer" {
        match result {
            Object::Atomic(Type::Integer(..)) => None,
            _ => Some(String::from("not xs:integer"))
        }
    } else if check == "namespace-node()" {
        match result {
            Object::Node(rf) => {
                if !rf.is_namespace() {
                    Some(String::from("not namespace-node()"))
                } else {
                    None
                }
            },
            _ => Some(String::from("not namespace-node()"))
        }
    } else {
        todo!("{}", check)
    }
}

pub(crate) fn check_assert_string_value(result: &EvalResult, check: &str) {
    let (env, obj) = result.as_ref().unwrap();
    let actual = object_to_string(env, obj);
    assert_eq!(check, actual);
}

pub(crate) fn bool_check_assert_string_value(result: &EvalResult, check: &str) -> bool {
    let (env, obj) = result.as_ref().unwrap();
    let actual = object_to_string(env, obj);
    check == actual
}

pub(crate) fn check_assert_empty(result: &EvalResult) {
    let (_, obj) = result.as_ref().unwrap();
    if obj != &Object::Empty {
        assert_eq!("not empty", format!("{:?}", obj));
    }
}

pub(crate) fn bool_check_assert_empty(result: &EvalResult) -> bool {
    let (_, obj) = result.as_ref().unwrap();
    obj != &Object::Empty
}

pub(crate) fn check_assert_true(result: &EvalResult) {
    let (_, obj) = result.as_ref().unwrap();
    if obj != &Object::Atomic(Type::Boolean(true)) {
        assert_eq!("not true", format!("{:?}", obj));
    }
}

pub(crate) fn bool_check_assert_true(result: &EvalResult) -> bool {
    let (_, obj) = result.as_ref().unwrap();
    obj != &Object::Atomic(Type::Boolean(true))
}

pub(crate) fn check_assert_false(result: &EvalResult) {
    let (_, obj) = result.as_ref().unwrap();
    if obj != &Object::Atomic(Type::Boolean(false)) {
        assert_eq!("not false", format!("{:?}", obj));
    }
}

pub(crate) fn bool_check_assert_false(result: &EvalResult) -> bool {
    let (_, obj) = result.as_ref().unwrap();
    obj != &Object::Atomic(Type::Boolean(false))
}

pub(crate) fn check_error(result: &EvalResult, expected_code: &str) {
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

pub(crate) fn bool_check_error(result: &EvalResult, expected_code: &str) -> bool {
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
