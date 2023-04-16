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
fn test_statics() {
    let sys_src = r#"
    class Sys {
      function void init() {
        do Foo.add(1);
        do Foo.add(2);
        return Foo.value() = 3;
      }
    }
    "#;
    let foo_src = r#"
    class Foo {
      static int sum;
      function void add(int x) {
        let sum = sum + x;
        return;
      }
      function int value() {
        return sum;
      }
    }
    "#;

    let sys_vm_src = SourceFile::new(compile(sys_src), "Sys.jack".into());
    let foo_vm_src = SourceFile::new(compile(foo_src), "Foo.jack".into());
    let ram = eval(&[sys_vm_src, foo_vm_src], 5000);
    assert_eq!(ram[0], 257);
    assert_eq!(ram[256], 0xffff);
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
fn test_methods_and_fields() {
    let sys_src = r#"
    class Sys {
      function void init() {
        var Counter c1, c2;
        let c1 = Counter.new(1);
        let c2 = Counter.new(2);
        do c1.addOne();
        do c2.addOne();
        do c1.addOther(c2);
        return c1.count() = 5;
      }
    }
    "#;
    let counter_src = r#"
    class Counter {
      field int count;
      constructor Counter new(int initial) {
        let count = initial;
        return this;
      }
      method void addOne() {
        let count = count + 1;
        return;
      }
      method void addOther(Counter other) {
        let count = count + other.count();
        return;
      }
      method int count() {
        return count;
      }
    }
    "#;
    let sys_vm_src = SourceFile::new(compile(sys_src), "Sys.jack".into());
    let counter_vm_src = SourceFile::new(compile(counter_src), "Counter.jack".into());

    let ram = eval(&[sys_vm_src, counter_vm_src, malloc_vm_src()], 5000);
    assert_eq!(ram[0], 257);
    assert_eq!(ram[256], 0xffff);
}

#[test]
fn test_this() {
    let prog_src = r#"
    class Sys {
      function void init() {
        var Sys s;
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
    let prog_vm_src = SourceFile::new(compile(prog_src), "Sys.jack".into());

    let ram = eval(&[prog_vm_src, malloc_vm_src()], 5000);
    assert_eq!(ram[0], 257);
    assert_eq!(ram[256], 0xffff);
}

#[test]
fn test_arrays() {
    let prog_src = r#"
    class Sys {
      function void init() {
        var Array xs, ys;
        let xs = Memory.alloc(2);
        let ys = Memory.alloc(2);
        let xs[0] = 4;
        let xs[1] = 5;
        let ys[0] = xs[1];
        let ys[1 - 0] = xs[xs[1] - xs[0]];
        return ys[1] = 5;
      }
    }
    "#;
    let prog_vm_src = SourceFile::new(compile(prog_src), "Sys.jack".into());
    let ram = eval(&[prog_vm_src, malloc_vm_src()], 5000);
    assert_eq!(ram[0], 257);
    assert_eq!(ram[256], 0xffff);
}

#[test]
fn test_strings() {
    let prog_src = r#"
    class Sys {
      function void init() {
        var String s;
        let s = "Hello, All!";
        return s.charAt(7) = 65;
      }
    }
    "#;
    let string_src = r#"
    class String {
      field Array buf, len;
      constructor String new(int capacity) {
        let buf = Memory.alloc(capacity);
        let len = 0;
        return this;
      }
      method String appendChar(char c) {
        let buf[len] = c;
        let len = len + 1;
        return this;
      }
      method int charAt(int pos) {
        return buf[pos];
      }
    }
    "#;
    let prog_vm_src = SourceFile::new(compile(prog_src), "Sys.jack".into());
    let string_vm_src = SourceFile::new(compile(string_src), "String.jack".into());
    let ram = eval(&[prog_vm_src, string_vm_src, malloc_vm_src()], 5000);
    assert_eq!(ram[0], 257);
    assert_eq!(ram[256], 0xffff);
}

// Very primitive bump allocator that never frees. Good enough to test object construction.
fn malloc_vm_src() -> SourceFile {
    let mem_src = r#"
    class Memory {
      static int next;
      function void alloc(int n) {
        var int ptr;
        if (next < 2048) {
          let next = 2048;
        }
        let ptr = next;
        let next = next + n;
        return ptr;
      }
    }
    "#;
    SourceFile::new(compile(mem_src), "Memory.jack".into())
}

fn compile_and_evaluate(jack_src: &str, steps: usize) -> Vec<u16> {
    let prog_vm_src = SourceFile::new(compile(jack_src), "Sys.jack".into());
    eval(&[prog_vm_src], steps)
}

fn compile(jack_src: &str) -> String {
    let class = jack::Parser::new(jack::Tokenizer::new(jack_src))
        .parse()
        .unwrap();
    jack::Codegen::new(&class).generate().unwrap().into()
}

fn assemble(asm_src: &str) -> String {
    let mut parser = asm::Parser::new(asm::Tokenizer::new(asm_src));
    let mut cg = asm::Codegen::new();
    cg.generate(&parser.parse().unwrap()).unwrap()
}

fn eval(vm_src_files: &[SourceFile], steps: usize) -> Vec<u16> {
    let asm_src = vm::translate(vm_src_files, true, true).unwrap();
    println!("{}", asm_src);
    let hack_src = assemble(&asm_src);
    let mut emu = emulator::Emulator::new(parse_rom(&hack_src));

    for _ in 0..steps {
        emu.step().unwrap();
    }

    emu.memory().to_owned()
}

fn parse_rom(hack_src: &str) -> Vec<u16> {
    let mut rom = vec![0u16; 0x2000];
    for (i, line) in hack_src.lines().enumerate() {
        rom[i] = u16::from_str_radix(line.trim_end(), 2).unwrap();
    }
    rom
}
