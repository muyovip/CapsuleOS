use anyhow::Result;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Identifier(String),
    Number(i64),
    String(String),
    Keyword(String),
    Operator(String),
    Eof,
}

pub fn tokenize(source: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    
    // Placeholder tokenization logic
    // In a real implementation, this would perform proper lexical analysis
    for word in source.split_whitespace() {
        if word.parse::<i64>().is_ok() {
            tokens.push(Token::Number(word.parse().unwrap()));
        } else {
            tokens.push(Token::Identifier(word.to_string()));
        }
    }
    
    tokens.push(Token::Eof);
    
    Ok(tokens)
}
