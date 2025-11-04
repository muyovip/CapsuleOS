use crate::*;

#[test]
fn test_line_comment() {
    let tokens = tokenize("// this is a comment").unwrap();
    match &tokens[0].kind {
        TokenKind::Comment(c) => {
            assert_eq!(c, "this is a comment");
        }
        _ => panic!("Expected comment token"),
    }
}

#[test]
fn test_line_comment_with_code() {
    let tokens = tokenize("let x = 1; // comment").unwrap();
    assert!(matches!(tokens[0].kind, TokenKind::Identifier(_)));
    let comment_idx = tokens.iter().position(|t| matches!(t.kind, TokenKind::Comment(_))).unwrap();
    match &tokens[comment_idx].kind {
        TokenKind::Comment(c) => {
            assert_eq!(c, "comment");
        }
        _ => panic!("Expected comment"),
    }
}

#[test]
fn test_block_comment_simple() {
    let tokens = tokenize("/* comment */").unwrap();
    match &tokens[0].kind {
        TokenKind::Comment(c) => {
            assert_eq!(c, "comment");
        }
        _ => panic!("Expected comment token"),
    }
}

#[test]
fn test_block_comment_multiline() {
    let tokens = tokenize("/* line1\nline2 */").unwrap();
    match &tokens[0].kind {
        TokenKind::Comment(c) => {
            assert!(c.contains("line1"));
            assert!(c.contains("line2"));
        }
        _ => panic!("Expected comment token"),
    }
}

#[test]
fn test_nested_block_comment() {
    let tokens = tokenize("/* outer /* inner */ still outer */").unwrap();
    match &tokens[0].kind {
        TokenKind::Comment(c) => {
            assert!(c.contains("outer"));
            assert!(c.contains("inner"));
        }
        _ => panic!("Expected comment token"),
    }
}

#[test]
fn test_deeply_nested_comments() {
    let tokens = tokenize("/* 1 /* 2 /* 3 */ 2 */ 1 */").unwrap();
    match &tokens[0].kind {
        TokenKind::Comment(_) => {
            // Just verify it parses correctly
        }
        _ => panic!("Expected comment token"),
    }
}

#[test]
fn test_unterminated_block_comment() {
    let result = tokenize("/* unterminated");
    assert!(result.is_err());
    match result {
        Err(ParseError::Lexical { message, .. }) => {
            assert!(message.contains("Unterminated"));
        }
        _ => panic!("Expected lexical error"),
    }
}

#[test]
fn test_comment_canonicalization() {
    let tokens1 = tokenize("//   spaces   ").unwrap();
    let tokens2 = tokenize("// spaces").unwrap();
    match (&tokens1[0].kind, &tokens2[0].kind) {
        (TokenKind::Comment(c1), TokenKind::Comment(c2)) => {
            assert_eq!(c1, c2);
        }
        _ => panic!("Expected comments"),
    }
}
