use std::fs;
use xmlparser::{Token, ElementEnd};
use std::collections::{HashSet, HashMap};
use std::borrow::Cow;
use nom::AsBytes;

enum AssertType {
    AllOf,
    AnyOf,
}

fn fn_name(name: String) -> String {
    name.to_lowercase()
        .replace("%", "_pc_")
        .replace("+", "_plus_")
        .replace(".", "_dot_")
        .replace("-", "_dash_")
        .replace(":", "_dots_")
        .replace("__", "_")
}

fn build_prefix(any_of_flag: bool) -> String {
    if any_of_flag {
        "            || bool_".to_string()
    } else {
        "        ".to_string()
    }
}

fn check_result(fn_name: String, any_of_flag: bool) -> String {
    let mut buf = String::new();
    buf.push_str(build_prefix(any_of_flag).as_str());
    buf.push_str(fn_name.as_str());
    buf.push_str("(&result)");
    if !any_of_flag { buf.push_str(";"); }
    buf.push_str("\n");
    buf
}

fn check_result_value(fn_name: String, assert: String, any_of_flag: bool) -> String {
    let mut buf = String::new();
    buf.push_str(build_prefix(any_of_flag).as_str());
    buf.push_str(fn_name.as_str());
    buf.push_str("(&result, \"");
    buf.push_str(cleanup(assert).as_str());
    buf.push_str("\")");
    if !any_of_flag { buf.push_str(";"); }
    buf.push_str("\n");
    buf
}

trait State {
    fn event(&mut self, token: Token, generated: &mut String) -> bool;
}

#[derive(Clone)]
struct Environment {
    path: String,
    reference: Option<String>,
    name: Option<String>,
    namespaces: Vec<Namespace>,
    sources: Vec<Source>,
}

#[derive(Clone)]
struct Namespace {
    prefix: String,
    uri: String,
}

#[derive(Clone)]
struct Source {
    role: String,
    file: String,
}

struct EnvironmentState {
    reference: Option<String>,
    name: Option<String>,

    namespace: Option<NamespaceState>,
    source: Option<SourceState>,

    namespaces: Vec<Namespace>,
    sources: Vec<Source>,
}

impl EnvironmentState {
    fn empty() -> Self {
        EnvironmentState {
            reference: None, name: None, namespace: None, source: None,
            namespaces: vec![], sources: vec![]
        }
    }

    fn to_data(&self, path: String) -> Option<Environment> {
        Some(Environment {
            path,
            reference: self.reference.clone(),
            name: self.name.clone(),
            namespaces: self.namespaces.clone(),
            sources: self.sources.clone()
        })
    }
}

impl State for EnvironmentState {
    fn event(&mut self, token: Token, generated: &mut String) -> bool {
        if let Some(namespace) = &mut self.namespace {
            if namespace.event(token, generated) {
                if let Some(ns) = namespace.to_data() {
                    self.namespaces.push(ns);
                }
                self.namespace = None;
            } else {
                return false;
            }
        }

        if let Some(source) = &mut self.source {
            if source.event(token, generated) {
                if let Some(sr) = source.to_data() {
                    self.sources.push(sr);
                }
                self.source = None;
            } else {
                return false;
            }
        }

        match token {
            Token::ElementStart { local, .. } => {
                match local.as_str() {
                    "namespace" => self.namespace = Some(NamespaceState::empty()),
                    "source" => self.source = Some(SourceState::empty()),
                    _ => {}
                }
                false
            }
            Token::Attribute { local, value, .. } => {
                match local.as_str() {
                    "ref" => self.reference = Some(String::from(value.as_str())),
                    "name" => self.name = Some(String::from(value.as_str())),
                    _ => {}
                }
                false
            }
            Token::ElementEnd { end, .. } => {
                match end {
                    ElementEnd::Open => {}
                    ElementEnd::Close(prefix, local) => {
                        if local.as_str() == "environment" {
                            return true
                        }
                    }
                    ElementEnd::Empty => {
                        if self.reference.is_some() {
                            return true
                        }
                    }
                }
                false
            },
            _ => false,
        }
    }
}

struct NamespaceState {
    prefix: Option<String>,
    uri: Option<String>,
}

