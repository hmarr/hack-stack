pub mod ast;
pub mod codegen;
pub mod ir;
mod optimize;
pub mod parser;
pub mod tokenizer;
pub mod tokens;

pub use codegen::Codegen;
pub use parser::Parser;
pub use tokenizer::Tokenizer;

use crate::common::{SourceFile, SpanError};

pub fn translate(
    source_files: &[SourceFile],
    bootstrap: bool,
    dce: bool,
) -> Result<String, (&SourceFile, Vec<SpanError>)> {
    let mut program = ir::Program::new();
    for source_file in source_files {
        let tokenizer = Tokenizer::new(&source_file.src);
        let mut parser = Parser::new(tokenizer);
        let instructions = match parser.parse() {
            Ok(instructions) => instructions,
            Err(errs) => {
                return Err((source_file, errs));
            }
        };
        program.add_module(instructions, source_file);
    }

    if dce {
        program.mark_reachable_functions();
    }

    program.optimize();

    let mut gen = Codegen::new(bootstrap);

    // Call the Sys.init function using the vm command. Although there's nowhere to return
    // to, and there's not much use in saving the current stack frame, using `call` ensures
    // the stack pointer points to the right place for the test cases.
    let bootstrap_code = String::from("call Sys.init 0\nlabel bootstrap.halt\ngoto bootstrap.halt");
    let bootstrap_source_file = SourceFile::new(bootstrap_code, "$BOOTSTRAP".to_owned());
    if bootstrap {
        let instructions = vm_code_to_ir(&bootstrap_source_file).unwrap();
        gen.generate_from_ir(&bootstrap_source_file, "$BOOTSTRAP", &instructions)
            .unwrap();
    }

    for module in program.modules.values() {
        if let Err(errs) =
            gen.generate_from_ir(module.source_file, "modulePrelude", &module.instructions)
        {
            return Err((module.source_file, errs));
        }
    }

    for function in program.functions.values() {
        if program.reachable_functions.contains(function.name) || !dce {
            if let Err(errs) = gen.generate_from_function(function) {
                return Err((function.source_file, errs));
            }
        }
    }

    Ok(gen.finalize().unwrap())
}

fn vm_code_to_ir(file: &SourceFile) -> Result<Vec<ir::Instruction>, Vec<SpanError>> {
    let tokenizer = Tokenizer::new(&file.src);
    let mut parser = Parser::new(tokenizer);
    parser.parse().map(|instructions| {
        instructions
            .into_iter()
            .map(ir::Instruction::Vm)
            .collect::<Vec<_>>()
    })
}
