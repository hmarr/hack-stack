use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use hack_stack::asm;
use hack_stack::common::SpanError;

fn main() {
    if let Err(_) = assemble_main() {
        std::process::exit(1);
    }
}

fn assemble_main() -> Result<(), ()> {
    let args = std::env::args().collect::<Vec<String>>();
    let source_path = args.get(1).ok_or_else(|| {
        eprintln!("usage: hack-assemble FILE");
    })?;

    let source = fs::read_to_string(source_path).map_err(|err| {
        eprintln!("reading {}: {}", source_path, err);
    })?;

    let tokenizer = asm::Tokenizer::new(&source);
    let mut parser = asm::Parser::new(tokenizer);
    let instructions = match parser.parse() {
        Ok(instructions) => instructions,
        Err(errs) => {
            display_span_errors(&parser, errs);
            return Err(());
        }
    };

    let mut gen = asm::Codegen::new();
    let machine_code = match gen.generate(&instructions) {
        Ok(output) => output,
        Err(errs) => {
            display_span_errors(&parser, errs);
            return Err(());
        }
    };

    let output_path = source_path.replace(".asm", "") + ".hack";
    let mut out_file = File::create(Path::new(&output_path)).map_err(|err| {
        eprintln!("creating {}: {}", output_path, err);
    })?;
    out_file.write_all(machine_code.as_bytes()).map_err(|err| {
        eprintln!("writing to {}: {}", output_path, err);
    })?;

    println!(
        "Assembled {} successfully, wrote to {}",
        source_path, output_path
    );

    Ok(())
}

fn display_span_errors(parser: &asm::Parser, errs: Vec<SpanError>) {
    for err in errs {
        let (line, col) = parser.error_loc(&err);
        eprintln!("line {}, char {}: {}", line, col, err.msg);
    }
    std::process::exit(1);
}