impl NamespaceState {
    fn empty() -> Self {
        NamespaceState {
            prefix: None,
            uri: None,
        }
    }

    fn to_data(&self) -> Option<Namespace> {
        if let Some(prefix) = &self.prefix {
            if let Some(uri) = &self.uri {
                return Some(Namespace { prefix: prefix.clone(), uri: uri.clone() });
            }
        }
        None
    }
}

impl State for NamespaceState {
    fn event(&mut self, token: Token, generated: &mut String) -> bool {
        match token {
            Token::Attribute { local, value, .. } => {
                match local.as_str() {
                    "prefix" => self.prefix = Some(value.as_str().to_string()),
                    "uri" => self.uri = Some(value.as_str().to_string()),
                    _ => {}
                }
                false
            },
            Token::ElementEnd { .. } => true,
            _ => false
        }
    }
}

struct SourceState {
    role: Option<String>,
    file: Option<String>,
}

impl SourceState {
    fn empty() -> Self {
        SourceState {
            role: None,
            file: None,
        }
    }

    fn to_data(&self) -> Option<Source> {
        if let Some(role) = &self.role {
            if let Some(file) = &self.file {
                return Some(Source { role: role.clone(), file: file.clone() });
            }
        }
        None
    }
}

impl State for SourceState {
    fn event(&mut self, token: Token, generated: &mut String) -> bool {
        match token {
            Token::Attribute { local, value, .. } => {
                match local.as_str() {
                    "role" => self.role = Some(value.as_str().to_string()),
                    "file" => self.file = Some(value.as_str().to_string()),
                    _ => {}
                }
                false
            },
            Token::ElementEnd { .. } => true,
            _ => panic!()
        }
    }
}

struct DependencyState {
    // "spec",
    // "feature",
    // "xml-version",
    // "xsd-version",
    // "language"
    // "satisfied" (true|false)
    t: Option<String>,
    v: Option<String>,
}

impl DependencyState {
    fn empty() -> Self {
        DependencyState { t: None, v: None }
    }
}

impl State for DependencyState {
    fn event(&mut self, token: Token, generated: &mut String) -> bool {
        match token {
            Token::Attribute { local, value, .. } => {
                match local.as_str() {
                    "type" => self.t = Some(String::from(value.as_str())),
                    "value" => self.v = Some(String::from(value.as_str())),
                    _ => {}
                }
                false
            }
            Token::ElementEnd { .. } => true,
            _ => panic!()
        }
    }
}

struct TestCaseState {
    dir: String,
    envs: HashMap<String, Environment>,

    env: Option<Environment>,
    spec: Option<String>,

    name: Option<String>,
    dependency: Option<DependencyState>,
    environment: Option<EnvironmentState>,
    test: Option<TestState>,
    result: Option<ResultState>,
}

impl TestCaseState {
    fn empty(dir: String, envs: HashMap<String, Environment>) -> TestCaseState {
        TestCaseState {
            dir, envs,
            env: None, spec: None,
            name: None,
            dependency: None, environment: None, test: None, result: None
        }
    }
}

impl State for TestCaseState {
    fn event(&mut self, token: Token, generated: &mut String) -> bool {
        if let Some(state) = &mut self.dependency {
            if state.event(token, generated) {
                if let Some(t) = &state.t {
                    if let Some(v) = &state.v {
                        match t.as_str() {
                            "spec" => self.spec = Some(v.clone()),
                            _ => {}
                        }

                    }
                }
                self.dependency = None;
            }
            return false;
        }

        if let Some(state) = &mut self.test {
            if state.event(token, generated) {
                self.test = None;
            }
            return false;
        }

        if let Some(state) = &mut self.environment {
            if state.event(token, generated) {
                if let Some(env) = state.to_data("".to_string()) {
                    if let Some(name) = env.reference {
                        if let Some(env) = self.envs.get(&name) {
                            self.env = Some(env.clone());
                        }
                    }
                }
                self.environment = None;
            }
            return false;
        }

        if let Some(state) = &mut self.result {
            if state.event(token, generated) {
                self.result = None;
            }
            return false;
        }

        match token {
            Token::ElementStart { local, .. } => {
                match local.as_str() {
                    "environment" => self.environment = Some(EnvironmentState::empty()),
                    "dependency" => self.dependency = Some(DependencyState::empty()),
                    "test" => self.test = Some(TestState::empty(
                        self.name.as_ref().unwrap().clone(), self.spec.clone(), self.env.clone(), self.dir.clone()
                    )),
                    "result" => self.result = Some(ResultState::empty()),
                    _ => {}
                }
            }
            Token::Attribute { local, value, .. } => {
                if local.as_str() == "name" {
                    self.name = Some(String::from(value.as_str()));
                }
            }
            Token::ElementEnd { end, .. } => {
                match end {
                    ElementEnd::Open => {}
                    ElementEnd::Close(prefix, local) => {
                        if local.as_str() == "test-case" {
                            generated.push_str("    }\n\n");
                            return true;
                        }
                    }
                    ElementEnd::Empty => {}
                }
            }
            _ => {}
        }
        false
    }
}

