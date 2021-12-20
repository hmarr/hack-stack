use hack_stack::{asm, common::SourceFile, emulator, jack, vm};

#[test]
fn test_simple_expression() {
    let src = r#"
    class Sys {
      function int init() {
        return 2 + 3 & 6 - 2 - -1;
      }
    }
    "#;

    let ram = compile_and_evaluate(src, 1000);
    assert_eq!(ram[0], 257);
    assert_eq!(ram[256], 3);
}

#[test]
fn test_locals() {
    let src = r#"
    class Sys {
      function int init() {
        var int x, y, z;
        let x = 1;
        let y = 2;
        let z = 3;
        return x + y + z;
      }
    }
    "#;

    let ram = compile_and_evaluate(src, 10000);
    assert_eq!(ram[0], 257);
    assert_eq!(ram[256], 6);
}

#[test]
fn test_while() {
    let src = r#"
    class Sys {
      function int init() {
        var int x, y;
        let x = 1;
        let y = 0;
        while (x < 4) {
          let y = y + x;
          let x = x + 1;
        }
        return y;
      }
    }
    "#;

    let ram = compile_and_evaluate(src, 1000);
    assert_eq!(ram[0], 257);
    assert_eq!(ram[256], 6);
}

#[test]
fn test_if() {
    let src = r#"
    class Sys {
      function int init() {
        var int x;
        let x = 0;

        if (true) {
          let x = x + 2;
        } else {
          let x = x + 3;
        }

        if (false) {
            let x = x + 5;
        } else {
            let x = x + 7;
        }

        return x;
      }
    }
    "#;

    let ram = compile_and_evaluate(src, 1000);
    assert_eq!(ram[0], 257);
    assert_eq!(ram[256], 9);
}

#[test]
fn test_function_call() {
    let src = r#"
    class Sys {
      function int init() {
        return Sys.foo() + Sys.bar(2);
      }

      function int foo() {
        return 1;
      }

      function int bar(int x) {
        return x + 2;
      }
    }
    "#;

    let ram = compile_and_evaluate(src, 1000);
    assert_eq!(ram[0], 257);
    assert_eq!(ram[256], 5);
}

#[test]
fn test_this() {
    let src = r#"
    class Sys {
      function void init() {
        var s Sys;
        let s = Sys.new();
        return s = s.self();
      }

      constructor Sys new() {
        return this;
      }

      method Sys self() {
        return this;
      }
    }
    "#;

    let ram = compile_and_evaluate(src, 1000);
    assert_eq!(ram[0], 257);
    assert_eq!(ram[256], 0xffff);
}

fn compile_and_evaluate(jack_src: &str, steps: usize) -> Vec<u16> {
    let source_file = SourceFile::new(compile(jack_src), "Sys.jack".into());
    let asm_src = vm::translate(&[source_file], true).unwrap();
    let hack_src = assemble(&asm_src);
    let mut emu = emulator::Emulator::new(parse_rom(&hack_src));

    for _ in 0..steps {
        emu.step().unwrap();
    }

    emu.memory().to_owned()
}

fn compile(jack_src: &str) -> String {
    let class_node = jack::Parser::new(jack::Tokenizer::new(jack_src))
        .parse()
        .unwrap();
    jack::Codegen::new().generate(&class_node).unwrap().into()
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
