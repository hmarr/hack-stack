use std::collections::HashMap;

use crate::{errors::SpanError, parse::ast};

pub struct Codegen<'a> {
    symbol_table: HashMap<&'a str, u16>,
    next_var_addr: u16,
}

impl<'a> Codegen<'a> {
    pub fn new() -> Self {
        let mut symbol_table = HashMap::new();
        symbol_table.insert("SP", 0);
        symbol_table.insert("LCL", 1);
        symbol_table.insert("ARG", 2);
        symbol_table.insert("THIS", 3);
        symbol_table.insert("THAT", 4);
        symbol_table.insert("R0", 0);
        symbol_table.insert("R1", 1);
        symbol_table.insert("R2", 2);
        symbol_table.insert("R3", 3);
        symbol_table.insert("R4", 4);
        symbol_table.insert("R5", 5);
        symbol_table.insert("R6", 6);
        symbol_table.insert("R7", 7);
        symbol_table.insert("R8", 8);
        symbol_table.insert("R9", 9);
        symbol_table.insert("R10", 10);
        symbol_table.insert("R11", 11);
        symbol_table.insert("R12", 12);
        symbol_table.insert("R13", 13);
        symbol_table.insert("R14", 14);
        symbol_table.insert("R15", 15);
        symbol_table.insert("SCREEN", 0x4000);
        symbol_table.insert("KBD", 0x6000);

        Self {
            symbol_table,
            next_var_addr: 0x10,
        }
    }

    pub fn generate(&mut self, ast: &'a [ast::Instruction]) -> Result<String, Vec<SpanError>> {
        let mut instructions = vec![];
        for instruction in ast {
            match &instruction {
                ast::Instruction::Label(label) => {
                    self.symbol_table
                        .insert(label.name, instructions.len() as u16);
                }
                ast::Instruction::A { .. } | ast::Instruction::C { .. } => {
                    instructions.push(instruction);
                }
            };
        }

        let mut errors = vec![];
        let mut buf = String::with_capacity(instructions.len() * 17);
        for instruction in ast {
            let inst = match &instruction {
                ast::Instruction::Label(_) => None,
                ast::Instruction::A(inst) => Some(self.a_instruction(inst)),
                ast::Instruction::C(inst) => match self.c_instruction(inst) {
                    Ok(inst) => Some(inst),
                    Err(err) => {
                        errors.push(SpanError::new(err, instruction.span()));
                        None
                    }
                },
            };

            if let Some(inst) = inst {
                buf.push_str(&format!("{:016b}\n", inst));
            }
        }

        if errors.is_empty() {
            Ok(buf)
        } else {
            Err(errors)
        }
    }

    fn a_instruction(&mut self, inst: &'a ast::AInstruction) -> u16 {
        let numeric_address = match inst.addr {
            ast::Address::Symbol(s) => match self.symbol_table.get(s) {
                Some(num_addr) => *num_addr,
                None => {
                    let addr = self.next_var_addr;
                    self.next_var_addr += 1;
                    self.symbol_table.insert(s, addr);
                    addr
                }
            },
            ast::Address::Value(n) => n,
        };
        numeric_address & 0x7FFF
    }

    fn c_instruction(&mut self, inst: &ast::CInstruction) -> Result<u16, String> {
        let binary_inst = 0xE000u16;

        let dest_bits = match &inst.dest {
            Some(dest) => {
                let a: u16 = if dest.a { 0b100 } else { 0 };
                let d: u16 = if dest.d { 0b010 } else { 0 };
                let m: u16 = if dest.m { 0b001 } else { 0 };
                a | d | m
            }
            None => 0,
        };

        let comp_bits = self.comp_bits(&inst.comp)?;

        let jump_bits = match inst.jump {
            None => 0b000,
            Some(ast::Jump::JGT) => 0b001,
            Some(ast::Jump::JEQ) => 0b010,
            Some(ast::Jump::JGE) => 0b011,
            Some(ast::Jump::JLT) => 0b100,
            Some(ast::Jump::JNE) => 0b101,
            Some(ast::Jump::JLE) => 0b110,
            Some(ast::Jump::JMP) => 0b111,
        };

        Ok(binary_inst | (comp_bits << 6) | (dest_bits << 3) | jump_bits)
    }