struct TestState {
    test_name: String,
    spec: Option<String>,
    env: Option<Environment>,
    dir: String,

    file: Option<String>,
    script: String,
}

impl TestState {
    fn empty(test_name: String, spec: Option<String>, env: Option<Environment>, dir: String) -> TestState {
        TestState {
            test_name, spec, env, dir,
            file: None, script: String::new()
        }
    }

    fn generate(&self, generated: &mut String) {
        if let Some(file) = &self.file {
            self.generate_file(generated, file.clone());
        } else {
            self.generate_script(generated, self.script.clone());
        }
    }

    fn generate_head(&self, generated: &mut String) {
        generated.push_str("    #[test]\n    fn ");
        generated.push_str(fn_name(self.test_name.clone()).as_str());

        generated.push_str("() {\n");
        if let Some(env) = &self.env {
            if env.sources.len() != 0 {

                generated.push_str("        let mut sources: Vec<(&str,&str)> = vec![];\n");
                for source in &env.sources {
                    generated.push_str("        sources.push((\"");
                    generated.push_str(source.role.as_str());
                    generated.push_str("\",\"");
                    generated.push_str(env.path.as_str());
                    generated.push_str(source.file.as_str());
                    generated.push_str("\"));\n");
                }
            } else {
                generated.push_str("        let sources: Vec<(&str,&str)> = vec![];\n");
            }
        } else {
            generated.push_str("        let sources: Vec<(&str,&str)> = vec![];\n");
        }
    }

    fn generate_file(&self, generated: &mut String, file: String) {
        self.generate_head(generated);

        if self.test_name == "K-Literals-29" {
            generated.push_str("        let script = String::new();\n");
        } else {
            generated.push_str("        let script = fs::read_to_string(\"./qt3tests/");
            generated.push_str(self.dir.as_str());
            generated.push_str(file.as_str());
            generated.push_str("\").unwrap();\n");
        }

        generated.push_str("        let result = eval(sources, script.as_str());\n");
        // generated.push_str("        println!(\"{:?}\", result);\n");
    }

    fn generate_script(&self, generated: &mut String, script: String) {
        let script = script.replace("\\", "\\\\")
            .replace("\"", "\\\"");

        self.generate_head(generated);

        if let Some(spec) = &self.spec {
            generated.push_str("        let result = eval_on_spec(\"");
            generated.push_str(spec.as_str());
            generated.push_str("\",sources,\"");
        } else {
            generated.push_str("        let result = eval(sources,\"");
        }
        generated.push_str(script.as_str());
        generated.push_str("\");\n\n");
        // generated.push_str("        println!(\"{:?}\", result);\n");
    }
}

impl State for TestState {
    fn event(&mut self, token: Token, generated: &mut String) -> bool {
        match token {
            Token::Attribute { local, value, .. } => {
                if local.as_str() == "file" {
                    self.file = Some(String::from(value.as_str()));
                }
                false
            }
            Token::ElementEnd { end, .. } => {
                match end {
                    ElementEnd::Open => false,
                    ElementEnd::Close(prefix, local) => {
                        if local.as_str() == "test" {
                            self.generate(generated);
                            true
                        } else {
                            false
                        }
                    }
                    ElementEnd::Empty => {
                        self.generate(generated);
                        true
                    }
                }
            },
            Token::Text { text, .. } => {
                let data = string_cleanup(text.as_bytes());
                self.script.push_str(data.as_str());
                false
            },
            Token::Cdata { text, .. } => {
                self.script.push_str(text.as_str());
                false
            }
            _ => panic!(),
        }
    }
}

