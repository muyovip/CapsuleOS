use crate::*;

#[test]
fn test_integer_zero() {
    let tokens = tokenize("0").unwrap();
    match &tokens[0].kind {
        TokenKind::IntegerLiteral { raw, canonical_value } => {
            assert_eq!(raw, "0");
            assert_eq!(canonical_value, "0");
        }
        _ => panic!("Expected integer literal"),
    }
}

#[test]
fn test_integer_simple() {
    let tokens = tokenize("123").unwrap();
    match &tokens[0].kind {
        TokenKind::IntegerLiteral { raw, canonical_value } => {
            assert_eq!(raw, "123");
            assert_eq!(canonical_value, "123");
        }
        _ => panic!("Expected integer literal"),
    }
}

#[test]
fn test_integer_with_underscores() {
    let tokens = tokenize("1_000_000").unwrap();
    match &tokens[0].kind {
        TokenKind::IntegerLiteral { raw, canonical_value } => {
            assert_eq!(raw, "1_000_000");
            assert_eq!(canonical_value, "1000000");
        }
        _ => panic!("Expected integer literal"),
    }
}

#[test]
fn test_hex_literal() {
    let tokens = tokenize("0xFF").unwrap();
    match &tokens[0].kind {
        TokenKind::IntegerLiteral { raw, canonical_value } => {
            assert_eq!(raw, "0xFF");
            assert_eq!(canonical_value, "0xff");
        }
        _ => panic!("Expected integer literal"),
    }
}

#[test]
fn test_hex_with_underscores() {
    let tokens = tokenize("0xDE_AD_BE_EF").unwrap();
    match &tokens[0].kind {
        TokenKind::IntegerLiteral { raw, canonical_value } => {
            assert_eq!(raw, "0xDE_AD_BE_EF");
            assert_eq!(canonical_value, "0xdeadbeef");
        }
        _ => panic!("Expected integer literal"),
    }
}

#[test]
fn test_binary_literal() {
    let tokens = tokenize("0b1010").unwrap();
    match &tokens[0].kind {
        TokenKind::IntegerLiteral { raw, canonical_value } => {
            assert_eq!(raw, "0b1010");
            assert_eq!(canonical_value, "0b1010");
        }
        _ => panic!("Expected integer literal"),
    }
}

#[test]
fn test_binary_with_underscores() {
    let tokens = tokenize("0b1111_0000").unwrap();
    match &tokens[0].kind {
        TokenKind::IntegerLiteral { raw, canonical_value } => {
            assert_eq!(raw, "0b1111_0000");
            assert_eq!(canonical_value, "0b11110000");
        }
        _ => panic!("Expected integer literal"),
    }
}

#[test]
fn test_octal_literal() {
    let tokens = tokenize("0o755").unwrap();
    match &tokens[0].kind {
        TokenKind::IntegerLiteral { raw, canonical_value } => {
            assert_eq!(raw, "0o755");
            assert_eq!(canonical_value, "0o755");
        }
        _ => panic!("Expected integer literal"),
    }
}

#[test]
fn test_float_simple() {
    let tokens = tokenize("3.14").unwrap();
    match &tokens[0].kind {
        TokenKind::FloatLiteral { raw, canonical_value } => {
            assert_eq!(raw, "3.14");
            assert_eq!(canonical_value, "3.14");
        }
        _ => panic!("Expected float literal"),
    }
}

#[test]
fn test_float_with_exponent() {
    let tokens = tokenize("1e10").unwrap();
    match &tokens[0].kind {
        TokenKind::FloatLiteral { raw, canonical_value } => {
            assert_eq!(raw, "1e10");
            assert_eq!(canonical_value, "1e10");
        }
        _ => panic!("Expected float literal"),
    }
}

#[test]
fn test_float_with_negative_exponent() {
    let tokens = tokenize("1.2e-3").unwrap();
    match &tokens[0].kind {
        TokenKind::FloatLiteral { raw, canonical_value } => {
            assert_eq!(raw, "1.2e-3");
            assert_eq!(canonical_value, "1.2e-3");
        }
        _ => panic!("Expected float literal"),
    }
}

#[test]
fn test_float_with_underscores() {
    let tokens = tokenize("1_234.567_890").unwrap();
    match &tokens[0].kind {
        TokenKind::FloatLiteral { raw, canonical_value } => {
            assert_eq!(raw, "1_234.567_890");
            assert_eq!(canonical_value, "1234.567890");
        }
        _ => panic!("Expected float literal"),
    }
}
