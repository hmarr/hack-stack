#[derive(Debug)]
pub struct Cpu {
    pub d: u16,
    pub a: u16,
    pub m: u16,
    pub pc: u16,
    pub write_m: bool,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            d: 0,
            a: 0,
            m: 0,
            pc: 0,
            write_m: false,
        }
    }

    pub fn reset(&mut self) {
        self.d = 0;
        self.a = 0;
        self.m = 0;
        self.pc = 0;
    }

    pub fn execute(&mut self, instruction: u16) -> Result<(), String> {
        self.write_m = false;

        if instruction & 0x8000 == 0 {
            self.execute_a_instruction(instruction);
        } else {
            self.execute_c_instruction(instruction)?;
        }

        Ok(())
    }

    pub fn execute_a_instruction(&mut self, instruction: u16) {
        self.a = instruction & 0x7FFF;
        self.pc += 1;
    }

    #[allow(clippy::unusual_byte_groupings)]
    pub fn execute_c_instruction(&mut self, instruction: u16) -> Result<(), String> {
        let comp_bits = (instruction >> 6) & 0b1111111;
        let alu_result = match comp_bits {
            0b0_101010 => 0u16,                        // 0
            0b0_111111 => 1u16,                        // 1
            0b0_001100 => self.d,                      // D
            0b0_110000 => self.a,                      // M
            0b1_110000 => self.m,                      // A
            0b0_001101 => !self.d,                     // !D
            0b0_110001 => !self.a,                     // !A
            0b1_110001 => !self.m,                     // !M
            0b0_111010 => 0xFFFF,                      // -1
            0b0_001111 => (!self.d).wrapping_add(1),   // -D
            0b0_110011 => (!self.a).wrapping_add(1),   // -A
            0b1_110011 => (!self.m).wrapping_add(1),   // -M
            0b0_011111 => self.d.wrapping_add(1),      // D+1
            0b0_110111 => self.a.wrapping_add(1),      // A+1
            0b1_110111 => self.m.wrapping_add(1),      // M+1
            0b0_001110 => self.d.wrapping_sub(1),      // D-1
            0b0_110010 => self.a.wrapping_sub(1),      // A-1
            0b1_110010 => self.m.wrapping_sub(1),      // M-1
            0b0_000010 => self.d.wrapping_add(self.a), // D+A
            0b1_000010 => self.d.wrapping_add(self.m), // D+M
            0b0_010011 => self.d.wrapping_sub(self.a), // D-A
            0b1_010011 => self.d.wrapping_sub(self.m), // D-M
            0b0_000111 => self.a.wrapping_sub(self.d), // A-D
            0b1_000111 => self.m.wrapping_sub(self.d), // M-D
            0b0_000000 => self.d & self.a,             // D&A
            0b1_000000 => self.d & self.m,             // D&M
            0b0_010101 => self.d | self.a,             // D|A
            0b1_010101 => self.d | self.m,             // D|M
            _ => return Err(format!("no such operation {:#b}", comp_bits)),
        };

        let dest_bits = (instruction >> 3) & 0b111;
        if dest_bits & 0b010 != 0 {
            self.d = alu_result
        }
        if dest_bits & 0b100 != 0 {
            self.a = alu_result;
        }
        if dest_bits & 0b001 != 0 {
            self.m = alu_result;
            self.write_m = true;
        }

        let jump_bits = instruction & 0b111;
        let jump = match jump_bits {
            0b000 => false,
            0b001 => (alu_result as i16) > 0,
            0b010 => (alu_result as i16) == 0,
            0b011 => (alu_result as i16) >= 0,
            0b100 => (alu_result as i16) < 0,
            0b101 => (alu_result as i16) != 0,
            0b110 => (alu_result as i16) <= 0,
            0b111 => true,
            _ => unreachable!(),
        };

        if jump {
            self.pc = self.a;
        } else {
            self.pc += 1;
        }

        Ok(())
    }

    pub fn next_m(&self) -> Option<u16> {
        if self.write_m {
            Some(self.m)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::asm;

    use super::*;

    fn assemble_one(source: &str) -> u16 {
        let tokenizer = asm::Tokenizer::new(source);
        let mut parser = asm::Parser::new(tokenizer);
        let instructions = parser.parse().unwrap();

        let mut gen = asm::Codegen::new();
        let machine_code = gen.generate(&instructions).unwrap();
        let inst = machine_code
            .lines()
            .map(|line| u16::from_str_radix(line.trim_end(), 2).unwrap())
            .collect::<Vec<u16>>();
        inst[0]
    }

    #[test]
    fn test_alu() {
        let mut cpu = Cpu::new();
        cpu.d = 2;
        cpu.m = 1;
        cpu.execute(assemble_one("@12")).unwrap();
        assert_eq!(cpu.a, 12);
        cpu.execute(assemble_one("A=0")).unwrap();
        assert_eq!(cpu.a, 0);
        cpu.execute(assemble_one("A=1")).unwrap();
        assert_eq!(cpu.a, 1);
        cpu.execute(assemble_one("A=D")).unwrap();
        assert_eq!(cpu.a, 2);
        cpu.execute(assemble_one("A=M")).unwrap();
        assert_eq!(cpu.a, 1);
        cpu.execute(assemble_one("A=A")).unwrap();
        assert_eq!(cpu.a, 1);
        cpu.execute(assemble_one("A=!D")).unwrap();
        assert_eq!(cpu.a, 0b1111111111111101);
        cpu.execute(assemble_one("A=!A")).unwrap();
        assert_eq!(cpu.a, 2);
        cpu.execute(assemble_one("A=!M")).unwrap();
        assert_eq!(cpu.a, 0b1111111111111110);
        cpu.execute(assemble_one("A=-1")).unwrap();
        assert_eq!(cpu.a, 0xffff);
        cpu.execute(assemble_one("A=-D")).unwrap();
        assert_eq!(cpu.a, 0b1111111111111110);
        cpu.execute(assemble_one("A=-A")).unwrap();
        assert_eq!(cpu.a, 2);
        cpu.execute(assemble_one("A=-M")).unwrap();
        assert_eq!(cpu.a, 0xffff);
        cpu.execute(assemble_one("A=D+1")).unwrap();
        assert_eq!(cpu.a, 3);
        cpu.execute(assemble_one("A=A+1")).unwrap();
        assert_eq!(cpu.a, 4);
        cpu.execute(assemble_one("A=M+1")).unwrap();
        assert_eq!(cpu.a, 2);
        cpu.execute(assemble_one("A=D-1")).unwrap();
        assert_eq!(cpu.a, 1);
        cpu.execute(assemble_one("A=A-1")).unwrap();
        assert_eq!(cpu.a, 0);
        cpu.execute(assemble_one("A=M-1")).unwrap();
        assert_eq!(cpu.a, 0);
        cpu.execute(assemble_one("A=D+A")).unwrap();
        assert_eq!(cpu.a, 2);
        cpu.execute(assemble_one("A=D+M")).unwrap();
        assert_eq!(cpu.a, 3);
        cpu.execute(assemble_one("A=D-A")).unwrap();
        assert_eq!(cpu.a, 0xffff);
        cpu.execute(assemble_one("A=D-M")).unwrap();
        assert_eq!(cpu.a, 1);
        cpu.execute(assemble_one("A=A-D")).unwrap();
        assert_eq!(cpu.a, 0xffff);
        cpu.execute(assemble_one("A=M-D")).unwrap();
        assert_eq!(cpu.a, 0xffff);
        cpu.execute(assemble_one("A=D&A")).unwrap();
        assert_eq!(cpu.a, 2);
        cpu.execute(assemble_one("A=D&M")).unwrap();
        assert_eq!(cpu.a, 0);
        cpu.a = 1;
        cpu.execute(assemble_one("A=D|A")).unwrap();
        assert_eq!(cpu.a, 3);

        cpu.m = 123;
        cpu.execute(assemble_one("AM=M+1")).unwrap();
        assert_eq!(cpu.a, 124);
        assert_eq!(cpu.m, 124);
        assert!(cpu.write_m);
    }
}
