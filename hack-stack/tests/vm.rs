use std::path::PathBuf;

use hack_stack::{asm, common, emulator, vm};

#[test]
fn test_simple_add() {
    let asm_src = vm_to_asm(&load_fixture("SimpleAdd.vm"));
    let hack_src = assemble(&asm_src);
    let mut emu = emulator::Emulator::new(parse_rom(&hack_src));

    emu.set_memory(0, 256).unwrap(); // SP

    for _ in 0..60 {
        emu.step().unwrap();
    }

    let ram = emu.memory();
    assert_eq!(ram[0], 257);
    assert_eq!(ram[256], 15);
}

#[test]
fn test_basic_test() {
    let asm_src = vm_to_asm(&load_fixture("BasicTest.vm"));
    let hack_src = assemble(&asm_src);
    let mut emu = emulator::Emulator::new(parse_rom(&hack_src));

    emu.set_memory(0, 256).unwrap(); // SP
    emu.set_memory(1, 300).unwrap(); // LOCAL
    emu.set_memory(2, 400).unwrap(); // ARGUMENT
    emu.set_memory(3, 3000).unwrap(); // THIS
    emu.set_memory(4, 3010).unwrap(); // THAT

    for _ in 0..600 {
        emu.step().unwrap();
    }

    let ram = emu.memory();
    assert_eq!(ram[256], 472);
    assert_eq!(ram[300], 10);
    assert_eq!(ram[401], 21);
    assert_eq!(ram[402], 22);
    assert_eq!(ram[3006], 36);
    assert_eq!(ram[3012], 42);
    assert_eq!(ram[3015], 45);
    assert_eq!(ram[11], 510);
}

#[test]
fn test_pointer_test() {
    let asm_src = vm_to_asm(&load_fixture("PointerTest.vm"));
    let hack_src = assemble(&asm_src);
    let mut emu = emulator::Emulator::new(parse_rom(&hack_src));

    emu.set_memory(0, 256).unwrap(); // SP

    for _ in 0..450 {
        emu.step().unwrap();
    }

    let ram = emu.memory();
    assert_eq!(ram[256], 6084);
    assert_eq!(ram[3], 3030);
    assert_eq!(ram[4], 3040);
    assert_eq!(ram[3032], 32);
    assert_eq!(ram[3046], 46);
}

#[test]
fn test_static_test() {
    let asm_src = vm_to_asm(&load_fixture("StaticTest.vm"));
    let hack_src = assemble(&asm_src);
    let mut emu = emulator::Emulator::new(parse_rom(&hack_src));

    emu.set_memory(0, 256).unwrap(); // SP

    for _ in 0..200 {
        emu.step().unwrap();
    }

    let ram = emu.memory();
    assert_eq!(ram[256], 1110);
}

#[test]
fn test_stack_test() {
    let asm_src = vm_to_asm(&load_fixture("StackTest.vm"));
    let hack_src = assemble(&asm_src);
    let mut emu = emulator::Emulator::new(parse_rom(&hack_src));

    emu.set_memory(0, 256).unwrap(); // SP

    for _ in 0..1000 {
        emu.step().unwrap();
    }

    let ram = emu.memory();
    assert_eq!(ram[0], 266);
    assert_eq!(ram[256], 0u16.wrapping_sub(1));
    assert_eq!(ram[257], 0);
    assert_eq!(ram[258], 0);
    assert_eq!(ram[259], 0);
    assert_eq!(ram[260], 0u16.wrapping_sub(1));
    assert_eq!(ram[261], 0);
    assert_eq!(ram[262], 0u16.wrapping_sub(1));
    assert_eq!(ram[263], 0);
    assert_eq!(ram[264], 0);
    assert_eq!(ram[265], 0u16.wrapping_sub(91));
}

fn vm_to_asm(src: &str) -> String {
    let source_file = common::SourceFile::new(src, "Test.vm");
    let tokenizer = vm::Tokenizer::new(src);
    let mut parser = vm::Parser::new(tokenizer);
    let instructions = parser.parse().unwrap();
    let mut codegen = vm::Codegen::new(&source_file);
    codegen.generate(&instructions).unwrap().to_owned()
}

fn assemble(asm_src: &str) -> String {
    let mut parser = asm::Parser::new(asm::Tokenizer::new(asm_src));
    let mut cg = asm::Codegen::new();
    cg.generate(&parser.parse().unwrap()).unwrap()
}

fn parse_rom(hack_src: &str) -> Vec<u16> {
    let mut rom = Vec::<u16>::with_capacity(0x2000);
    for line in hack_src.lines() {
        rom.push(u16::from_str_radix(line.trim_end(), 2).unwrap());
    }
    rom
}

fn load_fixture(name: &str) -> String {
    let root_dir: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_dir = root_dir.join("tests").join("fixtures").join("vm");
    std::fs::read_to_string(fixture_dir.join(name)).unwrap()
}
