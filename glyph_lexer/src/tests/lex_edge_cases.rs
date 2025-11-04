use crate::*;

#[test]
fn test_invalid_hex_no_digits() {
    let result = tokenize("0x");
    assert!(result.is_err());
}

#[test]
fn test_invalid_binary_no_digits() {
    let result = tokenize("0b");
    assert!(result.is_err());
}

#[test]
fn test_invalid_octal_no_digits() {
    let result = tokenize("0o");
    assert!(result.is_err());
}

#[test]
fn test_invalid_utf8_bytes() {
    let invalid_bytes = vec![0xFF, 0xFE, 0xFD];
    let result = tokenize_bytes(&invalid_bytes);
    assert!(result.is_err());
}

#[test]
fn test_valid_utf8_bytes() {
    let valid_bytes = "hello".as_bytes();
    let result = tokenize_bytes(valid_bytes);
    assert!(result.is_ok());
}

#[test]
fn test_number_followed_by_identifier() {
    let tokens = tokenize("123abc").unwrap();
    assert!(matches!(tokens[0].kind, TokenKind::IntegerLiteral { .. }));
    assert!(matches!(tokens[1].kind, TokenKind::Identifier(_)));
}

#[test]
fn test_dot_not_float() {
    let tokens = tokenize("1.foo").unwrap();
    assert!(matches!(tokens[0].kind, TokenKind::IntegerLiteral { .. }));
    assert!(matches!(tokens[1].kind, TokenKind::Delimiter('.')));
    assert!(matches!(tokens[2].kind, TokenKind::Identifier(_)));
}

#[test]
fn test_consecutive_operators() {
    let tokens = tokenize("+-*/").unwrap();
    assert_eq!(tokens.len(), 5);
    for i in 0..4 {
        assert!(matches!(tokens[i].kind, TokenKind::Operator(_)));
    }
}

#[test]
fn test_emoji_not_identifier_start() {
    let result = tokenize("😀");
    assert!(result.is_err());
}

#[test]
fn test_large_unicode_escape() {
    let tokens = tokenize(r#""\u{10FFFF}""#).unwrap();
    assert!(matches!(tokens[0].kind, TokenKind::StringLiteral(_)));
}

#[test]
fn test_zero_unicode_escape() {
    let tokens = tokenize(r#""\u{0}""#).unwrap();
    assert_eq!(tokens[0].kind, TokenKind::StringLiteral("\0".into()));
}

#[test]
fn test_all_delimiters() {
    let tokens = tokenize("(){}[];,.").unwrap();
    let delims: Vec<_> = tokens.iter()
        .filter_map(|t| match &t.kind {
            TokenKind::Delimiter(c) => Some(*c),
            _ => None,
        })
        .collect();
    assert_eq!(delims, vec!['(', ')', '{', '}', '[', ']', ';', ',', '.']);
}