    fn comp_bits(&self, comp: &ast::Comp) -> Result<u16, String> {
        use ast::{BinaryOperator::*, Bit::*, Operand::*, Register::*, UnaryOperator};

        let comp_bits = match comp {
            ast::Comp::Bit(Zero) => 0b0_101010,
            ast::Comp::Bit(One) => 0b0_111111,
            ast::Comp::Register(D) => 0b0_001100,
            ast::Comp::Register(A) => 0b0_110000,
            ast::Comp::Register(M) => 0b1_110000,
            ast::Comp::UnaryOperation(ast::UnaryOperation { op, operand }) => match (op, operand) {
                (UnaryOperator::Not, Register(D)) => 0b0_001101,
                (UnaryOperator::Not, Register(A)) => 0b0_110001,
                (UnaryOperator::Not, Register(M)) => 0b1_110001,
                (UnaryOperator::Minus, Bit(One)) => 0b0_111010,
                (UnaryOperator::Minus, Register(D)) => 0b0_001111,
                (UnaryOperator::Minus, Register(A)) => 0b0_110011,
                (UnaryOperator::Minus, Register(M)) => 0b1_110011,
                _ => return Err(format!("invalid operation {:?} {:?}", op, operand)),
            },
            ast::Comp::BinaryOperation(ast::BinaryOperation { lhs, op, rhs }) => {
                match (lhs, op, rhs) {
                    (D, Plus, Bit(One)) => 0b0_011111,
                    (A, Plus, Bit(One)) => 0b0_110111,
                    (M, Plus, Bit(One)) => 0b1_110111,
                    (D, Minus, Bit(One)) => 0b0_001110,
                    (A, Minus, Bit(One)) => 0b0_110010,
                    (M, Minus, Bit(One)) => 0b1_110010,
                    (D, Plus, Register(A)) => 0b0_000010,
                    (D, Plus, Register(M)) => 0b1_000010,
                    (D, Minus, Register(A)) => 0b0_010011,
                    (D, Minus, Register(M)) => 0b1_010011,
                    (A, Minus, Register(D)) => 0b0_000111,
                    (M, Minus, Register(D)) => 0b1_000111,
                    (D, And, Register(A)) => 0b0_000000,
                    (D, And, Register(M)) => 0b1_000000,
                    (D, Or, Register(A)) => 0b0_010101,
                    (D, Or, Register(M)) => 0b1_010101,
                    _ => return Err(format!("invalid operation {:?} {:?} {:?}", lhs, op, rhs)),
                }
            }
        };
        Ok(comp_bits)
    }
}

#[cfg(test)]
mod tests {
    use crate::{parse::Parser, tokenize::Tokenizer};

    use super::*;

    #[test]
    fn test_instructions() {
        let src = "@3
                        D=D-A;JMP";
        let expected = "0000000000000011\n\
                             1110010011010111\n";
        let mut parser = Parser::new(Tokenizer::new(src));
        let mut cg = Codegen::new();
        let out = cg.generate(&parser.parse().unwrap()).unwrap();
        assert_eq!(out, expected);
    }

    #[test]
    fn test_labels() {
        let src = "@1
                        @2
                        (thing)
                        M=0
                        @thing
                        @end
                        (end)
                        M=0";
        let expected = "0000000000000001\n\
                             0000000000000010\n\
                             1110101010001000\n\
                             0000000000000010\n\
                             0000000000000101\n\
                             1110101010001000\n";
        let mut parser = Parser::new(Tokenizer::new(src));
        let mut cg = Codegen::new();
        let out = cg.generate(&parser.parse().unwrap()).unwrap();
        assert_eq!(out, expected);
    }

    #[test]
    fn test_variable_addresses() {
        let src = "@foo
                        M=0
                        @bar
                        M=1
                        @R15";
        let expected = "0000000000010000\n\
                                 1110101010001000\n\
                                 0000000000010001\n\
                                 1110111111001000\n\
                                 0000000000001111\n";
        let mut parser = Parser::new(Tokenizer::new(src));
        let mut cg = Codegen::new();
        let out = cg.generate(&parser.parse().unwrap()).unwrap();
        assert_eq!(out, expected);
    }
}
