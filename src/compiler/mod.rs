use anyhow::{Result, Context};
use std::fs;

pub mod lexer;
pub mod parser;
pub mod codegen;

pub async fn compile(input_path: &str, output_path: &str) -> Result<()> {
    let source = fs::read_to_string(input_path)
        .with_context(|| format!("Failed to read source file: {}", input_path))?;
    
    println!("  [1/3] Lexical analysis...");
    let tokens = lexer::tokenize(&source)?;
    
    println!("  [2/3] Parsing...");
    let ast = parser::parse(tokens)?;
    
    println!("  [3/3] Code generation...");
    let bytecode = codegen::generate(ast)?;
    
    fs::write(output_path, bytecode)
        .with_context(|| format!("Failed to write output file: {}", output_path))?;
    
    Ok(())
}
