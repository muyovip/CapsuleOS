pub mod error;
pub mod lexer;
pub mod token;

pub use error::ParseError;
pub use lexer::{normalize_line_endings, tokenize, tokenize_bytes};
pub use token::{Span, Token, TokenKind};

#[cfg(test)]
mod tests;
