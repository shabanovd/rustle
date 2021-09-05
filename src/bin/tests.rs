use std::fs;
use xmlparser::{Token, ElementEnd};
use std::collections::{HashSet, HashMap};
use std::borrow::Cow;
use nom::AsBytes;

enum AssertType {
    AllOf,
    AnyOf,
}

fn fn_name(name: &str) -> String {
    name.to_lowercase()
        .replace("%", "_pc_")
        .replace("+", "_plus_")
        .replace(".", "_dot_")
        .replace("-", "_dash_")
        .replace(":", "_dots_")
        .replace("__", "_")
}

fn string_cleanup(data: &[u8]) -> String {
    let data = quick_xml::escape::unescape(data).unwrap();
    let data = std::str::from_utf8(data.as_bytes()).unwrap();
    let data = String::from(data);

    data.replace("\r", "\n")
}

fn cleanup(data: String) -> String {
    data.replace("\\", "\\\\")
        .replace("\"", "\\\"")
}

fn generate_tests(name: &str, file: &str) -> String {
    let data = fs::read_to_string(format!("./qt3tests/{}", file)).unwrap();
    let dir = folder(String::from(file.clone()));

    let mut generated = String::from("#[cfg(test)]
mod tests {
    use crate::tests::*;
    use std::fs;\n\n");

    let mut test_case_node = false;
    let mut test_name: Option<&str> = None;

    let mut dependency_node = false;

    let mut dependency_spec_flag = false;
    let mut dependency_spec: Option<String> = None;

    let mut dependency_feature_flag = false;
    let mut dependency_feature: Option<String> = None;

    let mut dependency_xml_version_flag = false;
    let mut dependency_xsd_version_flag = false;
    let mut dependency_language_flag = false;

    let mut dependency_satisfied = true;


    let mut script_flag = false;

    let mut result_flag = false;
    let mut any_of_flag = false;
    let mut assert_flag = false;
    let mut assert_empty_flag = false;
    let mut assert_true_flag = false;
    let mut assert_false_flag = false;

    let mut error_flag = false;

    let mut error_code: Option<&str> = None;

    let mut script = String::new();
    let mut script_file: Option<&str> = None;
    let mut assert = String::new();

    let build_prefix = |any_of_flag| -> &str {
        if any_of_flag {
            "            || bool_"
        } else {
            "        "
        }
    };

    let check_result = |fn_name, any_of_flag| -> String {
        let mut generated = String::new();
        generated.push_str(build_prefix(any_of_flag));
        generated.push_str(fn_name);
        generated.push_str("(&result)");
        if !any_of_flag { generated.push_str(";"); }
        generated.push_str("\n");
        generated
    };

    let check_result_value = |fn_name, assert, any_of_flag| -> String {
        let mut generated = String::new();
        generated.push_str(build_prefix(any_of_flag));
        generated.push_str(fn_name);
        generated.push_str("(&result, \"");
        generated.push_str(cleanup(assert).as_str());
        generated.push_str("\")");
        if !any_of_flag { generated.push_str(";"); }
        generated.push_str("\n");
        generated
    };

    for token in xmlparser::Tokenizer::from(data.as_str()) {
        match token.unwrap() {
            Token::ElementStart { prefix, local, span } => {
                test_case_node = false;
                dependency_node = false;

                match local.as_str() {
                    "test-case" => {
                        test_case_node = true;
                        test_name = None;

                        dependency_spec = None;
                        dependency_feature = None;
                        dependency_satisfied = true;

                        dependency_spec_flag = false;
                        dependency_feature_flag = false;
                        dependency_xml_version_flag = false;
                        dependency_xsd_version_flag = false;
                        dependency_language_flag = false;
                    },
                    "dependency" => {
                        dependency_node = true;
                    }
                    "test" => {
                        script_flag = true;
                        script.clear();
                    },
                    "result" => {
                        result_flag = true;
                    },
                    "any-of" => {
                        any_of_flag = true;
                        generated.push_str("        assert!(true\n");
                    },
                    "error" => {
                        if result_flag {
                            error_flag = true;
                        }
                    }
                    "assert-empty" => {
                        assert_empty_flag = true;
                    }
                    "assert-true" => {
                        assert_true_flag = true;
                    }
                    "assert-false"  => {
                        assert_false_flag = true;
                    }
                    "assert" |
                    "assert-eq" |
                    "assert-count" |
                    "assert-deep-eq" |
                    "assert-permutation" |
                    "assert-xml" |
                    "assert-type" |
                    "assert-string-value" => {
                        assert_flag = true;
                        assert.clear();
                    },
                    _ => {}
                }
            },
            Token::Attribute { prefix: _, local, value, span: _ } => {
                // println!("Attribute {:?} {:?} {}", local.as_str(), value.as_str(), test_case_node);
                if dependency_node {
                    if local.as_str() == "value" {
                        if dependency_spec_flag {
                            dependency_spec_flag = false;
                            dependency_spec = Some(value.as_str().to_string());
                        } else if dependency_feature_flag {
                            dependency_feature_flag = false;
                            dependency_feature = Some(value.as_str().to_string());
                        } else if dependency_xml_version_flag {
                        } else if dependency_xsd_version_flag {
                        } else if dependency_language_flag {
                        } else {
                            println!("Attribute {:?} = {:?}", local.as_str(), value.as_str());
                        }
                    } else if local.as_str() == "type" {
                        match value.as_str() {
                            "spec" => dependency_spec_flag = true,
                            "feature"  => dependency_feature_flag = true,
                            "xml-version" => dependency_xml_version_flag = true,
                            "xsd-version"  => dependency_xsd_version_flag = true,
                            "language" => dependency_language_flag = true,
                            _ => {
                                println!("Attribute {:?} = {:?}", local.as_str(), value.as_str());
                            }
                        }
                    } else if local.as_str() == "satisfied" {
                        if value.as_str() == "false" {
                            dependency_satisfied = false;
                        } else {
                            dependency_satisfied = true;
                        }
                    } else {
                        println!("Attribute {:?} = {:?}", local.as_str(), value.as_str());
                    }
                }
                if test_case_node && local.as_str() == "name" {
                    test_name = Some(value.as_str())
                }
                if script_flag && local.as_str() == "file" {
                    script_file = Some(value.as_str());
                }
                if error_flag && local.as_str() == "code" {
                    error_code = Some(value.as_str())
                }
            },
            Token::ElementEnd { end, span } => {
                match end {
                    ElementEnd::Open => {
                        test_case_node = false;
                        dependency_node = false;
                    },
                    ElementEnd::Close(prefix, local) => {
                        match local.as_str() {
                            "test-case" => {
                                test_case_node = false;

                                generated.push_str("    }\n\n");
                            },
                            "test" => {
                                script_flag = false;

                                script = script.replace("\\", "\\\\")
                                    .replace("\"", "\\\"");

                                generated.push_str("    #[test]\n    fn ");
                                generated.push_str(fn_name(test_name.unwrap()).as_str());
                                generated.push_str("() {\n");
                                if let Some(spec) = dependency_spec.as_ref() {
                                    generated.push_str("        let result = eval_on_spec(\"");
                                    generated.push_str(spec.as_str());
                                    generated.push_str("\",\"");
                                } else {
                                    generated.push_str("        let result = eval(\"");
                                }
                                generated.push_str(script.as_str());
                                generated.push_str("\");\n\n");
                                // generated.push_str("        println!(\"{:?}\", result);\n");

                                test_name = None;
                                script.clear();
                            },
                            "result" => {
                                result_flag = false;
                            },
                            "any-of" => {
                                any_of_flag = false;
                                generated.push_str("        );\n");
                            },
                            "assert" => {
                                assert_flag = false;

                                generated.push_str(check_result_value(
                                    "check_assert",
                                    assert.clone(), any_of_flag
                                ).as_str());

                                assert.clear();
                            }
                            "assert-eq" => {
                                assert_flag = false;

                                generated.push_str(check_result_value(
                                    "check_assert_eq",
                                    assert.clone(), any_of_flag
                                ).as_str());

                                assert.clear();
                            }
                            "assert-count" => {
                                assert_flag = false;

                                generated.push_str(check_result_value(
                                    "check_assert_count",
                                    assert.clone(), any_of_flag
                                ).as_str());

                                assert.clear();
                            }
                            "assert-deep-eq" => {
                                assert_flag = false;

                                generated.push_str(check_result_value(
                                    "check_assert_deep_eq",
                                    assert.clone(), any_of_flag
                                ).as_str());

                                assert.clear();
                            }
                            "assert-permutation" => {
                                assert_flag = false;

                                generated.push_str(check_result_value(
                                    "check_assert_permutation",
                                    assert.clone(), any_of_flag
                                ).as_str());

                                assert.clear();
                            }
                            "assert-xml" => {
                                assert_flag = false;

                                generated.push_str(check_result_value(
                                    "check_assert_xml",
                                    assert.clone(), any_of_flag
                                ).as_str());

                                assert.clear();
                            }
                            "assert-type" => {
                                assert_flag = false;

                                generated.push_str(check_result_value(
                                    "check_assert_type",
                                    assert.clone(), any_of_flag
                                ).as_str());

                                assert.clear();
                            }
                            "assert-string-value" => {
                                assert_flag = false;

                                generated.push_str(check_result_value(
                                    "check_assert_string_value",
                                    assert.clone(), any_of_flag
                                ).as_str());

                                assert.clear();
                            },
                            _ => {}
                        }
                    },
                    ElementEnd::Empty => {
                        if script_flag {
                            script_flag = false;

                            generated.push_str("    #[test]\n    fn ");
                            generated.push_str(fn_name(test_name.unwrap()).as_str());

                            generated.push_str("() {\n");

                            if test_name.unwrap() == "K-Literals-29" {
                                generated.push_str("        let script = String::new();\n");
                            } else {
                                generated.push_str("        let script = fs::read_to_string(\"./qt3tests/");
                                generated.push_str(dir.as_str());
                                generated.push_str(script_file.unwrap());
                                generated.push_str("\").unwrap();\n");
                            }

                            generated.push_str("        let result = eval(script.as_str());\n");
                            // generated.push_str("        println!(\"{:?}\", result);\n");

                            test_name = None;
                            script.clear();
                        } else if assert_empty_flag {
                            assert_empty_flag = false;

                            let code = check_result("check_assert_empty", any_of_flag);
                            generated.push_str(code.as_str());

                        } else if assert_true_flag {
                            assert_true_flag = false;

                            let code = check_result("check_assert_true", any_of_flag);
                            generated.push_str(code.as_str());

                        } else if assert_false_flag {
                            assert_false_flag = false;

                            let code = check_result("check_assert_false", any_of_flag);
                            generated.push_str(code.as_str());

                        } else if error_flag {
                            error_flag = false;

                            generated.push_str(build_prefix(any_of_flag));
                            generated.push_str("check_error(&result, \"");
                            generated.push_str(error_code.unwrap());
                            generated.push_str("\")");
                            if !any_of_flag { generated.push_str(";"); }
                            generated.push_str("\n");

                            error_code = None;
                        }
                    },
                }
            }
            Token::Declaration { version: _, encoding: _, standalone: _, span: _ } => {}
            Token::ProcessingInstruction { target: _, content: _, span: _ } => {}
            Token::Comment { text, span: _ } => {}
            Token::DtdStart { name: _, external_id: _, span: _ } => {}
            Token::EmptyDtd { name: _, external_id: _, span: _ } => {}
            Token::EntityDeclaration { name: _, definition: _, span: _ } => {}
            Token::DtdEnd { span: _ } => {}
            Token::Text { text } => {
                if script_flag {
                    let data = string_cleanup(text.as_bytes());
                    script.push_str(data.as_str());
                } else if assert_flag {
                    let data = string_cleanup(text.as_bytes());
                    assert.push_str(data.as_str());
                }
            }
            Token::Cdata { text, span: _ } => {
                if script_flag {
                    script.push_str(text.as_str());
                } else if assert_flag {
                    assert.push_str(text.as_str());
                }
            }
        }
    }

    generated.push_str("}\n");

    println!("{}", file);
    let file = file.to_lowercase()
        .replace("fn/","fun/")
        .replace("false.xml","bool_false.xml")
        .replace("true.xml","bool_true.xml")
        .replace("do.xml","loop_do.xml")
        .replace(".","_")
        .replace("-","_");

    // remove .xml
    let mut file = file.as_str()[0..file.len() - 4].to_string();

    file.push_str(".rs");

    let dir = folder(file.clone());

    fs::create_dir_all(format!("src/xqts/{}", dir))
        .expect("Unable to create folders");

    if !file.contains("fs::") {
        file.replace("    use std::fs;\n", "");
    }

    fs::write(format!("src/xqts/{}", file), generated)
        .expect("Unable to write file");

    file
}

pub fn generate() {
    let data = fs::read_to_string("./qt3tests/catalog.xml").unwrap();

    let mut files = vec![];

    let mut inside_test_set = false;
    let mut tests_name = "";
    let mut tests_file = "";

    for token in xmlparser::Tokenizer::from(data.as_str()) {
        match token.unwrap() {
            Token::ElementStart { prefix, local, span } => {
                match local.as_str() {
                    "test-set" => {
                        inside_test_set = true;
                        tests_name = "";
                        tests_file = "";
                    },
                    _ => {},
                }
            },
            Token::Attribute { prefix: _, local, value, span: _ } => {
                if inside_test_set {
                    match local.as_str() {
                        "name" => tests_name = value.as_str(),
                        "file" => tests_file = value.as_str(),
                        _ => {},
                    }
                }
            },
            Token::ElementEnd { end, span } => {
                match end {
                    ElementEnd::Open => {},
                    ElementEnd::Close(prefix, local) => {
                        match local.as_str() {
                            "" => {
                            },
                            _ => {},
                        }

                    },
                    ElementEnd::Empty => {
                        if inside_test_set {
                            let file = generate_tests(tests_name, tests_file);

                            files.push(file);

                            tests_name = "";
                            tests_file = "";
                            inside_test_set = false;
                        }
                    },
                }
            },
            _ => {}
        }
    }

    files.sort();

    // generate mods
    let mut top = String::new();
    let mut processed = HashSet::new();

    let mut mods = HashMap::new();
    for file in files {
        let dir = folder(file.clone());

        let data = mods.entry(dir.clone())
            .or_insert_with(|| String::new());

        let mut data = data.clone();
        data.push_str("mod ");
        data.push_str(file.replace(dir.as_str(), "").replace(".rs", "").as_str());
        data.push_str(";\n");

        mods.insert(dir.clone(), data);

        let parent = folder(dir.clone());
        if processed.insert(parent.clone()) {
            top.push_str("mod ");
            top.push_str(parent.replace("/", "").as_str());
            top.push_str(";\n");
        }
    }
    top.push_str("\n");

    for (path, data) in mods {
        fs::write(format!("src/xqts/{}/mod.rs", path), data)
            .expect("Unable to write file");
    }

    fs::write("src/xqts/mod.rs", top)
        .expect("Unable to write file");
}

fn folder(path: String) -> String {
    let mut names = vec![];

    let parts = path.split("/");
    for part in parts {
        names.push(part)
    }

    names.remove(names.len() - 1);

    let mut path = names.join("/");
    path.push_str("/");
    path
}

fn main() {
    generate()
}