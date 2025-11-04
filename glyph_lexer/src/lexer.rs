use crate::error::ParseError;
use crate::token::{Token, TokenKind};
use std::str::CharIndices;
use unicode_xid::UnicodeXID;

pub fn normalize_line_endings(input: &str) -> String {
    input.replace("\r\n", "\n").replace('\r', "\n")
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, ParseError> {
    let normalized = normalize_line_endings(input);
    let mut lexer = Lexer::new(&normalized);
    lexer.tokenize_all()
}

pub fn tokenize_bytes(bytes: &[u8]) -> Result<Vec<Token>, ParseError> {
    let input = std::str::from_utf8(bytes)
        .map_err(|e| ParseError::lexical(format!("Invalid UTF-8: {}", e), None))?;
    tokenize(input)
}

struct Lexer<'a> {
    input: &'a str,
    chars: CharIndices<'a>,
    cur_pos: usize,
    peek: Option<(usize, char)>,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        let mut chars = input.char_indices();
        let peek = chars.next();
        Lexer {
            input,
            chars,
            cur_pos: 0,
            peek,
        }
    }

    fn peek_char(&self) -> Option<(usize, char)> {
        self.peek
    }

    fn advance(&mut self) -> Option<(usize, char)> {
        let current = self.peek;
        self.peek = self.chars.next();
        if let Some((pos, _)) = current {
            self.cur_pos = pos;
        }
        current
    }

    fn peek_ahead(&self, n: usize) -> Option<char> {
        self.input[self.cur_pos..]
            .chars()
            .nth(n)
    }

    fn consume_whitespace(&mut self) {
        while let Some((_, ch)) = self.peek_char() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn lex_identifier(&mut self) -> Token {
        let start = self.cur_pos;
        let mut ident = String::new();

        while let Some((_pos, ch)) = self.peek_char() {
            if UnicodeXID::is_xid_continue(ch) || ch == '_' {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        let end = self.peek_char().map_or(self.input.len(), |(pos, _)| pos);
        Token::new(TokenKind::Identifier(ident), start, end)
    }

    fn lex_number(&mut self) -> Result<Token, ParseError> {
        let start = self.cur_pos;
        let mut raw = String::new();

        if self.peek_char().map(|(_, ch)| ch) == Some('0') {
            if let Some(next_ch) = self.peek_ahead(1) {
                match next_ch {
                    'x' | 'X' => return self.lex_hex_number(start),
                    'b' | 'B' => return self.lex_bin_number(start),
                    'o' | 'O' => return self.lex_oct_number(start),
                    _ => {}
                }
            }
        }

        while let Some((pos, ch)) = self.peek_char() {
            if ch.is_ascii_digit() || ch == '_' {
                raw.push(ch);
                self.advance();
            } else if ch == '.' {
                // Look at what comes after the '.'
                let after_dot = self.input[(pos + 1)..].chars().next();
                if after_dot.map_or(false, |c| c.is_ascii_digit()) {
                    return self.lex_float(start, raw);
                } else {
                    break;
                }
            } else if ch == 'e' || ch == 'E' {
                return self.lex_float(start, raw);
            } else {
                break;
            }
        }

        let end = self.peek_char().map_or(self.input.len(), |(pos, _)| pos);
        let canonical = raw.replace('_', "");
        Ok(Token::new(
            TokenKind::IntegerLiteral {
                raw,
                canonical_value: canonical,
            },
            start,
            end,
        ))
    }

    fn lex_hex_number(&mut self, start: usize) -> Result<Token, ParseError> {
        let mut raw = String::new();
        raw.push_str("0x");
        self.advance();
        self.advance();

        let mut has_digits = false;
        while let Some((_, ch)) = self.peek_char() {
            if ch.is_ascii_hexdigit() || ch == '_' {
                raw.push(ch);
                if ch != '_' {
                    has_digits = true;
                }
                self.advance();
            } else {
                break;
            }
        }

        if !has_digits {
            return Err(ParseError::lexical(
                "Invalid hex literal",
                Some((start, self.cur_pos)),
            ));
        }

        let end = self.peek_char().map_or(self.input.len(), |(pos, _)| pos);
        let canonical = raw.replace('_', "").to_lowercase();
        Ok(Token::new(
            TokenKind::IntegerLiteral {
                raw,
                canonical_value: canonical,
            },
            start,
            end,
        ))
    }

    fn lex_bin_number(&mut self, start: usize) -> Result<Token, ParseError> {
        let mut raw = String::new();
        raw.push_str("0b");
        self.advance();
        self.advance();

        let mut has_digits = false;
        while let Some((_, ch)) = self.peek_char() {
            if ch == '0' || ch == '1' || ch == '_' {
                raw.push(ch);
                if ch != '_' {
                    has_digits = true;
                }
                self.advance();
            } else {
                break;
            }
        }

        if !has_digits {
            return Err(ParseError::lexical(
                "Invalid binary literal",
                Some((start, self.cur_pos)),
            ));
        }

        let end = self.peek_char().map_or(self.input.len(), |(pos, _)| pos);
        let canonical = raw.replace('_', "");
        Ok(Token::new(
            TokenKind::IntegerLiteral {
                raw,
                canonical_value: canonical,
            },
            start,
            end,
        ))
    }

    fn lex_oct_number(&mut self, start: usize) -> Result<Token, ParseError> {
        let mut raw = String::new();
        raw.push_str("0o");
        self.advance();
        self.advance();

        let mut has_digits = false;
        while let Some((_, ch)) = self.peek_char() {
            if ('0'..='7').contains(&ch) || ch == '_' {
                raw.push(ch);
                if ch != '_' {
                    has_digits = true;
                }
                self.advance();
            } else {
                break;
            }
        }

        if !has_digits {
            return Err(ParseError::lexical(
                "Invalid octal literal",
                Some((start, self.cur_pos)),
            ));
        }

        let end = self.peek_char().map_or(self.input.len(), |(pos, _)| pos);
        let canonical = raw.replace('_', "");
        Ok(Token::new(
            TokenKind::IntegerLiteral {
                raw,
                canonical_value: canonical,
            },
            start,
            end,
        ))
    }

    fn lex_float(&mut self, start: usize, mut raw: String) -> Result<Token, ParseError> {
        if let Some((_, '.')) = self.peek_char() {
            raw.push('.');
            self.advance();
            while let Some((_, ch)) = self.peek_char() {
                if ch.is_ascii_digit() || ch == '_' {
                    raw.push(ch);
                    self.advance();
                } else {
                    break;
                }
            }
        }

        if let Some((_, ch)) = self.peek_char() {
            if ch == 'e' || ch == 'E' {
                raw.push(ch);
                self.advance();
                if let Some((_, sign)) = self.peek_char() {
                    if sign == '+' || sign == '-' {
                        raw.push(sign);
                        self.advance();
                    }
                }
                let mut has_exp_digits = false;
                while let Some((_, ch)) = self.peek_char() {
                    if ch.is_ascii_digit() || ch == '_' {
                        raw.push(ch);
                        if ch != '_' {
                            has_exp_digits = true;
                        }
                        self.advance();
                    } else {
                        break;
                    }
                }
                if !has_exp_digits {
                    return Err(ParseError::lexical(
                        "Invalid float exponent",
                        Some((start, self.cur_pos)),
                    ));
                }
            }
        }

        let end = self.peek_char().map_or(self.input.len(), |(pos, _)| pos);
        let canonical = raw.replace('_', "");
        Ok(Token::new(
            TokenKind::FloatLiteral {
                raw,
                canonical_value: canonical,
            },
            start,
            end,
        ))
    }

    fn lex_string(&mut self) -> Result<Token, ParseError> {
        let start = self.cur_pos;
        self.advance();

        let mut value = String::new();
        loop {
            match self.peek_char() {
                None => {
                    return Err(ParseError::lexical(
                        "Unterminated string literal",
                        Some((start, self.cur_pos)),
                    ));
                }
                Some((_, '"')) => {
                    self.advance();
                    break;
                }
                Some((_, '\\')) => {
                    self.advance();
                    let escaped = self.parse_escape(start)?;
                    value.push_str(&escaped);
                }
                Some((_, ch)) => {
                    value.push(ch);
                    self.advance();
                }
            }
        }

        let end = self.cur_pos;
        Ok(Token::new(TokenKind::StringLiteral(value), start, end))
    }

    fn lex_char(&mut self) -> Result<Token, ParseError> {
        let start = self.cur_pos;
        self.advance();

        let ch = match self.peek_char() {
            None => {
                return Err(ParseError::lexical(
                    "Unterminated char literal",
                    Some((start, self.cur_pos)),
                ));
            }
            Some((_, '\\')) => {
                self.advance();
                let escaped = self.parse_escape(start)?;
                if escaped.len() != 1 {
                    escaped
                        .chars()
                        .next()
                        .ok_or_else(|| ParseError::lexical("Empty escape in char", Some((start, self.cur_pos))))?
                } else {
                    escaped.chars().next().unwrap()
                }
            }
            Some((_, ch)) => {
                self.advance();
                ch
            }
        };

        match self.peek_char() {
            Some((_, '\'')) => {
                self.advance();
                let end = self.cur_pos;
                Ok(Token::new(TokenKind::CharLiteral(ch), start, end))
            }
            _ => Err(ParseError::lexical(
                "Unterminated char literal",
                Some((start, self.cur_pos)),
            )),
        }
    }

    fn parse_escape(&mut self, start: usize) -> Result<String, ParseError> {
        match self.peek_char() {
            Some((_, 'n')) => {
                self.advance();
                Ok("\n".to_string())
            }
            Some((_, 't')) => {
                self.advance();
                Ok("\t".to_string())
            }
            Some((_, 'r')) => {
                self.advance();
                Ok("\r".to_string())
            }
            Some((_, '\\')) => {
                self.advance();
                Ok("\\".to_string())
            }
            Some((_, '"')) => {
                self.advance();
                Ok("\"".to_string())
            }
            Some((_, '\'')) => {
                self.advance();
                Ok("'".to_string())
            }
            Some((_, '0')) => {
                self.advance();
                Ok("\0".to_string())
            }
            Some((_, 'x')) => {
                self.advance();
                self.parse_hex_escape(start, 2)
            }
            Some((_, 'u')) => {
                self.advance();
                self.parse_unicode_escape(start)
            }
            _ => Err(ParseError::lexical(
                "Invalid escape sequence",
                Some((start, self.cur_pos)),
            )),
        }
    }

    fn parse_hex_escape(&mut self, start: usize, count: usize) -> Result<String, ParseError> {
        let mut hex = String::new();
        for _ in 0..count {
            match self.peek_char() {
                Some((_, ch)) if ch.is_ascii_hexdigit() => {
                    hex.push(ch);
                    self.advance();
                }
                _ => {
                    return Err(ParseError::lexical(
                        "Invalid hex escape",
                        Some((start, self.cur_pos)),
                    ));
                }
            }
        }
        let value = u32::from_str_radix(&hex, 16)
            .map_err(|_| ParseError::lexical("Invalid hex value", Some((start, self.cur_pos))))?;
        char::from_u32(value)
            .map(|c| c.to_string())
            .ok_or_else(|| ParseError::lexical("Invalid Unicode codepoint", Some((start, self.cur_pos))))
    }

    fn parse_unicode_escape(&mut self, start: usize) -> Result<String, ParseError> {
        if self.peek_char().map(|(_, ch)| ch) != Some('{') {
            return Err(ParseError::lexical(
                "Expected '{' in Unicode escape",
                Some((start, self.cur_pos)),
            ));
        }
        self.advance();

        let mut hex = String::new();
        loop {
            match self.peek_char() {
                Some((_, '}')) => {
                    self.advance();
                    break;
                }
                Some((_, ch)) if ch.is_ascii_hexdigit() => {
                    hex.push(ch);
                    self.advance();
                }
                _ => {
                    return Err(ParseError::lexical(
                        "Invalid Unicode escape",
                        Some((start, self.cur_pos)),
                    ));
                }
            }
        }

        if hex.is_empty() || hex.len() > 6 {
            return Err(ParseError::lexical(
                "Invalid Unicode escape length",
                Some((start, self.cur_pos)),
            ));
        }

        let value = u32::from_str_radix(&hex, 16)
            .map_err(|_| ParseError::lexical("Invalid hex value", Some((start, self.cur_pos))))?;
        char::from_u32(value)
            .map(|c| c.to_string())
            .ok_or_else(|| ParseError::lexical("Invalid Unicode codepoint", Some((start, self.cur_pos))))
    }

    fn lex_line_comment(&mut self) -> Result<Token, ParseError> {
        let start = self.cur_pos;
        self.advance();
        self.advance();

        let mut content = String::new();
        while let Some((_, ch)) = self.peek_char() {
            if ch == '\n' {
                break;
            }
            content.push(ch);
            self.advance();
        }

        let end = self.peek_char().map_or(self.input.len(), |(pos, _)| pos);
        let canonical = canonicalize_comment(&content);
        Ok(Token::new(TokenKind::Comment(canonical), start, end))
    }

    fn lex_block_comment(&mut self) -> Result<Token, ParseError> {
        let start = self.cur_pos;
        self.advance();
        self.advance();

        let mut content = String::new();
        let mut nesting = 1;

        while nesting > 0 {
            match self.peek_char() {
                None => {
                    return Err(ParseError::lexical(
                        "Unterminated block comment",
                        Some((start, self.cur_pos)),
                    ));
                }
                Some((pos, '/')) => {
                    let next_char = self.input[(pos + 1)..].chars().next();
                    if next_char == Some('*') {
                        nesting += 1;
                        content.push('/');
                        content.push('*');
                        self.advance();
                        self.advance();
                    } else {
                        content.push('/');
                        self.advance();
                    }
                }
                Some((pos, '*')) => {
                    let next_char = self.input[(pos + 1)..].chars().next();
                    if next_char == Some('/') {
                        nesting -= 1;
                        if nesting > 0 {
                            content.push('*');
                            content.push('/');
                        }
                        self.advance();
                        self.advance();
                    } else {
                        content.push('*');
                        self.advance();
                    }
                }
                Some((_, ch)) => {
                    content.push(ch);
                    self.advance();
                }
            }
        }

        let end = self.cur_pos;
        let canonical = canonicalize_comment(&content);
        Ok(Token::new(TokenKind::Comment(canonical), start, end))
    }

    fn match_operator(&mut self) -> Option<Token> {
        let start = self.cur_pos;
        
        const OPERATORS: &[&str] = &[
            "::", "->", "=>", "==", "!=", "<=", ">=", "&&", "||",
            "+=", "-=", "*=", "/=", "%=", "<<", ">>",
            "+", "-", "*", "/", "%", "<", ">", "=", "!", "&", "|", "^", "~", "?", ":",
        ];

        for &op in OPERATORS {
            if self.matches_str(op) {
                for _ in 0..op.len() {
                    self.advance();
                }
                let end = self.peek_char().map_or(self.input.len(), |(pos, _)| pos);
                return Some(Token::new(TokenKind::Operator(op.to_string()), start, end));
            }
        }

        None
    }

    fn matches_str(&self, s: &str) -> bool {
        self.input[self.cur_pos..].starts_with(s)
    }

    fn tokenize_all(&mut self) -> Result<Vec<Token>, ParseError> {
        let mut tokens = Vec::new();

        loop {
            let (pos, ch) = match self.peek_char() {
                Some(p) => p,
                None => {
                    tokens.push(Token::new(TokenKind::Eof, self.cur_pos, self.cur_pos));
                    break;
                }
            };

            match ch {
                c if c.is_whitespace() => {
                    self.consume_whitespace();
                    continue;
                }
                '/' => {
                    let next_char = self.input[(pos + 1)..].chars().next();
                    if next_char == Some('*') {
                        self.cur_pos = pos;
                        tokens.push(self.lex_block_comment()?);
                        continue;
                    } else if next_char == Some('/') {
                        self.cur_pos = pos;
                        tokens.push(self.lex_line_comment()?);
                        continue;
                    } else {
                        self.cur_pos = pos;
                        if let Some(op) = self.match_operator() {
                            tokens.push(op);
                        } else {
                            return Err(ParseError::lexical(
                                format!("Unexpected character: {}", ch),
                                Some((pos, pos + ch.len_utf8())),
                            ));
                        }
                    }
                }
                '"' => {
                    self.cur_pos = pos;
                    tokens.push(self.lex_string()?);
                }
                '\'' => {
                    self.cur_pos = pos;
                    tokens.push(self.lex_char()?);
                }
                c if UnicodeXID::is_xid_start(c) || c == '_' => {
                    self.cur_pos = pos;
                    tokens.push(self.lex_identifier());
                }
                c if c.is_ascii_digit() => {
                    self.cur_pos = pos;
                    tokens.push(self.lex_number()?);
                }
                '(' | ')' | '{' | '}' | '[' | ']' | ',' | ';' | '.' => {
                    self.advance();
                    let end = self.peek_char().map_or(self.input.len(), |(pos, _)| pos);
                    tokens.push(Token::new(TokenKind::Delimiter(ch), pos, end));
                }
                _ => {
                    self.cur_pos = pos;
                    if let Some(op) = self.match_operator() {
                        tokens.push(op);
                    } else {
                        return Err(ParseError::lexical(
                            format!("Unexpected character: {}", ch),
                            Some((pos, pos + ch.len_utf8())),
                        ));
                    }
                }
            }
        }

        Ok(tokens)
    }
}

fn canonicalize_comment(content: &str) -> String {
    let trimmed = content.trim();
    let normalized = trimmed.replace("\r\n", "\n").replace('\r', "\n");
    let mut result = String::new();
    let mut prev_ws = false;

    for ch in normalized.chars() {
        if ch.is_whitespace() && ch != '\n' {
            if !prev_ws {
                result.push(' ');
                prev_ws = true;
            }
        } else {
            result.push(ch);
            prev_ws = false;
        }
    }

    result
}
