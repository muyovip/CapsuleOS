use crate::*;

#[test]
fn test_crlf_normalization() {
    let input1 = "let\r\nx";
    let input2 = "let\nx";
    let tokens1 = tokenize(input1).unwrap();
    let tokens2 = tokenize(input2).unwrap();
    assert_eq!(tokens1.len(), tokens2.len());
    for (t1, t2) in tokens1.iter().zip(tokens2.iter()) {
        assert_eq!(t1.kind, t2.kind);
    }
}

#[test]
fn test_cr_normalization() {
    let input1 = "let\rx";
    let input2 = "let\nx";
    let tokens1 = tokenize(input1).unwrap();
    let tokens2 = tokenize(input2).unwrap();
    assert_eq!(tokens1.len(), tokens2.len());
    for (t1, t2) in tokens1.iter().zip(tokens2.iter()) {
        assert_eq!(t1.kind, t2.kind);
    }
}

#[test]
fn test_multiple_spaces_normalization() {
    let input1 = "let   x    =    1;";
    let input2 = "let x = 1;";
    let tokens1 = tokenize(input1).unwrap();
    let tokens2 = tokenize(input2).unwrap();
    assert_eq!(tokens1.len(), tokens2.len());
    for (t1, t2) in tokens1.iter().zip(tokens2.iter()) {
        assert_eq!(t1.kind, t2.kind);
    }
}

#[test]
fn test_tabs_and_spaces() {
    let input1 = "let\tx\t=\t1;";
    let input2 = "let x = 1;";
    let tokens1 = tokenize(input1).unwrap();
    let tokens2 = tokenize(input2).unwrap();
    assert_eq!(tokens1.len(), tokens2.len());
    for (t1, t2) in tokens1.iter().zip(tokens2.iter()) {
        assert_eq!(t1.kind, t2.kind);
    }
}
