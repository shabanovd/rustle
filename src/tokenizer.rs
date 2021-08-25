
pub struct Tokenizer {
    nextToken: Token,
    inputOffset: usize
}

impl Tokenizer {
    
    fn tokenize(input: &str, start: usize) -> Tokenizer {
        let tokenizer = Tokenizer {
            nextToken: Token::EOF,
            inputOffset: start
        };

        tokenizer.lookAhead();
        tokenizer.next();

        tokenizer
    }
    
    fn lookAhead() {
        
    }
    
    fn next() {
        
    }
}