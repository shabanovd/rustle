
pub enum Error {
    Nothing,
}

pub struct ParserError(Error);

impl XQuery {
    pub fn evaluate(&self, context: &Context) {
        
    }
}

pub struct XQueryEngine {
    parser: Parser,
}

impl XQueryEngine {
    
    fn compile(&self, script: &str) -> Result<XQuery, ParserError>  {
        let tokenizer = Tokenizer::new(xpath);
    }
    
}