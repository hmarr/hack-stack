use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use hack_stack::common;
use hack_stack::jack;

fn main() {
    if let Err(_) = compile_main() {
        std::process::exit(1);
    }
}

fn compile_main() -> Result<(), ()> {
    let args = std::env::args().collect::<Vec<String>>();
    let source_path = args.get(1).ok_or_else(|| {
        eprintln!("usage: jack-compile FILE");
    })?;

    let source = fs::read_to_string(source_path).map_err(|err| {
        eprintln!("reading {}: {}", source_path, err);
    })?;

    let source_file = common::SourceFile::new(source, source_path.to_owned());
    let tokenizer = jack::Tokenizer::new(&source_file.src);
    let mut parser = jack::Parser::new(tokenizer);
    let klass = match parser.parse() {
        Ok(klass) => klass,
        Err(err) => {
            display_span_errors(&source_file, &vec![err]);
            return Err(());
        }
    };

    let mut gen = jack::Codegen::new();
    let vm_code = match gen.generate(&klass) {
        Ok(output) => output,
        Err(errs) => {
            display_span_errors(&source_file, errs);
            return Err(());
        }
    };

    let output_path = source_path.replace(".jack", "") + ".vm";
    let mut out_file = File::create(Path::new(&output_path)).map_err(|err| {
        eprintln!("creating {}: {}", output_path, err);
    })?;
    out_file.write_all(vm_code.as_bytes()).map_err(|err| {
        eprintln!("writing to {}: {}", output_path, err);
    })?;

    println!(
        "Compiled {} successfully, wrote to {}",
        source_path, output_path
    );

    Ok(())
}

fn display_span_errors(source_file: &common::SourceFile, errs: &Vec<common::SpanError>) {
    for err in errs {
        let (line, col) = source_file.loc_for_byte_pos(err.span.start);
        eprintln!("line {}, char {}: {}", line, col, err.msg);
    }
    std::process::exit(1);
}
