use crate::*;

#[test]
fn test_empty_string() {
    let tokens = tokenize(r#""""#).unwrap();
    assert_eq!(tokens[0].kind, TokenKind::StringLiteral("".into()));
}

#[test]
fn test_simple_string() {
    let tokens = tokenize(r#""hello""#).unwrap();
    assert_eq!(tokens[0].kind, TokenKind::StringLiteral("hello".into()));
}

#[test]
fn test_string_with_spaces() {
    let tokens = tokenize(r#""hello world""#).unwrap();
    assert_eq!(tokens[0].kind, TokenKind::StringLiteral("hello world".into()));
}

#[test]
fn test_string_escape_newline() {
    let tokens = tokenize(r#""line1\nline2""#).unwrap();
    assert_eq!(tokens[0].kind, TokenKind::StringLiteral("line1\nline2".into()));
}

#[test]
fn test_string_escape_tab() {
    let tokens = tokenize(r#""tab\there""#).unwrap();
    assert_eq!(tokens[0].kind, TokenKind::StringLiteral("tab\there".into()));
}

#[test]
fn test_string_escape_quote() {
    let tokens = tokenize(r#""say \"hello\"""#).unwrap();
    assert_eq!(tokens[0].kind, TokenKind::StringLiteral(r#"say "hello""#.into()));
}

#[test]
fn test_string_escape_backslash() {
    let tokens = tokenize(r#""path\\file""#).unwrap();
    assert_eq!(tokens[0].kind, TokenKind::StringLiteral(r"path\file".into()));
}

#[test]
fn test_string_unicode_escape() {
    let tokens = tokenize(r#""\u{1F600}""#).unwrap();
    assert_eq!(tokens[0].kind, TokenKind::StringLiteral("😀".into()));
}

#[test]
fn test_string_hex_escape() {
    let tokens = tokenize(r#""\x41""#).unwrap();
    assert_eq!(tokens[0].kind, TokenKind::StringLiteral("A".into()));
}

#[test]
fn test_unterminated_string() {
    let result = tokenize(r#""unterminated"#);
    assert!(result.is_err());
    match result {
        Err(ParseError::Lexical { message, .. }) => {
            assert!(message.contains("Unterminated"));
        }
        _ => panic!("Expected lexical error"),
    }
}

#[test]
fn test_char_literal() {
    let tokens = tokenize("'a'").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::CharLiteral('a'));
}

#[test]
fn test_char_escape_newline() {
    let tokens = tokenize(r"'\n'").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::CharLiteral('\n'));
}

#[test]
fn test_char_unicode() {
    let tokens = tokenize("'π'").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::CharLiteral('π'));
}

#[test]
fn test_char_unterminated() {
    let result = tokenize("'a");
    assert!(result.is_err());
}
