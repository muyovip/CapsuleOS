use crate::*;

#[test]
fn test_simple_identifier() {
    let tokens = tokenize("hello").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Identifier("hello".into()));
}

#[test]
fn test_identifier_with_underscore() {
    let tokens = tokenize("hello_world").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Identifier("hello_world".into()));
}

#[test]
fn test_identifier_starting_with_underscore() {
    let tokens = tokenize("_private").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Identifier("_private".into()));
}

#[test]
fn test_identifier_with_numbers() {
    let tokens = tokenize("var123").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Identifier("var123".into()));
}

#[test]
fn test_unicode_identifier_greek() {
    let tokens = tokenize("π_radius").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Identifier("π_radius".into()));
}

#[test]
fn test_unicode_identifier_chinese() {
    let tokens = tokenize("变量").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Identifier("变量".into()));
}

#[test]
fn test_unicode_identifier_emoji_combining() {
    let tokens = tokenize("café").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Identifier("café".into()));
}

#[test]
fn test_multiple_identifiers() {
    let tokens = tokenize("foo bar baz").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Identifier("foo".into()));
    assert_eq!(tokens[1].kind, TokenKind::Identifier("bar".into()));
    assert_eq!(tokens[2].kind, TokenKind::Identifier("baz".into()));
}
