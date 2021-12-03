use std::{
    ffi::OsStr,
    fs::{self, File},
    io::Write,
    path::Path,
};

use hack_stack::jack::{self, debugxml::write_tree};
use hack_stack::{common, jack::debugxml};

fn main() {
    if let Err(_) = translate_main() {
        std::process::exit(1);
    }
}

fn translate_main() -> Result<(), ()> {
    let args = std::env::args().collect::<Vec<String>>();
    let path_arg = args.get(1).ok_or_else(|| {
        eprintln!("usage: hack-analyze PATH");
    })?;
    let source_path = Path::new(path_arg).canonicalize().map_err(|err| {
        eprintln!("reading path {}: {}", path_arg, err);
    })?;
    let source_path_str = source_path.to_str().unwrap().to_owned();

    let source_paths = if source_path.is_dir() {
        let files = fs::read_dir(&source_path).map_err(|err| {
            eprintln!("listing directory {}: {}", source_path_str, err);
        })?;
        files
            .filter_map(|r| r.ok())
            .map(|f| f.path())
            .filter(|f| f.extension() == Some(OsStr::new("jack")))
            .map(|p| p.to_str().unwrap().to_owned())
            .collect::<Vec<String>>()
    } else {
        vec![source_path_str.clone()]
    };

    for source_path in source_paths {
        let source = fs::read_to_string(&source_path).map_err(|err| {
            eprintln!("reading {}: {}", source_path, err);
        })?;

        let source_file_name = Path::new(&source_path)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap();
        let source_file = common::SourceFile::new(source, source_file_name.to_owned());

        println!("Writing token and parse tree files for {}", &source_path);
        write_tokens(&source_path, &source_file)?;
        write_parse_tree(&source_path, &source_file)?;
    }

    Ok(())
}

fn write_tokens(source_file_path: &String, source_file: &common::SourceFile) -> Result<(), ()> {
    let output_path = source_file_path.replace(".jack", "") + "T.xml";
    let mut out_file = File::create(Path::new(&output_path)).map_err(|err| {
        eprintln!("creating {}: {}", output_path, err);
    })?;

    writeln!(out_file, "<tokens>").unwrap();
    let mut tokenizer = jack::tokenizer::Tokenizer::new(&source_file.src);
    let mut token = tokenizer.next_token();
    while token.kind != jack::tokens::Kind::EOF {
        debugxml::write_token(&mut out_file, &token);
        token = tokenizer.next_token();
    }
    writeln!(out_file, "</tokens>").unwrap();
    Ok(())
}

fn write_parse_tree(source_file_path: &String, source_file: &common::SourceFile) -> Result<(), ()> {
    let output_path = source_file_path.replace(".jack", "") + ".xml";
    let mut out_file = File::create(Path::new(&output_path)).map_err(|err| {
        eprintln!("creating {}: {}", output_path, err);
    })?;

    let tokenizer = jack::tokenizer::Tokenizer::new(&source_file.src);
    let mut parser = jack::parser::Parser::new(tokenizer);
    let tree = parser.parse().map_err(|err| {
        display_span_errors(&source_file, &[err]);
    })?;

    write_tree(&mut out_file, &tree, 0);

    Ok(())
}

fn display_span_errors(source_file: &common::SourceFile, errs: &[common::SpanError]) {
    for err in errs {
        let (line, col) = source_file.loc_for_byte_pos(err.span.start);
        eprintln!(
            "{} (line {}, char {}): {}",
            source_file.name, line, col, err.msg
        );
    }
    std::process::exit(1);
}
