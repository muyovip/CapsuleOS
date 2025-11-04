use glyph_lexer::*;

fn main() {
    let src = "3.14";
    let tokens = tokenize(src).unwrap();
    for t in &tokens {
        println!("{:?}", t.kind);
    }
    
    println!("\n---\n");
    
    let src2 = "/* comment */";
    match tokenize(src2) {
        Ok(tokens) => {
            for t in &tokens {
                println!("{:?}", t.kind);
            }
        }
        Err(e) => println!("Error: {}", e),
    }
}
