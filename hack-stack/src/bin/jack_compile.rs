use std::{
    ffi::OsStr,
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
        eprintln!("usage: jack-compile PATH");
    })?;

    if Path::new(source_path).is_dir() {
        let files = fs::read_dir(source_path).map_err(|err| {
            eprintln!("listing directory {}: {}", source_path, err);
        })?;

        files
            .filter_map(|r| r.ok())
            .map(|f| f.path())
            .filter(|f| f.extension() == Some(OsStr::new("jack")))
            .map(|p| p.to_str().unwrap().to_owned())
            .try_for_each(|path| compile_file(&path))?;
    } else {
        compile_file(source_path)?;
    }

    Ok(())
}

fn compile_file(source_path: &String) -> Result<(), ()> {
    let source = fs::read_to_string(source_path).map_err(|err| {
        eprintln!("reading {}: {}", source_path, err);
    })?;
    let source_file = common::SourceFile::new(source, source_path.to_owned());
    let tokenizer = jack::Tokenizer::new(&source_file.src);
    let mut parser = jack::Parser::new(tokenizer);
    let class = match parser.parse() {
        Ok(class) => class,
        Err(err) => {
            display_span_errors(&source_file, &vec![err]);
            return Err(());
        }
    };
    let mut gen = jack::Codegen::new(&class);
    let vm_code = match gen.generate() {
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
