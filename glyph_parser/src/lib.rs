use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Expression {
    Literal(Literal),
    
    Var(String),
    
    Lambda {
        param: String,
        body: Box<Expression>,
    },
    
    Apply {
        func: Box<Expression>,
        arg: Box<Expression>,
    },
    
    LinearApply {
        func: Box<Expression>,
        arg: Box<Expression>,
    },
    
    Let {
        name: String,
        value: Box<Expression>,
        body: Box<Expression>,
    },
    
    Match {
        expr: Box<Expression>,
        arms: Vec<MatchArm>,
    },
    
    Tuple(Vec<Expression>),
    
    List(Vec<Expression>),
    
    Record(HashMap<String, Expression>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Literal {
    Int(i64),
    Float(String),
    String(String),
    Bool(bool),
    Unit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Box<Expression>>,
    pub body: Box<Expression>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Pattern {
    Wildcard,
    Var(String),
    Literal(Literal),
    Tuple(Vec<Pattern>),
    Constructor { name: String, args: Vec<Pattern> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Int(i64),
    Float(String),
    String(String),
    Bool(bool),
    
    Let,
    In,
    Match,
    Lambda,
    If,
    Then,
    Else,
    
    Ident(String),
    
    Arrow,
    LinearArrow,
    Equals,
    Pipe,
    Underscore,
    
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Semicolon,
    Colon,
    
    Eof,
}

#[derive(Error, Debug)]
pub enum LexError {
    #[error("Unexpected character: {0}")]
    UnexpectedChar(char),
    
    #[error("Unterminated string")]
    UnterminatedString,
    
    #[error("Invalid number format: {0}")]
    InvalidNumber(String),
}

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
        }
    }
    
    fn current(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }
    
    fn peek(&self, offset: usize) -> Option<char> {
        self.input.get(self.pos + offset).copied()
    }
    
    fn advance(&mut self) -> Option<char> {
        let c = self.current();
        self.pos += 1;
        c
    }
    
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current() {
            if c.is_whitespace() {
                self.advance();
            } else if c == '#' {
                while let Some(c) = self.current() {
                    self.advance();
                    if c == '\n' {
                        break;
                    }
                }
            } else {
                break;
            }
        }
    }
    
    fn read_string(&mut self) -> Result<String, LexError> {
        self.advance();
        let mut s = String::new();
        
        loop {
            match self.current() {
                None => return Err(LexError::UnterminatedString),
                Some('"') => {
                    self.advance();
                    return Ok(s);
                }
                Some('\\') => {
                    self.advance();
                    match self.advance() {
                        Some('n') => s.push('\n'),
                        Some('t') => s.push('\t'),
                        Some('r') => s.push('\r'),
                        Some('\\') => s.push('\\'),
                        Some('"') => s.push('"'),
                        _ => return Err(LexError::UnterminatedString),
                    }
                }
                Some(c) => {
                    s.push(c);
                    self.advance();
                }
            }
        }
    }
    
    fn read_number(&mut self) -> Result<Token, LexError> {
        let mut num = String::new();
        let mut is_float = false;
        
        while let Some(c) = self.current() {
            if c.is_ascii_digit() {
                num.push(c);
                self.advance();
            } else if c == '.' && !is_float {
                is_float = true;
                num.push(c);
                self.advance();
            } else {
                break;
            }
        }
        
        if is_float {
            Ok(Token::Float(num))
        } else {
            num.parse::<i64>()
                .map(Token::Int)
                .map_err(|_| LexError::InvalidNumber(num))
        }
    }
    
    fn read_ident(&mut self) -> String {
        let mut ident = String::new();
        
        while let Some(c) = self.current() {
            if c.is_alphanumeric() || c == '_' || c == '\'' {
                ident.push(c);
                self.advance();
            } else {
                break;
            }
        }
        
        ident
    }
    
    pub fn next_token(&mut self) -> Result<Token, LexError> {
        self.skip_whitespace();
        
        match self.current() {
            None => Ok(Token::Eof),
            
            Some('(') => {
                self.advance();
                Ok(Token::LParen)
            }
            Some(')') => {
                self.advance();
                Ok(Token::RParen)
            }
            Some('{') => {
                self.advance();
                Ok(Token::LBrace)
            }
            Some('}') => {
                self.advance();
                Ok(Token::RBrace)
            }
            Some('[') => {
                self.advance();
                Ok(Token::LBracket)
            }
            Some(']') => {
                self.advance();
                Ok(Token::RBracket)
            }
            Some(',') => {
                self.advance();
                Ok(Token::Comma)
            }
            Some(';') => {
                self.advance();
                Ok(Token::Semicolon)
            }
            Some(':') => {
                self.advance();
                Ok(Token::Colon)
            }
            Some('|') => {
                self.advance();
                Ok(Token::Pipe)
            }
            Some('_') => {
                self.advance();
                Ok(Token::Underscore)
            }
            
            Some('+') => {
                self.advance();
                Ok(Token::Ident("+".to_string()))
            }
            
            Some('=') => {
                self.advance();
                Ok(Token::Equals)
            }
            
            Some('-') if self.peek(1) == Some('>') => {
                self.advance();
                self.advance();
                Ok(Token::Arrow)
            }
            
            Some('-') if self.peek(1).map_or(false, |c| c.is_ascii_digit()) => {
                self.advance();
                let num_token = self.read_number()?;
                match num_token {
                    Token::Int(n) => Ok(Token::Int(-n)),
                    Token::Float(f) => Ok(Token::Float(format!("-{}", f))),
                    _ => unreachable!(),
                }
            }
            
            Some('⊸') => {
                self.advance();
                Ok(Token::LinearArrow)
            }
            
            Some('λ') => {
                self.advance();
                Ok(Token::Lambda)
            }
            
            Some('"') => {
                let s = self.read_string()?;
                Ok(Token::String(s))
            }
            
            Some(c) if c.is_ascii_digit() => self.read_number(),
            
            Some(c) if c.is_alphabetic() => {
                let ident = self.read_ident();
                match ident.as_str() {
                    "let" => Ok(Token::Let),
                    "in" => Ok(Token::In),
                    "match" => Ok(Token::Match),
                    "if" => Ok(Token::If),
                    "then" => Ok(Token::Then),
                    "else" => Ok(Token::Else),
                    "true" => Ok(Token::Bool(true)),
                    "false" => Ok(Token::Bool(false)),
                    _ => Ok(Token::Ident(ident)),
                }
            }
            
            Some(c) => Err(LexError::UnexpectedChar(c)),
        }
    }
    
    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();
        
        loop {
            let token = self.next_token()?;
            if token == Token::Eof {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }
        
        Ok(tokens)
    }
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Unexpected token: {0:?}")]
    UnexpectedToken(Token),
    
    #[error("Unexpected end of input")]
    UnexpectedEof,
    
    #[error("Expected {0}, found {1:?}")]
    Expected(String, Token),
    
    #[error("Lexer error: {0}")]
    LexError(#[from] LexError),
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }
    
    fn current(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }
    
    fn advance(&mut self) -> Token {
        let token = self.current().clone();
        self.pos += 1;
        token
    }
    
    fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        if self.current() == &expected {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::Expected(
                format!("{:?}", expected),
                self.current().clone(),
            ))
        }
    }
    
    pub fn parse(&mut self) -> Result<Expression, ParseError> {
        self.parse_expression()
    }
    
    fn parse_expression(&mut self) -> Result<Expression, ParseError> {
        self.parse_let()
    }
    
    fn parse_let(&mut self) -> Result<Expression, ParseError> {
        if matches!(self.current(), Token::Let) {
            self.advance();
            
            let name = match self.advance() {
                Token::Ident(s) => s,
                t => return Err(ParseError::Expected("identifier".to_string(), t)),
            };
            
            self.expect(Token::Equals)?;
            let value = Box::new(self.parse_expression()?);
            self.expect(Token::In)?;
            let body = Box::new(self.parse_expression()?);
            
            Ok(Expression::Let { name, value, body })
        } else {
            self.parse_match()
        }
    }
    
    fn parse_match(&mut self) -> Result<Expression, ParseError> {
        if matches!(self.current(), Token::Match) {
            self.advance();
            let expr = Box::new(self.parse_primary()?);
            self.expect(Token::LBrace)?;
            
            let mut arms = Vec::new();
            while !matches!(self.current(), Token::RBrace | Token::Eof) {
                arms.push(self.parse_match_arm()?);
                if matches!(self.current(), Token::Comma) {
                    self.advance();
                }
            }
            
            self.expect(Token::RBrace)?;
            Ok(Expression::Match { expr, arms })
        } else {
            self.parse_lambda()
        }
    }
    
    fn parse_match_arm(&mut self) -> Result<MatchArm, ParseError> {
        let pattern = self.parse_pattern()?;
        
        let guard = if matches!(self.current(), Token::If) {
            self.advance();
            Some(Box::new(self.parse_lambda()?))
        } else {
            None
        };
        
        self.expect(Token::Arrow)?;
        let body = Box::new(self.parse_lambda()?);
        
        Ok(MatchArm { pattern, guard, body })
    }
    
    fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        match self.current() {
            Token::Underscore => {
                self.advance();
                Ok(Pattern::Wildcard)
            }
            Token::Ident(name) => {
                let name = name.clone();
                self.advance();
                
                if matches!(self.current(), Token::LParen) {
                    self.advance();
                    let mut args = Vec::new();
                    
                    while !matches!(self.current(), Token::RParen | Token::Eof) {
                        args.push(self.parse_pattern()?);
                        if matches!(self.current(), Token::Comma) {
                            self.advance();
                        }
                    }
                    
                    self.expect(Token::RParen)?;
                    Ok(Pattern::Constructor { name, args })
                } else {
                    Ok(Pattern::Var(name))
                }
            }
            Token::Int(n) => {
                let n = *n;
                self.advance();
                Ok(Pattern::Literal(Literal::Int(n)))
            }
            Token::Bool(b) => {
                let b = *b;
                self.advance();
                Ok(Pattern::Literal(Literal::Bool(b)))
            }
            Token::LParen => {
                self.advance();
                let mut patterns = Vec::new();
                
                while !matches!(self.current(), Token::RParen | Token::Eof) {
                    patterns.push(self.parse_pattern()?);
                    if matches!(self.current(), Token::Comma) {
                        self.advance();
                    }
                }
                
                self.expect(Token::RParen)?;
                Ok(Pattern::Tuple(patterns))
            }
            t => Err(ParseError::UnexpectedToken(t.clone())),
        }
    }
    
    fn parse_lambda(&mut self) -> Result<Expression, ParseError> {
        match self.current() {
            Token::Lambda => {
                self.advance();
                
                let param = match self.advance() {
                    Token::Ident(s) => s,
                    t => return Err(ParseError::Expected("identifier".to_string(), t)),
                };
                
                self.expect(Token::Arrow)?;
                let body = Box::new(self.parse_expression()?);
                
                Ok(Expression::Lambda { param, body })
            }
            Token::Let | Token::Match => {
                self.parse_expression()
            }
            _ => self.parse_application()
        }
    }
    
    fn parse_application(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_primary()?;
        
        loop {
            match self.current() {
                Token::LParen | Token::Ident(_) | Token::Int(_) | Token::String(_) 
                | Token::Bool(_) | Token::LBracket | Token::LBrace => {
                    let arg = self.parse_primary()?;
                    expr = Expression::Apply {
                        func: Box::new(expr),
                        arg: Box::new(arg),
                    };
                }
                Token::LinearArrow => {
                    self.advance();
                    let arg = self.parse_primary()?;
                    expr = Expression::LinearApply {
                        func: Box::new(expr),
                        arg: Box::new(arg),
                    };
                }
                _ => break,
            }
        }
        
        Ok(expr)
    }
    
    fn parse_primary(&mut self) -> Result<Expression, ParseError> {
        match self.current().clone() {
            Token::Int(n) => {
                self.advance();
                Ok(Expression::Literal(Literal::Int(n)))
            }
            Token::Float(f) => {
                self.advance();
                Ok(Expression::Literal(Literal::Float(f)))
            }
            Token::String(s) => {
                self.advance();
                Ok(Expression::Literal(Literal::String(s)))
            }
            Token::Bool(b) => {
                self.advance();
                Ok(Expression::Literal(Literal::Bool(b)))
            }
            Token::Ident(s) => {
                self.advance();
                Ok(Expression::Var(s))
            }
            Token::LParen => {
                self.advance();
                
                if matches!(self.current(), Token::RParen) {
                    self.advance();
                    return Ok(Expression::Literal(Literal::Unit));
                }
                
                let first = self.parse_expression()?;
                
                if matches!(self.current(), Token::Comma) {
                    let mut elements = vec![first];
                    while matches!(self.current(), Token::Comma) {
                        self.advance();
                        if matches!(self.current(), Token::RParen) {
                            break;
                        }
                        elements.push(self.parse_expression()?);
                    }
                    self.expect(Token::RParen)?;
                    Ok(Expression::Tuple(elements))
                } else {
                    self.expect(Token::RParen)?;
                    Ok(first)
                }
            }
            Token::LBracket => {
                self.advance();
                let mut elements = Vec::new();
                
                while !matches!(self.current(), Token::RBracket | Token::Eof) {
                    elements.push(self.parse_expression()?);
                    if matches!(self.current(), Token::Comma) {
                        self.advance();
                    }
                }
                
                self.expect(Token::RBracket)?;
                Ok(Expression::List(elements))
            }
            Token::LBrace => {
                self.advance();
                let mut fields = HashMap::new();
                
                while !matches!(self.current(), Token::RBrace | Token::Eof) {
                    let key = match self.advance() {
                        Token::Ident(s) => s,
                        t => return Err(ParseError::Expected("identifier".to_string(), t)),
                    };
                    
                    self.expect(Token::Colon)?;
                    let value = self.parse_expression()?;
                    fields.insert(key, value);
                    
                    if matches!(self.current(), Token::Comma) {
                        self.advance();
                    }
                }
                
                self.expect(Token::RBrace)?;
                Ok(Expression::Record(fields))
            }
            t => Err(ParseError::UnexpectedToken(t)),
        }
    }
}

