use crate::*;

#[test]
fn test_single_char_operators() {
    let ops = vec!["+", "-", "*", "/", "%", "<", ">", "=", "!", "&", "|", "^", "~", "?", ":"];
    for op in ops {
        let tokens = tokenize(op).unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Operator(op.to_string()));
    }
}

#[test]
fn test_double_colon() {
    let tokens = tokenize("::").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Operator("::".into()));
}

#[test]
fn test_arrow() {
    let tokens = tokenize("->").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Operator("->".into()));
}

#[test]
fn test_fat_arrow() {
    let tokens = tokenize("=>").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Operator("=>".into()));
}

#[test]
fn test_equality() {
    let tokens = tokenize("==").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Operator("==".into()));
}

#[test]
fn test_not_equal() {
    let tokens = tokenize("!=").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Operator("!=".into()));
}

#[test]
fn test_less_equal() {
    let tokens = tokenize("<=").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Operator("<=".into()));
}

#[test]
fn test_greater_equal() {
    let tokens = tokenize(">=").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Operator(">=".into()));
}

#[test]
fn test_logical_and() {
    let tokens = tokenize("&&").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Operator("&&".into()));
}

#[test]
fn test_logical_or() {
    let tokens = tokenize("||").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Operator("||".into()));
}

#[test]
fn test_compound_assignment_plus() {
    let tokens = tokenize("+=").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Operator("+=".into()));
}

#[test]
fn test_compound_assignment_minus() {
    let tokens = tokenize("-=").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Operator("-=".into()));
}

#[test]
fn test_left_shift() {
    let tokens = tokenize("<<").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Operator("<<".into()));
}

#[test]
fn test_right_shift() {
    let tokens = tokenize(">>").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Operator(">>".into()));
}

#[test]
fn test_longest_match_precedence() {
    let tokens = tokenize("==>").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Operator("==".into()));
    assert_eq!(tokens[1].kind, TokenKind::Operator(">".into()));
}

#[test]
fn test_operators_no_spaces() {
    let tokens = tokenize("a+b*c").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Identifier("a".into()));
    assert_eq!(tokens[1].kind, TokenKind::Operator("+".into()));
    assert_eq!(tokens[2].kind, TokenKind::Identifier("b".into()));
    assert_eq!(tokens[3].kind, TokenKind::Operator("*".into()));
    assert_eq!(tokens[4].kind, TokenKind::Identifier("c".into()));
}
