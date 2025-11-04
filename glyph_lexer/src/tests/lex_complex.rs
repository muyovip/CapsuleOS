use crate::*;

#[test]
fn test_complex_expression() {
    let src = "(x + y) * z - w / 2";
    let tokens = tokenize(src).unwrap();
    let kinds: Vec<_> = tokens.iter().map(|t| &t.kind).collect();
    assert!(matches!(kinds[0], TokenKind::Delimiter('(')));
    assert!(matches!(kinds[1], TokenKind::Identifier(_)));
    assert!(matches!(kinds[2], TokenKind::Operator(_)));
}

#[test]
fn test_function_definition() {
    let src = "fn add(a, b) -> int { return a + b; }";
    let tokens = tokenize(src).unwrap();
    assert!(tokens.iter().any(|t| matches!(&t.kind, TokenKind::Identifier(s) if s == "fn")));
    assert!(tokens.iter().any(|t| matches!(&t.kind, TokenKind::Identifier(s) if s == "add")));
    assert!(tokens.iter().any(|t| matches!(&t.kind, TokenKind::Operator(s) if s == "->")));
}

#[test]
fn test_nested_structures() {
    let src = "{ { { x } } }";
    let tokens = tokenize(src).unwrap();
    let open_count = tokens.iter().filter(|t| matches!(t.kind, TokenKind::Delimiter('{'))).count();
    let close_count = tokens.iter().filter(|t| matches!(t.kind, TokenKind::Delimiter('}'))).count();
    assert_eq!(open_count, 3);
    assert_eq!(close_count, 3);
}

#[test]
fn test_array_initialization() {
    let src = "let arr = [1, 2, 3, 4, 5];";
    let tokens = tokenize(src).unwrap();
    let int_count = tokens.iter().filter(|t| matches!(t.kind, TokenKind::IntegerLiteral { .. })).count();
    assert_eq!(int_count, 5);
}

#[test]
fn test_mixed_literals() {
    let src = r#"let x = 42; let y = 3.14; let z = "hello"; let c = 'a';"#;
    let tokens = tokenize(src).unwrap();
    assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::IntegerLiteral { .. })));
    assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::FloatLiteral { .. })));
    assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::StringLiteral(_))));
    assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::CharLiteral(_))));
}

#[test]
fn test_chained_method_calls() {
    let src = "obj.method1().method2().method3()";
    let tokens = tokenize(src).unwrap();
    let dot_count = tokens.iter().filter(|t| matches!(t.kind, TokenKind::Delimiter('.'))).count();
    assert_eq!(dot_count, 3);
}
