use std::{
    ffi::OsStr,
    fs::{self, File},
    io::Write,
    path::Path,
};

use hack_stack::common;
use hack_stack::vm;

fn main() {
    if translate_main().is_err() {
        std::process::exit(1);
    }
}

fn translate_main() -> Result<(), ()> {
    let args = std::env::args().collect::<Vec<String>>();
    let path_arg = args.get(1).ok_or_else(|| {
        eprintln!("usage: hack-vm-translate PATH");
    })?;
    let source_path = Path::new(path_arg).canonicalize().map_err(|err| {
        eprintln!("reading path {}: {}", path_arg, err);
    })?;
    let source_path_str = source_path.to_str().unwrap().to_owned();

    let (bootstrap, source_paths) = if source_path.is_dir() {
        let files = fs::read_dir(&source_path).map_err(|err| {
            eprintln!("listing directory {}: {}", source_path_str, err);
        })?;
        let paths = files
            .filter_map(|r| r.ok())
            .map(|f| f.path())
            .filter(|f| f.extension() == Some(OsStr::new("vm")))
            .map(|p| p.to_str().unwrap().to_owned())
            .collect::<Vec<String>>();
        (true, paths)
    } else {
        (false, vec![source_path_str.clone()])
    };

    let mut source_files: Vec<common::SourceFile> = vec![];
    for source_path in source_paths {
        let source = fs::read_to_string(&source_path).map_err(|err| {
            eprintln!("reading {}: {}", source_path, err);
        })?;

        let source_file_name = Path::new(&source_path)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap();
        source_files.push(common::SourceFile::new(source, source_file_name.to_owned()));
    }

    let output_asm =
        vm::translate(&source_files, bootstrap, bootstrap).map_err(|(file, errs)| {
            display_span_errors(file, errs);
        })?;

    let output_path = if source_path.is_dir() {
        let dir_name = source_path.file_name().unwrap().to_str().unwrap();
        source_path
            .join(dir_name.to_owned() + ".asm")
            .to_str()
            .unwrap()
            .to_owned()
    } else {
        source_path_str.replace(".vm", "") + ".asm"
    };
    let mut out_file = File::create(Path::new(&output_path)).map_err(|err| {
        eprintln!("creating {}: {}", output_path, err);
    })?;
    out_file.write_all(output_asm.as_bytes()).map_err(|err| {
        eprintln!("writing to {}: {}", output_path, err);
    })?;

    println!(
        "Translated {} successfully, wrote to {}",
        source_path_str, output_path
    );

    Ok(())
}

fn display_span_errors(source_file: &common::SourceFile, errs: Vec<common::SpanError>) {
    for err in errs {
        let (line, col) = source_file.loc_for_byte_pos(err.span.start);
        eprintln!(
            "{} (line {}, char {}): {}",
            source_file.name, line, col, err.msg
        );
    }
    std::process::exit(1);
}
