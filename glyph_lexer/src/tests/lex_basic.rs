use crate::*;

#[test]
fn test_basic_let() {
    let src = "let x = 1;";
    let tokens = tokenize(src).expect("lex");
    let kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            &TokenKind::Identifier("let".to_string()),
            &TokenKind::Identifier("x".to_string()),
            &TokenKind::Operator("=".to_string()),
            &TokenKind::IntegerLiteral {
                raw: "1".into(),
                canonical_value: "1".into()
            },
            &TokenKind::Delimiter(';'),
            &TokenKind::Eof
        ]
    );
}

#[test]
fn test_empty_input() {
    let tokens = tokenize("").unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Eof);
}

#[test]
fn test_whitespace_only() {
    let tokens = tokenize("   \n\t  ").unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Eof);
}

#[test]
fn test_parentheses() {
    let tokens = tokenize("(a, b)").unwrap();
    let kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            &TokenKind::Delimiter('('),
            &TokenKind::Identifier("a".into()),
            &TokenKind::Delimiter(','),
            &TokenKind::Identifier("b".into()),
            &TokenKind::Delimiter(')'),
            &TokenKind::Eof
        ]
    );
}

#[test]
fn test_braces() {
    let tokens = tokenize("{ x }").unwrap();
    let kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            &TokenKind::Delimiter('{'),
            &TokenKind::Identifier("x".into()),
            &TokenKind::Delimiter('}'),
            &TokenKind::Eof
        ]
    );
}

#[test]
fn test_brackets() {
    let tokens = tokenize("[1, 2]").unwrap();
    let kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            &TokenKind::Delimiter('['),
            &TokenKind::IntegerLiteral {
                raw: "1".into(),
                canonical_value: "1".into()
            },
            &TokenKind::Delimiter(','),
            &TokenKind::IntegerLiteral {
                raw: "2".into(),
                canonical_value: "2".into()
            },
            &TokenKind::Delimiter(']'),
            &TokenKind::Eof
        ]
    );
}

#[test]
fn test_mixed_delimiters() {
    let tokens = tokenize("({[,;]})").unwrap();
    let kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            &TokenKind::Delimiter('('),
            &TokenKind::Delimiter('{'),
            &TokenKind::Delimiter('['),
            &TokenKind::Delimiter(','),
            &TokenKind::Delimiter(';'),
            &TokenKind::Delimiter(']'),
            &TokenKind::Delimiter('}'),
            &TokenKind::Delimiter(')'),
            &TokenKind::Eof
        ]
    );
}

#[test]
fn test_simple_expression() {
    let tokens = tokenize("x + y").unwrap();
    let kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            &TokenKind::Identifier("x".into()),
            &TokenKind::Operator("+".into()),
            &TokenKind::Identifier("y".into()),
            &TokenKind::Eof
        ]
    );
}

#[test]
fn test_function_call() {
    let tokens = tokenize("func(arg1, arg2);").unwrap();
    let kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            &TokenKind::Identifier("func".into()),
            &TokenKind::Delimiter('('),
            &TokenKind::Identifier("arg1".into()),
            &TokenKind::Delimiter(','),
            &TokenKind::Identifier("arg2".into()),
            &TokenKind::Delimiter(')'),
            &TokenKind::Delimiter(';'),
            &TokenKind::Eof
        ]
    );
}
