mod data;
mod logic;
mod error;

use std::process::Command;
use logic::{Token, Tokens, lexer_start, parser_start, semantic_analyze, SemanticContext};

use crate::data::SymbolTable;
use crate::data::TokenType;

const INPUT_CODE: &str = "./input/test.c";

fn main() {
    let preproces_source = match preproces_source(INPUT_CODE) {
        Ok(src) => src,
        Err(e) => {
            eprintln!("Preprocessing failed: {}", e);
            return;
        }
    };

    let mut tokens = match lexer_start(&preproces_source) {
        Ok(tokens) => Tokens { tokens },
        Err(e) => {
            eprintln!("\nLexer Error: {}\n", e);
            return;
        }
    };
    
    let ast = match parser_start(&mut tokens, 0) {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("\nParser Error: {}\n", e);
            return;
        }
    };

    let mut sem_ctx = SemanticContext {
        fn_table: SymbolTable::new(),
        var_table: SymbolTable::new(),
        scope: vec!["global".to_string()],
    };

    if let Err(e) = semantic_analyze(&ast, &mut sem_ctx) {
        eprintln!("\nSemantic Error: {}\n", e);
        return;
    }

    /* - DEGUB - */
    let json = serde_json::to_string_pretty(&ast).unwrap();
    println!("{}\n\n\n", json);
}


fn preproces_source(file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("cpp")
        .arg(file_path)
        .output()?;

    if !output.status.success() {
        return Err(format!("Preprocessor failed with code {:?}", output.status.code()).into());
    }

    Ok(String::from_utf8(output.stdout)?)
}