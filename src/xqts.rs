use std::fs;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    use crate::eval::*;
    use xmlparser::{Token, ElementEnd};
    use crate::eval::Object::Empty;
    use crate::fns::object_to_string;

    #[test]
    fn eval_simple() {
        let data = fs::read_to_string("./qt3tests/prod/StepExpr.xml").unwrap();

        let mut script_flag = false;
        let mut result_flag = false;

        let mut script = String::new();
        let mut evaluated: Object = Empty;

        let mut expect_result: String = String::new();

        for token in xmlparser::Tokenizer::from(data.as_str()) {
//            println!("{:?}", token);

            match token.unwrap() {
                Token::ElementStart { prefix, local, span } => {
                    match local.as_str() {
                        "test" => {
                            script_flag = true;

                            script.clear();
                            evaluated = Empty;
                        },
                        "result" => {
                            result_flag = true;
                        },
                        _ => {}
                    }
                },
                Token::ElementEnd { end, span } => {
                    match end {
                        ElementEnd::Open => {},
                        ElementEnd::Close(prefix, local) => {
                            match local.as_str() {
                                "test" => {
                                    script_flag = false;
                                    evaluated = eval(script.as_str());
                                },
                                "result" => {
                                    result_flag = false;
                                    let result = object_to_string(&evaluated);
                                    assert_eq!(expect_result, result)
                                },
                                _ => {}
                            }
                        },
                        ElementEnd::Empty => {},
                    }
                }
                Token::Declaration { version: _, encoding: _, standalone: _, span: _ } => {}
                Token::ProcessingInstruction { target: _, content: _, span: _ } => {}
                Token::Comment { text: _, span: _ } => {}
                Token::DtdStart { name: _, external_id: _, span: _ } => {}
                Token::EmptyDtd { name: _, external_id: _, span: _ } => {}
                Token::EntityDeclaration { name: _, definition: _, span: _ } => {}
                Token::DtdEnd { span: _ } => {}
                Token::Attribute { prefix: _, local: _, value: _, span: _ } => {}
                Token::Text { text } => {
                    if script_flag {
                        script.push_str(text.as_str());
                    }
                }
                Token::Cdata { text: _, span: _ } => {}
            }
        }
    }

    fn eval(input: &str) -> Object {
        println!("script: {:?}", input);

        let parsed = parse(input);
        if parsed.is_ok() {
            let (_, program) = parsed.unwrap();

            let mut env = Environment::new();

            let (new_env, result) = eval_statements(program, Box::new(env), &Object::Empty);

            println!("result: {:?}", result);

            result
        } else {
            panic!("error {:?}", parsed)
        }
    }
}