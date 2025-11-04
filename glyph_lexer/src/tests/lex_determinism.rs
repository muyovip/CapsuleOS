use crate::*;

#[test]
fn test_canonicalization_comments_whitespace() {
    let a = "let x=1; // comment\n";
    let b = "let   x  = 1 ;/* comment */";
    let ta = tokenize(a).unwrap();
    let tb = tokenize(b).unwrap();
    
    let kinds_a: Vec<_> = ta.iter()
        .filter(|t| !matches!(t.kind, TokenKind::Comment(_)))
        .map(|t| &t.kind)
        .collect();
    let kinds_b: Vec<_> = tb.iter()
        .filter(|t| !matches!(t.kind, TokenKind::Comment(_)))
        .map(|t| &t.kind)
        .collect();
    
    assert_eq!(kinds_a, kinds_b, "Token streams must be identical");
}

#[test]
fn test_run_twice_same_output() {
    let src = "let x = 42;";
    let tokens1 = tokenize(src).unwrap();
    let tokens2 = tokenize(src).unwrap();
    
    assert_eq!(tokens1.len(), tokens2.len());
    for (t1, t2) in tokens1.iter().zip(tokens2.iter()) {
        assert_eq!(t1.kind, t2.kind);
        assert_eq!(t1.span, t2.span);
    }
}

#[test]
fn test_different_formatting_same_semantics() {
    let inputs = vec![
        "fn foo(x, y) { return x + y; }",
        "fn foo(x,y){return x+y;}",
        "fn   foo  (  x  ,  y  )  {  return  x  +  y  ;  }",
        "fn\nfoo\n(\nx\n,\ny\n)\n{\nreturn\nx\n+\ny\n;\n}",
    ];
    
    let mut token_streams = Vec::new();
    for input in &inputs {
        token_streams.push(tokenize(input).unwrap());
    }
    
    for tokens in &token_streams {
        assert_eq!(tokens.len(), token_streams[0].len());
        for (t, t0) in tokens.iter().zip(token_streams[0].iter()) {
            assert_eq!(t.kind, t0.kind);
        }
    }
}
