use std::fs;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    use crate::eval::*;
    use xmlparser::{Token, ElementEnd};
    use crate::eval::Object::Empty;

    #[test]
    fn eval_simple() {
        let data = fs::read_to_string("../qt3tests/map/get.xml").unwrap();

        let mut script_flag = false;
        let mut result_flag = false;

        let mut script = "";
        let mut evaluated: Object = Empty;

        for token in xmlparser::Tokenizer::from(data.as_str()) {
//            println!("{:?}", token);

            match token.unwrap() {
                Token::ElementStart { prefix, local, span } => {
                    match local.as_str() {
                        "test" => script_flag = true,
                        "result" => result_flag = true,
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
                                    evaluated = eval(script);
                                },
                                "result" => result_flag = false,
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
                        script = text.as_str();
                    }
                }
                Token::Cdata { text: _, span: _ } => {}
            }
        }
    }

    fn eval(input: &str) -> Object {
        println!("script: {:?}", input);

        let result = parse(input);

        println!("parsed: {:?}", result);

        if result.is_ok() {
            let (_, program) = result.unwrap();

            let mut env = Environment::new();

            let (new_env, result) = eval_statements(program, &mut env);

            println!("result: {:?}", result);

            result
        } else {
            Object::Empty // TODO: raise error!
        }
    }
}