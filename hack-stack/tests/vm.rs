use std::path::PathBuf;

use hack_stack::{asm, common, emulator, vm};

#[test]
fn test_simple_add() {
    let source_files = &[load_fixture("SimpleAdd.vm")];
    let asm_src = vm::translate(source_files, false).unwrap();
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
    let source_files = &[load_fixture("BasicTest.vm")];
    let asm_src = vm::translate(source_files, false).unwrap();
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
    let source_files = &[load_fixture("PointerTest.vm")];
    let asm_src = vm::translate(source_files, false).unwrap();
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
    let source_files = &[load_fixture("StaticTest.vm")];
    let asm_src = vm::translate(source_files, false).unwrap();
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
    let source_files = &[load_fixture("StackTest.vm")];
    let asm_src = vm::translate(source_files, false).unwrap();
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

#[test]
fn test_basic_loop() {
    let source_files = &[load_fixture("BasicLoop.vm")];
    let asm_src = vm::translate(source_files, false).unwrap();
    let hack_src = assemble(&asm_src);
    let mut emu = emulator::Emulator::new(parse_rom(&hack_src));

    emu.set_memory(0, 256).unwrap();
    emu.set_memory(1, 300).unwrap();
    emu.set_memory(2, 400).unwrap();
    emu.set_memory(400, 3).unwrap();

    for _ in 0..600 {
        emu.step().unwrap();
    }

    let ram = emu.memory();
    assert_eq!(ram[0], 257);
    assert_eq!(ram[256], 6);
}

#[test]
fn test_fibonacci_series() {
    let source_files = &[load_fixture("FibonacciSeries.vm")];
    let asm_src = vm::translate(source_files, false).unwrap();
    let hack_src = assemble(&asm_src);
    let mut emu = emulator::Emulator::new(parse_rom(&hack_src));

    emu.set_memory(0, 256).unwrap();
    emu.set_memory(1, 300).unwrap();
    emu.set_memory(2, 400).unwrap();
    emu.set_memory(400, 6).unwrap();
    emu.set_memory(401, 3000).unwrap();

    for _ in 0..1100 {
        emu.step().unwrap();
    }

    let ram = emu.memory();
    assert_eq!(ram[3000], 0);
    assert_eq!(ram[3001], 1);
    assert_eq!(ram[3002], 1);
    assert_eq!(ram[3003], 2);
    assert_eq!(ram[3004], 3);
    assert_eq!(ram[3005], 5);
}

#[test]
fn test_simple_function() {
    let source_files = &[load_fixture("SimpleFunction.vm")];
    let asm_src = vm::translate(source_files, false).unwrap();
    let hack_src = assemble(&asm_src);
    let mut emu = emulator::Emulator::new(parse_rom(&hack_src));

    emu.set_memory(0, 317).unwrap();
    emu.set_memory(1, 317).unwrap();
    emu.set_memory(2, 310).unwrap();
    emu.set_memory(3, 3000).unwrap();
    emu.set_memory(4, 4000).unwrap();
    emu.set_memory(310, 1234).unwrap();
    emu.set_memory(311, 37).unwrap();
    emu.set_memory(312, 1000).unwrap();
    emu.set_memory(313, 305).unwrap();
    emu.set_memory(314, 300).unwrap();
    emu.set_memory(315, 3010).unwrap();
    emu.set_memory(316, 4010).unwrap();

    for _ in 0..300 {
        emu.step().unwrap();
    }

    let ram = emu.memory();
    assert_eq!(ram[0], 311);
    assert_eq!(ram[1], 305);
    assert_eq!(ram[2], 300);
    assert_eq!(ram[3], 3010);
    assert_eq!(ram[4], 4010);
    assert_eq!(ram[310], 1196);
}

#[test]
fn test_nested_call() {
    let source_files = &[load_fixture("NestedCall/Sys.vm")];
    let asm_src = vm::translate(source_files, false).unwrap();
    let hack_src = assemble(&asm_src);
    let mut emu = emulator::Emulator::new(parse_rom(&hack_src));

    emu.set_memory(0, 261).unwrap();
    emu.set_memory(1, 261).unwrap();
    emu.set_memory(2, 256).unwrap();
    emu.set_memory(3, 0u16.wrapping_sub(3)).unwrap();
    emu.set_memory(4, 0u16.wrapping_sub(4)).unwrap();
    emu.set_memory(5, 0u16.wrapping_sub(1)).unwrap();
    emu.set_memory(6, 0u16.wrapping_sub(1)).unwrap();
    emu.set_memory(256, 1234).unwrap();
    emu.set_memory(257, 0u16.wrapping_sub(1)).unwrap();
    emu.set_memory(258, 0u16.wrapping_sub(2)).unwrap();
    emu.set_memory(259, 0u16.wrapping_sub(3)).unwrap();
    emu.set_memory(260, 0u16.wrapping_sub(4)).unwrap();
    for addr in 261..=299 {
        emu.set_memory(addr, 0u16.wrapping_sub(1)).unwrap();
    }

    for _ in 0..4000 {
        emu.step().unwrap();
    }

    let ram = emu.memory();
    assert_eq!(ram[0], 261);
    assert_eq!(ram[1], 261);
    assert_eq!(ram[2], 256);
    assert_eq!(ram[3], 4000);
    assert_eq!(ram[4], 5000);
    assert_eq!(ram[5], 135);
    assert_eq!(ram[6], 246);
}

#[test]
fn test_fibonacci_element() {
    let source_files = &[
        load_fixture("FibonacciElement/Main.vm"),
        load_fixture("FibonacciElement/Sys.vm"),
    ];
    let asm_src = vm::translate(source_files, true).unwrap();
    let hack_src = assemble(&asm_src);
    let mut emu = emulator::Emulator::new(parse_rom(&hack_src));

    for _ in 0..6000 {
        emu.step().unwrap();
    }

    let ram = emu.memory();
    assert_eq!(ram[0], 262);
    assert_eq!(ram[261], 3);
}

#[test]
fn test_statics_test() {
    let source_files = &[
        load_fixture("StaticsTest/Sys.vm"),
        load_fixture("StaticsTest/Class1.vm"),
        load_fixture("StaticsTest/Class2.vm"),
    ];
    let asm_src = vm::translate(source_files, true).unwrap();
    let hack_src = assemble(&asm_src);
    let mut emu = emulator::Emulator::new(parse_rom(&hack_src));

    emu.set_memory(0, 256).unwrap();

    for _ in 0..2500 {
        emu.step().unwrap();
    }

    let ram = emu.memory();
    assert_eq!(ram[0], 263);
    assert_eq!(ram[261], 0u16.wrapping_sub(2));
    assert_eq!(ram[262], 8);
}

fn assemble(asm_src: &str) -> String {
    let mut parser = asm::Parser::new(asm::Tokenizer::new(asm_src));
    let mut cg = asm::Codegen::new();
    cg.generate(&parser.parse().unwrap()).unwrap()
}

fn parse_rom(hack_src: &str) -> Vec<u16> {
    let mut rom = vec![0u16; 0x2000];
    for (i, line) in hack_src.lines().enumerate() {
        rom[i] = u16::from_str_radix(line.trim_end(), 2).unwrap();
    }
    rom
}

fn load_fixture(name: &str) -> common::SourceFile {
    let root_dir: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_dir = root_dir.join("tests").join("fixtures").join("vm");
    let src = std::fs::read_to_string(fixture_dir.join(name)).unwrap();
    common::SourceFile::new(src, name.to_owned())
}
