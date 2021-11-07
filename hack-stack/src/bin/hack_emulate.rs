use std::{fs, io::stdin};

use hack_stack::emulator;

fn main() {
    if let Err(_) = emulate_main() {
        std::process::exit(1);
    }
}

fn emulate_main() -> Result<(), ()> {
    let args_and_opts = std::env::args().collect::<Vec<String>>();
    let (opts, args): (Vec<&String>, Vec<&String>) = args_and_opts
        .iter()
        .skip(1)
        .partition(|&a| a.starts_with("--"));

    let source_path = args.get(0).ok_or_else(|| {
        eprintln!("usage: hack-emulate FILE");
    })?;

    let trace = opts.iter().any(|o| *o == "--trace");

    let source = fs::read_to_string(source_path).map_err(|err| {
        eprintln!("reading {}: {}", source_path, err);
    })?;

    let mut rom = vec![0u16; 0x2000];
    for (i, line) in source.lines().enumerate() {
        rom[i] = u16::from_str_radix(line.trim_end(), 2).unwrap();
    }

    let mut emulator = emulator::Emulator::new(rom);
    if trace {
        println!("|     D |     A |    PC | Memory");
    }
    for _ in 0..20000000 {
        if trace {
            let memory = emulator.memory()[0..16]
                .iter()
                .map(|x: &u16| format!("{:04X}", x))
                .collect::<Vec<String>>()
                .join(" ");
            println!(
                "| {:5} | {:5} | {:5} | {} |",
                emulator.cpu.d, emulator.cpu.a, emulator.cpu.pc, memory
            );
        }

        emulator.step().map_err(|err| {
            eprintln!("emulator error: {}", err);
        })?;

        let mut buf = String::new();
        if stdin().read_line(&mut buf).expect("reading line") == 0 {
            break;
        }
    }

    Ok(())
}
