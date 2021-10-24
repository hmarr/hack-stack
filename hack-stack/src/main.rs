use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use errors::SpanError;
use parse::Parser;

mod codegen;
mod errors;
mod parse;
mod tokenize;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<String>>();
    let source_path = args.get(1).unwrap_or_else(|| {
        eprintln!("usage: hack-as FILE");
        std::process::exit(1);
    });

    let source = fs::read_to_string(source_path).unwrap();
    let tokenizer = tokenize::Tokenizer::new(&source);
    let mut parser = parse::Parser::new(tokenizer);
    match parser.parse() {
        Ok(ast) => {
            let output_path = source_path.replace(".asm", "") + ".hack";
            let mut gen = codegen::Codegen::new();
            match gen.generate(&ast) {
                Ok(output) => {
                    let mut out_file = File::create(Path::new(&output_path))?;
                    out_file.write_all(output.as_bytes())?;
                    println!(
                        "Assembled {} successfully, wrote to {}",
                        source_path, output_path
                    )
                }
                Err(errs) => abort_with_errors(&parser, errs),
            }
        }
        Err(errs) => abort_with_errors(&parser, errs),
    }
    Ok(())
}

fn abort_with_errors(parser: &Parser, errs: Vec<SpanError>) {
    for err in errs {
        let (line, col) = parser.error_loc(&err);
        eprintln!("line {}, char {}: {}", line, col, err.msg);
    }
    std::process::exit(1);
}