struct ErrorState {
    any_of_flag: bool,
    code: Option<String>,
}

impl ErrorState {
    fn empty(any_of_flag: bool) -> Self {
        ErrorState { any_of_flag, code: None }
    }
}

impl State for ErrorState {
    fn event(&mut self, token: Token, generated: &mut String) -> bool {
        match token {
            Token::Attribute { local, value, .. } => {
                if local.as_str() == "code" {
                    self.code = Some(String::from(value.as_str()))
                }
                false
            }
            Token::ElementEnd { end, .. } => {
                match end {
                    ElementEnd::Empty => {
                        generated.push_str(build_prefix(self.any_of_flag).as_str());
                        generated.push_str("check_error(&result, \"");
                        generated.push_str(self.code.as_ref().unwrap().as_str());
                        generated.push_str("\")");
                        if !self.any_of_flag { generated.push_str(";"); }
                        generated.push_str("\n");
                    },
                    _ => panic!()
                }

                true
            },
            _ => panic!()
        }
    }
}

struct AssertState {
    open_tag: String,
    fn_name: String,
    have_value: bool,
    any_of_flag: bool,

    assert: String,
}

impl AssertState {
    fn empty(open_tag: String, have_value: bool, any_of_flag: bool) -> Self {
        let mut fn_name = "check_".to_string();
        fn_name.push_str(open_tag.replace("-", "_").as_str());

        AssertState { open_tag, fn_name, have_value, any_of_flag, assert: String::new() }
    }
}

impl State for AssertState {
    fn event(&mut self, token: Token, generated: &mut String) -> bool {
        match token {
            Token::Attribute { local, value, .. } => {
                // TODO handle it
                false
            },
            Token::Text { text } => {
                let data = string_cleanup(text.as_bytes());
                self.assert.push_str(data.as_str());
                false
            }
            Token::Cdata { text, span: _ } => {
                self.assert.push_str(text.as_str());
                false
            }
            Token::ElementEnd { end, .. } => {
                match end {
                    ElementEnd::Open => false,
                    ElementEnd::Empty => {
                        if self.have_value {
                            let code = check_result_value(
                                self.fn_name.clone(),
                                self.assert.clone(),
                                self.any_of_flag
                            );
                            generated.push_str(code.as_str());
                        } else {
                            let code = check_result(
                                self.fn_name.clone(),
                                self.any_of_flag
                            );
                            generated.push_str(code.as_str());
                        }
                        true
                    },
                    ElementEnd::Close(prefix, local) => {
                        if local.as_str() == self.open_tag.as_str() {
                            if self.have_value {
                                let code = check_result_value(
                                    self.fn_name.clone(),
                                    self.assert.clone(),
                                    self.any_of_flag
                                );
                                generated.push_str(code.as_str());
                                true
                            } else {
                                panic!()
                            }
                        } else {
                            false
                        }
                    }
                }
            }
            _ => panic!()
        }
    }
}

struct ResultState {
    not_flag: bool,
    all_of_flag: bool,
    any_of_flag: bool,
    error: Option<ErrorState>,
    assert: Option<AssertState>,
}

impl ResultState {
    fn empty() -> Self {
        ResultState {
            not_flag: false, all_of_flag: false, any_of_flag: false, error: None, assert: None,
        }
    }
}

