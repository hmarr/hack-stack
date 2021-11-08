pub mod ast;
pub mod codegen;
pub mod parser;
pub mod tokenizer;
pub mod tokens;

pub use codegen::Codegen;
pub use parser::Parser;
pub use tokenizer::Tokenizer;

use crate::common::{SourceFile, SpanError};

pub fn translate<'a>(
    source_files: &'a [SourceFile],
    bootstrap: bool,
) -> Result<String, (&'a SourceFile, Vec<SpanError>)> {
    let mut buf = String::new();

    if bootstrap {
        // SP=256
        buf.push_str("@256\nD=A\n@0\nM=D");

        // Call the Sys.init function using the vm command. Although there's nowhere to return
        // to, and there's not much use in saving the current stack frame, using `call` ensures
        // the stack pointer points to the right place for the test cases.
        let bootstrap_code = String::from("call Sys.init 0");
        let source_file = SourceFile::new(bootstrap_code, "$BOOTSTRAP".to_owned());
        translate_file(&mut buf, &source_file).unwrap();
    }

    for file in source_files {
        translate_file(&mut buf, file)?;
    }

    Ok(buf)
}

fn translate_file<'a>(
    buf: &mut String,
    file: &'a SourceFile,
) -> Result<(), (&'a SourceFile, Vec<SpanError>)> {
    let tokenizer = Tokenizer::new(&file.src);
    let mut parser = Parser::new(tokenizer);
    let instructions = match parser.parse() {
        Ok(instructions) => instructions,
        Err(errs) => {
            return Err((file, errs));
        }
    };

    let mut gen = Codegen::new(file);
    let assembly = match gen.generate(&instructions) {
        Ok(output) => output,
        Err(errs) => {
            return Err((file, errs));
        }
    };
    buf.push_str(&format!("// File: {}\n\n", file.name));
    buf.push_str(assembly);
    buf.push_str("\n\n");
    Ok(())
}
