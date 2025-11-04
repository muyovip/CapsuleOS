use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    Lexical {
        message: String,
        span: Option<(usize, usize)>,
    },
}

impl ParseError {
    pub fn lexical(msg: impl Into<String>, span: Option<(usize, usize)>) -> Self {
        ParseError::Lexical {
            message: msg.into(),
            span,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Lexical { message, span } => {
                if let Some((start, end)) = span {
                    write!(f, "Lexical error at {}..{}: {}", start, end, message)
                } else {
                    write!(f, "Lexical error: {}", message)
                }
            }
        }
    }
}

impl std::error::Error for ParseError {}