impl State for ResultState {
    fn event(&mut self, token: Token, generated: &mut String) -> bool {

        if let Some(state) = &mut self.error {
            if state.event(token, generated) {
                self.error = None
            } else {
                return false;
            }
        }

        if let Some(state) = &mut self.assert {
            if state.event(token, generated) {
                self.assert = None
            } else {
                return false;
            }
        }

        match token {
            Token::ElementStart { local, .. } => {
                let name = local.as_str();
                match name {
                    "all-of" => {
                        // TODO
                    }
                    "any-of" => {
                        self.any_of_flag = true;
                        generated.push_str("        assert!(true\n");
                    },
                    "not" => {
                        // TODO
                    }
                    "error" => self.error = Some(ErrorState::empty(self.any_of_flag)),
                    "assert-empty" |
                    "assert-true" |
                    "assert-false"  => {
                        self.assert = Some(
                            AssertState::empty(name.to_string(), false, self.any_of_flag)
                        )
                    },
                    "assert" |
                    "assert-eq" |
                    "assert-count" |
                    "assert-deep-eq" |
                    "assert-permutation" |
                    "assert-xml" |
                    "assert-type" |
                    "assert-string-value" |
                    "assert-serialization-error" |
                    "serialization-matches" => {
                        self.assert = Some(
                            AssertState::empty(name.to_string(), true, self.any_of_flag)
                        )
                    },
                    _ => panic!()
                }
                false
            }
            Token::ElementEnd { end, .. } => {
                match end {
                    ElementEnd::Open => {}
                    ElementEnd::Close(prefix, local) => {
                        match local.as_str() {
                            "all-of" => {
                                self.all_of_flag = false;
                                // TODO
                            }
                            "any-of" => {
                                self.any_of_flag = false;
                                generated.push_str("        );\n");
                            },
                            "not" => {
                                self.not_flag = false;
                                // TODO
                            }
                            "result" => return true,
                            _ => {}
                        }
                    }
                    ElementEnd::Empty => {}
                }
                false
            }
            _ => false
        }
    }
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

fn generate_tests(name: &str, file: &str, mut envs: HashMap<String, Environment>) -> String {
    println!("tests: {}", file);
    let data = fs::read_to_string(format!("./qt3tests/{}", file)).unwrap();
    let dir = folder(String::from(file.clone()));

    let mut generated = String::from("#[cfg(test)]
mod tests {
    use crate::tests::*;
    use std::fs;\n\n");

    let mut environment: Option<EnvironmentState> = None;
    let mut test_case: Option<TestCaseState> = None;

    for token in xmlparser::Tokenizer::from(data.as_str()) {
        let token = token.unwrap();

        if let Some(env) = &mut environment {
            if env.event(token, &mut generated) {
                if let Some(env) = env.to_data("./qt3tests/".to_string()) {
                    envs.insert(env.name.as_ref().unwrap().clone(), env);
                }
                environment = None;
            } else {
                continue
            }
        }

        if let Some(test) = &mut test_case {
            if test.event(token, &mut generated) {
                test_case = None;
            } else {
                continue
            }
        }

        match token {
            Token::ElementStart { prefix, local, span } => {
                match local.as_str() {
                    "environment" => environment = Some(EnvironmentState::empty()),
                    "test-case" => test_case = Some(TestCaseState::empty(dir.clone(), envs.clone())),
                    _ => {}
                }
            }
            Token::Attribute { .. } => {}
            Token::ElementEnd { .. } => {}
            _ => {}
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

    // check use of `use std::fs;`
    if !generated.contains("fs::") {
        generated = generated.replace("    use std::fs;\n", "");
    }

    fs::write(format!("src/xqts/{}", file), generated)
        .expect("Unable to write file");

    file
}

pub fn generate() {
    let data = fs::read_to_string("./qt3tests/catalog.xml").unwrap();

    let mut files = vec![];

    let mut envs: HashMap<String, Environment> = HashMap::new();
    let mut environment: Option<EnvironmentState> = None;

    let mut inside_test_set = false;
    let mut tests_name = "";
    let mut tests_file = "";

    for token in xmlparser::Tokenizer::from(data.as_str()) {
        let token = token.unwrap();

        if let Some(env) = &mut environment {
            if env.event(token, &mut String::new()) {
                if let Some(env) = env.to_data("./qt3tests/".to_string()) {
                    envs.insert(env.name.as_ref().unwrap().clone(), env);
                }
                environment = None;
            } else {
                continue
            }
        }

        match token {
            Token::ElementStart { prefix, local, span } => {
                match local.as_str() {
                    "environment" => environment = Some(EnvironmentState::empty()),
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
                            let file = generate_tests(tests_name, tests_file, envs.clone());

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