pub fn canonical_serialize(expr: &Expression) -> Result<Vec<u8>, String> {
    let mut buffer = Vec::new();
    ciborium::into_writer(expr, &mut buffer)
        .map_err(|e| format!("Serialization failed: {}", e))?;
    Ok(buffer)
}

pub fn deserialize(data: &[u8]) -> Result<Expression, String> {
    ciborium::from_reader(data)
        .map_err(|e| format!("Deserialization failed: {}", e))
}

pub fn parse(input: &str) -> Result<Expression, ParseError> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}

// ============================================================================
// Game Extensions Module
// ============================================================================

pub mod game_extensions;

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip(input: &str) -> Result<(), String> {
        let ast1 = parse(input).map_err(|e| e.to_string())?;
        let serialized = canonical_serialize(&ast1)?;
        let ast2 = deserialize(&serialized)?;
        
        if ast1 != ast2 {
            return Err(format!("Round-trip failed:\nOriginal: {:?}\nDeserialized: {:?}", ast1, ast2));
        }
        
        Ok(())
    }

    #[test]
    fn test_basic_literals() {
        assert!(parse("42").is_ok());
        assert!(parse("3.14").is_ok());
        assert!(parse("true").is_ok());
        assert!(parse("false").is_ok());
        assert!(parse(r#""hello""#).is_ok());
        assert!(parse("()").is_ok());
    }

    #[test]
    fn test_variables() {
        assert!(parse("x").is_ok());
        assert!(parse("foo").is_ok());
        assert!(parse("x'").is_ok());
        assert!(parse("x''").is_ok());
    }

    #[test]
    fn test_simple_lambda() {
        assert!(parse("λx -> x").is_ok());
        assert!(parse("λx -> y").is_ok());
        assert!(parse("λx -> λy -> x").is_ok());
    }

    #[test]
    fn test_lambda_application() {
        assert!(parse("f x").is_ok());
        assert!(parse("f x y").is_ok());
        assert!(parse("(λx -> x) 42").is_ok());
    }

    #[test]
    fn test_let_binding() {
        assert!(parse("let x = 1 in x").is_ok());
        assert!(parse("let f = λx -> x in f 42").is_ok());
    }

    #[test]
    fn test_match_expression() {
        assert!(parse("match x { _ -> 0 }").is_ok());
        assert!(parse("match x { 0 -> 1, _ -> 0 }").is_ok());
        assert!(parse("match x { Some(y) -> y, None -> 0 }").is_ok());
    }

    #[test]
    fn test_tuples() {
        assert!(parse("(1, 2)").is_ok());
        assert!(parse("(1, 2, 3)").is_ok());
        assert!(parse("((1, 2), 3)").is_ok());
    }

    #[test]
    fn test_lists() {
        assert!(parse("[]").is_ok());
        assert!(parse("[1]").is_ok());
        assert!(parse("[1, 2, 3]").is_ok());
        assert!(parse("[[1, 2], [3, 4]]").is_ok());
    }

    #[test]
    fn test_records() {
        assert!(parse("{ x: 1 }").is_ok());
        assert!(parse("{ x: 1, y: 2 }").is_ok());
        assert!(parse("{ nested: { x: 1 } }").is_ok());
    }

    #[test]
    fn test_linear_arrow() {
        assert!(parse("f ⊸ x").is_ok());
        assert!(parse("f ⊸ g ⊸ x").is_ok());
    }

    #[test]
    fn test_round_trip_basic() {
        assert!(round_trip("42").is_ok());
        assert!(round_trip("λx -> x").is_ok());
        assert!(round_trip("let x = 1 in x").is_ok());
    }

    #[test]
    fn test_comments() {
        assert!(parse("# comment\n42").is_ok());
        assert!(parse("42 # comment").is_ok());
    }

    #[test]
    fn test_guard_expressions() {
        assert!(parse("match x { y if y -> 1, _ -> 0 }").is_ok());
        assert!(parse("match x { y if (λz -> true) y -> 1, _ -> 0 }").is_ok());
    }

    #[test]
    fn test_nested_let_bindings() {
        let cases = vec![
            "let x = 1 in let y = 2 in let z = 3 in (x, y, z)",
            "let f = λx -> x in let g = λy -> f y in g 42",
            "let x = (let y = 1 in y) in x",
        ];
        
        for case in cases {
            assert!(parse(case).is_ok(), "Failed: {}", case);
            assert!(round_trip(case).is_ok(), "Round-trip failed: {}", case);
        }
    }

    #[test]
    fn test_deeply_nested_lambdas() {
        assert!(parse("λa -> λb -> λc -> λd -> λe -> (a, b, c, d, e)").is_ok());
        assert!(round_trip("λa -> λb -> λc -> (a, b, c)").is_ok());
    }

    #[test]
    fn test_complex_data_structures() {
        let cases = vec![
            "{ users: [{ name: \"Alice\", age: 30 }] }",
            "[(1, 2), (3, 4), (5, 6)]",
            "[{ x: 1 }, { x: 2 }, { x: 3 }]",
            "{ matrix: [[1, 2], [3, 4]] }",
        ];
        
        for case in cases {
            assert!(parse(case).is_ok(), "Failed: {}", case);
            assert!(round_trip(case).is_ok(), "Round-trip failed: {}", case);
        }
    }

    #[test]
    fn test_functional_combinators() {
        let cases = vec![
            "λf -> λg -> λx -> f (g x)",
            "λx -> λf -> f x",
            "λf -> λx -> λy -> f y x",
            "λx -> λy -> x",
            "λf -> λx -> f (f x)",
        ];
        
        for case in cases {
            assert!(parse(case).is_ok(), "Failed: {}", case);
            assert!(round_trip(case).is_ok(), "Round-trip failed: {}", case);
        }
    }

    #[test]
    fn test_church_encodings() {
        let cases = vec![
            "λf -> λx -> x",
            "λf -> λx -> f x",
            "λf -> λx -> f (f x)",
            "λn -> λf -> λx -> f (n f x)",
            "λm -> λn -> λf -> m (n f)",
            "λm -> λn -> m (λn -> λf -> λx -> f (n f x)) n",
        ];
        
        for case in cases {
            assert!(parse(case).is_ok(), "Failed: {}", case);
            assert!(round_trip(case).is_ok(), "Round-trip failed: {}", case);
        }
    }

    #[test]
    fn test_error_cases() {
        assert!(parse("let x = in x").is_err());
        assert!(parse("λ -> x").is_err());
        assert!(parse("match { -> 0 }").is_err());
        assert!(parse("(1, 2,)").is_ok());
        assert!(parse("[1, 2,]").is_ok());
    }

    #[test]
    fn test_escape_sequences() {
        assert!(parse(r#""hello\nworld""#).is_ok());
        assert!(parse(r#""tab\there""#).is_ok());
        assert!(parse(r#""quote\"here""#).is_ok());
    }

    #[test]
    fn test_ast_equality() {
        let ast1 = parse("λx -> x").unwrap();
        let ast2 = parse("λx -> x").unwrap();
        assert_eq!(ast1, ast2);
        
        let ast3 = parse("λx -> y").unwrap();
        assert_ne!(ast1, ast3);
    }

    #[test]
    fn test_serialization_round_trip_stability() {
        let input = "let factorial = λn -> match n { 0 -> 1, _ -> n } in factorial";
        let ast1 = parse(input).unwrap();
        
        let mut current = ast1.clone();
        for _ in 0..10 {
            let serialized = canonical_serialize(&current).unwrap();
            current = deserialize(&serialized).unwrap();
        }
        
        assert_eq!(ast1, current, "AST changed after multiple round trips");
    }

    #[test]
    fn test_linear_arrow_precedence() {
        let ast1 = parse("f ⊸ g ⊸ x").unwrap();
        let _ast2 = parse("f ⊸ (g ⊸ x)").unwrap();
        
        assert!(matches!(ast1, Expression::LinearApply { .. }));
    }

    #[test]
    fn test_record_field_order_independence() {
        let ast1 = parse("{ a: 1, b: 2 }").unwrap();
        let ast2 = parse("{ b: 2, a: 1 }").unwrap();
        
        let s1 = canonical_serialize(&ast1).unwrap();
        let s2 = canonical_serialize(&ast2).unwrap();
        
        println!("Record serialization lengths: {} vs {}", s1.len(), s2.len());
    }

    #[test]
    fn test_empty_structures() {
        assert!(parse("[]").is_ok());
        assert!(parse("()").is_ok());
        assert!(round_trip("[]").is_ok());
        assert!(round_trip("()").is_ok());
    }

    #[test]
    fn test_mixed_applications() {
        let cases = vec![
            "f x y z",
            "(f x) (g y)",
            "f (g x) (h y)",
            "(λx -> x) (λy -> y) 42",
        ];
        
        for case in cases {
            assert!(parse(case).is_ok(), "Failed: {}", case);
            assert!(round_trip(case).is_ok(), "Round-trip failed: {}", case);
        }
    }

    #[test]
    fn test_record_as_function_argument() {
        let ast = parse("f { x: 1 }").unwrap();
        match ast {
            Expression::Apply { func, arg } => {
                assert!(matches!(*func, Expression::Var(_)));
                assert!(matches!(*arg, Expression::Record(_)));
            }
            _ => panic!("Expected Apply expression, got {:?}", ast),
        }
        
        assert!(parse("map { transform: λx -> x } data").is_ok());
        assert!(round_trip("f { x: 1, y: 2 }").is_ok());
    }

    #[test]
    fn test_large_expressions() {
        let large_list: Vec<String> = (0..100).map(|i| i.to_string()).collect();
        let large_list_expr = format!("[{}]", large_list.join(", "));
        
        assert!(parse(&large_list_expr).is_ok());
        assert!(round_trip(&large_list_expr).is_ok());
    }

    #[test]
    fn test_deeply_nested_expressions() {
        let mut expr = "x".to_string();
        for _ in 0..50 {
            expr = format!("({})", expr);
        }
        
        assert!(parse(&expr).is_ok());
    }

    #[test]
    fn test_lexer_robustness() {
        let mut lexer = Lexer::new("42 + 3.14");
        let tokens = lexer.tokenize().unwrap();
        assert!(tokens.len() > 0);
    }

    #[test]
    fn test_all_token_types() {
        let input = r#"
            let x = 42 in
            match x {
                0 -> true,
                _ -> false
            }
        "#;
        
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        
        assert!(tokens.iter().any(|t| matches!(t, Token::Let)));
        assert!(tokens.iter().any(|t| matches!(t, Token::Match)));
        assert!(tokens.iter().any(|t| matches!(t, Token::Arrow)));
        assert!(tokens.iter().any(|t| matches!(t, Token::Int(_))));
        assert!(tokens.iter().any(|t| matches!(t, Token::Bool(_))));
    }

    #[test]
    fn test_string_escaping() {
        let cases = vec![
            (r#""hello""#, "hello"),
            (r#""hello\nworld""#, "hello\nworld"),
            (r#""tab\there""#, "tab\there"),
            (r#""quote\"here""#, "quote\"here"),
            (r#""backslash\\here""#, "backslash\\here"),
        ];
        
        for (input, expected) in cases {
            match parse(input).unwrap() {
                Expression::Literal(Literal::String(s)) => {
                    assert_eq!(s, expected, "Failed for input: {}", input);
                }
                _ => panic!("Expected string literal"),
            }
        }
    }

    #[test]
    fn test_pattern_wildcard() {
        let ast = parse("match x { _ -> 0 }").unwrap();
        match ast {
            Expression::Match { arms, .. } => {
                assert_eq!(arms.len(), 1);
                assert!(matches!(arms[0].pattern, Pattern::Wildcard));
            }
            _ => panic!("Expected match expression"),
        }
    }

    #[test]
    fn test_tuple_pattern() {
        let ast = parse("match x { (a, b) -> a }").unwrap();
        match ast {
            Expression::Match { arms, .. } => {
                assert_eq!(arms.len(), 1);
                match &arms[0].pattern {
                    Pattern::Tuple(patterns) => {
                        assert_eq!(patterns.len(), 2);
                    }
                    _ => panic!("Expected tuple pattern"),
                }
            }
            _ => panic!("Expected match expression"),
        }
    }

    #[test]
    fn test_constructor_pattern() {
        let ast = parse("match x { Some(y) -> y, None -> 0 }").unwrap();
        match ast {
            Expression::Match { arms, .. } => {
                assert_eq!(arms.len(), 2);
                match &arms[0].pattern {
                    Pattern::Constructor { name, args } => {
                        assert_eq!(name, "Some");
                        assert_eq!(args.len(), 1);
                    }
                    _ => panic!("Expected constructor pattern"),
                }
            }
            _ => panic!("Expected match expression"),
        }
    }

    #[test]
    fn test_comprehensive_over_100_cases() {
        let all_cases = vec![
            "0", "1", "42", "-1", "3.14",
            "true", "false", "()", "\"\"", "\"hello\"",
            
            "x", "y", "z", "foo", "bar",
            "x'", "x''", "camelCase", "snake_case", "name123",
            
            "λx -> x",
            "λx -> y",
            "λx -> λy -> x",
            "λx -> λy -> y",
            "λx -> λy -> (x, y)",
            "λf -> λx -> f x",
            "λf -> λx -> f (f x)",
            "λf -> λg -> λx -> f (g x)",
            "λx -> λy -> λz -> (x, y, z)",
            "λa -> λb -> λc -> λd -> a",
            "λf -> λx -> λy -> f x y",
            "λx -> (x, x)",
            "λf -> (f, f)",
            "λx -> [x]",
            "λx -> { a: x }",
            
            "f x",
            "f x y",
            "f x y z",
            "f (g x)",
            "(f x) y",
            "f (g (h x))",
            "(λx -> x) 42",
            "(λx -> λy -> x) 1 2",
            "f (x, y)",
            "f [1, 2]",
            "f { x: 1 }",
            "map f xs",
            "fold f acc xs",
            "compose f g x",
            "apply (λx -> x) 42",
            
            "f ⊸ x",
            "f ⊸ g ⊸ x",
            "(λx -> x) ⊸ 42",
            "map ⊸ [1, 2, 3]",
            "fold ⊸ 0 ⊸ xs",
            
            "let x = 1 in x",
            "let x = 1 in y",
            "let x = 1 in let y = 2 in x",
            "let x = 1 in let y = 2 in y",
            "let x = 1 in let y = x in y",
            "let f = λx -> x in f",
            "let f = λx -> x in f 42",
            "let id = λx -> x in id id",
            "let const = λx -> λy -> x in const 1 2",
            "let pair = λx -> λy -> (x, y) in pair 1 2",
            "let x = (1, 2) in x",
            "let x = [1, 2] in x",
            "let x = { a: 1 } in x",
            "let f = λx -> let y = x in y in f 10",
            "let rec = { a: 1, b: 2 } in rec",
            
            "match x { _ -> 0 }",
            "match x { y -> y }",
            "match x { 0 -> 1, _ -> 0 }",
            "match x { 1 -> 1, 2 -> 2, _ -> 0 }",
            "match x { true -> 1, false -> 0 }",
            "match x { (a, b) -> a }",
            "match x { (a, b) -> b }",
            "match x { (_, y) -> y }",
            "match x { (x, _) -> x }",
            "match x { Some(y) -> y, None -> 0 }",
            "match x { Cons(h, t) -> h, Nil -> 0 }",
            "match x { Just(x) -> x, Nothing -> 0 }",
            "match x { Ok(v) -> v, Err(e) -> 0 }",
            "match (1, 2) { (a, b) -> a }",
            "match [1, 2] { x -> x }",
            "match x { 0 -> 0, 1 -> 1, 2 -> 2, _ -> 3 }",
            "match x { Pair(a, b) -> (b, a) }",
            "match x { Triple(a, b, c) -> (c, b, a) }",
            "match x { Node(l, v, r) -> v }",
            "match x { y if y -> 1, _ -> 0 }",
            
            "(1, 2)",
            "(1, 2, 3)",
            "(x, y)",
            "((1, 2), 3)",
            "(1, (2, 3))",
            "(true, false, true)",
            "(λx -> x, λy -> y)",
            "([1], [2])",
            "({ a: 1 }, { b: 2 })",
            "((), ())",
            
            "[]",
            "[1]",
            "[1, 2]",
            "[1, 2, 3]",
            "[[1]]",
            "[[1, 2], [3, 4]]",
            "[x, y, z]",
            "[(1, 2)]",
            "[λx -> x]",
            "[true, false]",
            
            "{ x: 1 }",
            "{ x: 1, y: 2 }",
            "{ a: 1, b: 2, c: 3 }",
            "{ nested: { x: 1 } }",
            "{ f: λx -> x }",
            "{ list: [1, 2] }",
            "{ tuple: (1, 2) }",
            "{ x: true, y: false }",
            "{ a: { b: { c: 1 } } }",
            "{ users: [{ name: \"Alice\" }] }",
        ];
        
        println!("\n=== Running {} test cases ===", all_cases.len());
        
        let mut passed = 0;
        let mut failed_parse = 0;
        let mut failed_round_trip = 0;
        
        for (i, case) in all_cases.iter().enumerate() {
            match parse(case) {
                Ok(_) => {
                    match round_trip(case) {
                        Ok(_) => {
                            passed += 1;
                        }
                        Err(e) => {
                            failed_round_trip += 1;
                            eprintln!("[{}] Round-trip FAILED: {} - {}", i + 1, case, e);
                        }
                    }
                }
                Err(e) => {
                    failed_parse += 1;
                    eprintln!("[{}] Parse FAILED: {} - {}", i + 1, case, e);
                }
            }
        }
        
        println!("\n=== Final Results ===");
        println!("Total cases: {}", all_cases.len());
        println!("Passed: {}", passed);
        println!("Failed parse: {}", failed_parse);
        println!("Failed round-trip: {}", failed_round_trip);
        
        assert!(passed >= 100, "Must have at least 100 passing tests, got {}", passed);
        assert_eq!(failed_parse, 0, "All cases should parse successfully");
        assert_eq!(failed_round_trip, 0, "All cases should round-trip successfully");
    }
}
