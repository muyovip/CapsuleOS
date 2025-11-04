use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Span { start, end }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Identifier(String),
    IntegerLiteral {
        raw: String,
        canonical_value: String,
    },
    FloatLiteral {
        raw: String,
        canonical_value: String,
    },
    StringLiteral(String),
    CharLiteral(char),
    Symbol(String),
    Operator(String),
    Delimiter(char),
    Comment(String),
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, start: usize, end: usize) -> Self {
        Token {
            kind,
            span: Span { start, end },
        }
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.span == other.span
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Identifier(s) => write!(f, "Identifier({})", s),
            TokenKind::IntegerLiteral { canonical_value, .. } => {
                write!(f, "IntegerLiteral({})", canonical_value)
            }
            TokenKind::FloatLiteral { canonical_value, .. } => {
                write!(f, "FloatLiteral({})", canonical_value)
            }
            TokenKind::StringLiteral(s) => write!(f, "StringLiteral({:?})", s),
            TokenKind::CharLiteral(c) => write!(f, "CharLiteral('{}')", c),
            TokenKind::Symbol(s) => write!(f, "Symbol({})", s),
            TokenKind::Operator(s) => write!(f, "Operator({})", s),
            TokenKind::Delimiter(c) => write!(f, "Delimiter('{}')", c),
            TokenKind::Comment(s) => write!(f, "Comment({:?})", s),
            TokenKind::Eof => write!(f, "Eof"),
        }
    }
}
