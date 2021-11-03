use super::ast;
use crate::common::{SourceFile, SpanError};

pub struct Codegen<'a> {
    source_file: &'a SourceFile<'a>,
    buf: String,
    static_prefix: String,
    next_label_index: usize,
}

enum PopOp {
    Assign,
    Add,
    And,
    Or,
    MSubD,
}

const TEMP_BASE_ADDR: u16 = 5;
const POINTER_BASE_ADDR: u16 = 3;

impl<'a> Codegen<'a> {
    pub fn new(source_file: &'a SourceFile<'a>) -> Self {
        Self {
            source_file,
            buf: String::new(),
            static_prefix: source_file.name.replace(".vm", ""),
            next_label_index: 0,
        }
    }

    pub fn generate(&mut self, ast: &[ast::Instruction]) -> Result<&str, Vec<SpanError>> {
        let mut errors = vec![];
        for instruction in ast {
            // Emit a comment showing the VM instruction to make the assembly easier to read
            self.buf.push_str("// ");
            self.emit(self.source_file.str_for_span(instruction.span()));

            let res = match &instruction {
                ast::Instruction::Push(push) => self.push(push),
                ast::Instruction::Pop(pop) => self.pop(pop),
                ast::Instruction::Add(_) => self.binary_op(PopOp::Add),
                ast::Instruction::Sub(_) => self.binary_op(PopOp::MSubD),
                ast::Instruction::Eq(_) => self.cmp("JEQ"),
                ast::Instruction::Gt(_) => self.cmp("JGT"),
                ast::Instruction::Lt(_) => self.cmp("JLT"),
                ast::Instruction::Neg(_) => self.neg(),
                ast::Instruction::And(_) => self.binary_op(PopOp::And),
                ast::Instruction::Or(_) => self.binary_op(PopOp::Or),
                ast::Instruction::Not(_) => self.not(),
            };
            self.buf.push('\n');

            // If we got an error, keep track of it but keep going to see if we find more
            if let Err(msg) = res {
                errors.push(SpanError::new(msg, instruction.span()));
            }
        }

        // At the end of the program, enter an infinite loop to avoid running
        // the program counter into unknown territory
        self.emit("(VM_END_LOOP)");
        self.emit("@VM_END_LOOP");
        self.emit("0;JMP");

        if errors.is_empty() {
            Ok(&self.buf)
        } else {
            Err(errors)
        }
    }

    fn push(&mut self, inst: &ast::PushInstruction) -> Result<(), String> {
        match inst.segment {
            ast::Segment::Constant => {
                // CONSTANT is a virtual memory segment that just loads constant
                // values onto the stack
                self.setd_const(inst.offset);
                self.pushd();
            }
            ast::Segment::Local => {
                self.setd_segment_value("LCL", inst.offset);
                self.pushd();
            }
            ast::Segment::Argument => {
                self.setd_segment_value("ARG", inst.offset);
                self.pushd();
            }
            ast::Segment::Static => {
                self.set_a(&format!("{}.{}", self.static_prefix, inst.offset));
                self.emit("D=M");
                self.pushd();
            }
            ast::Segment::This => {
                self.setd_segment_value("THIS", inst.offset);
                self.pushd();
            }
            ast::Segment::That => {
                self.setd_segment_value("THAT", inst.offset);
                self.pushd();
            }
            ast::Segment::Temp => {
                self.set_a(&(TEMP_BASE_ADDR + inst.offset).to_string());
                self.emit("D=M");
                self.pushd();
            }
            ast::Segment::Pointer => {
                if inst.offset > 1 {
                    return Err(String::from("pointer offset must be 0 or 1"));
                }
                self.set_a(&(POINTER_BASE_ADDR + inst.offset).to_string());
                self.emit("D=M");
                self.pushd();
            }
        }
        Ok(())
    }

    fn pop(&mut self, inst: &ast::PopInstruction) -> Result<(), String> {
        match inst.segment {
            ast::Segment::Constant => {
                return Err(String::from(
                    "cannot pop to virtual memory segment constant",
                ));
            }
            ast::Segment::Local => {
                self.pop_to_segment("LCL", inst.offset);
            }
            ast::Segment::Argument => {
                self.pop_to_segment("ARG", inst.offset);
            }
            ast::Segment::Static => {
                self.popd(PopOp::Assign);
                self.set_a(&format!("{}.{}", self.static_prefix, inst.offset));
                self.emit("M=D");
            }
            ast::Segment::This => {
                self.pop_to_segment("THIS", inst.offset);
            }
            ast::Segment::That => {
                self.pop_to_segment("THAT", inst.offset);
            }
            ast::Segment::Temp => {
                self.popd(PopOp::Assign);
                self.set_a(&(TEMP_BASE_ADDR + inst.offset).to_string());
                self.emit("M=D");
            }
            ast::Segment::Pointer => {
                if inst.offset > 1 {
                    return Err(String::from("pointer offset must be 0 or 1"));
                }
                self.popd(PopOp::Assign);
                self.set_a(&(POINTER_BASE_ADDR + inst.offset).to_string());
                self.emit("M=D");
            }
        }
        Ok(())
    }

    fn binary_op(&mut self, op: PopOp) -> Result<(), String> {
        // Assign the top-of-stack operand (operand 2) to D
        self.popd(PopOp::Assign);
        // Apply the operation to the next operand (operand 1) to D
        self.dec_deref_sp();
        match op {
            PopOp::Assign => self.emit("M=M"),
            PopOp::Add => self.emit("M=D+M"),
            PopOp::And => self.emit("M=D&M"),
            PopOp::Or => self.emit("M=D|M"),
            PopOp::MSubD => self.emit("M=M-D"),
        }
        // Return the result to the stack
        self.inc_sp();
        Ok(())
    }

    fn cmp(&mut self, jump_type: &str) -> Result<(), String> {
        // Assign the top-of-stack operand (operand 2) to D
        self.popd(PopOp::Assign);
        // Subtract the D from the next operand (operand 1)
        self.popd(PopOp::MSubD);
        // Handle the truthy case, setting the top of the stack to 1 (true)
        self.emit("M=-1");

        // Generate unique lable to jump to the end
        let end_label = format!("{}:END_CMP.{}", self.static_prefix, self.next_label_index);
        self.next_label_index += 1;
        self.set_a(&end_label);

        // Jump to the end if comparison operation is true
        self.emit(&format!("D;{}", jump_type));

        // If we haven't jumped, set the top of the stack to 0 (false)
        self.set_a("SP");
        self.emit("A=M");
        self.emit("M=0");

        // Emit the end label and bump SP
        self.emit(&format!("({})", end_label));
        self.inc_sp();

        Ok(())
    }

    fn neg(&mut self) -> Result<(), String> {
        self.dec_deref_sp();
        self.emit("M=!M");
        self.emit("M=M+1");
        self.inc_sp();
        Ok(())
    }

    fn not(&mut self) -> Result<(), String> {
        self.dec_deref_sp();
        self.emit("M=!M");
        self.inc_sp();
        Ok(())
    }

    fn pop_to_segment(&mut self, seg: &str, offset: u16) {
        self.setd_segment_ptr(seg, offset);
        self.set_a("R13");
        self.emit("M=D");
        self.popd(PopOp::Assign);
        self.set_a("R13");
        self.emit("A=M");
        self.emit("M=D");
    }

    fn setd_const(&mut self, val: u16) {
        self.set_a(&val.to_string());
        self.emit("D=A");
    }

    fn setd_segment_value(&mut self, seg: &str, offset: u16) {
        self.setd_const(offset);
        self.set_a(seg);
        self.emit("A=D+M");
        self.emit("D=M");
    }

    fn setd_segment_ptr(&mut self, seg: &str, offset: u16) {
        self.setd_const(offset);
        self.set_a(seg);
        self.emit("D=D+M");
    }

    fn popd(&mut self, op: PopOp) {
        self.dec_deref_sp();
        match op {
            PopOp::Assign => self.emit("D=M"),
            PopOp::Add => self.emit("D=D+M"),
            PopOp::And => self.emit("D=D&M"),
            PopOp::Or => self.emit("D=D|M"),
            PopOp::MSubD => self.emit("D=M-D"),
        }
    }

    fn pushd(&mut self) {
        self.set_a("SP");
        self.emit("A=M");
        self.emit("M=D");
        self.inc_sp();
    }

    fn inc_sp(&mut self) {
        self.set_a("SP");
        self.emit("M=M+1");
    }

    fn dec_deref_sp(&mut self) {
        self.set_a("SP");
        self.emit("AM=M-1");
    }

    fn set_a(&mut self, a: &str) {
        self.buf.push('@');
        self.buf.push_str(a);
        self.buf.push('\n');
    }

    fn emit(&mut self, s: &str) {
        self.buf.push_str(s);
        self.buf.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::min;

    use crate::vm::{parser::Parser, Tokenizer};

    use super::*;

    #[test]
    fn test_push() {
        let src = "
        push constant 8
        push static 7
        push local 6
        push argument 5
        push this 4
        push that 3
        push temp 2
        push pointer 1";

        let expected = "
        // push constant 8
        @8
        D=A
        @SP
        A=M
        M=D
        @SP
        M=M+1

        // push static 7
        @Test.7
        D=M
        @SP
        A=M
        M=D
        @SP
        M=M+1
        
        // push local 6
        @6
        D=A
        @LCL
        A=D+M
        D=M
        @SP
        A=M
        M=D
        @SP
        M=M+1

        // push argument 5
        @5
        D=A
        @ARG
        A=D+M
        D=M
        @SP
        A=M
        M=D
        @SP
        M=M+1

        // push this 4
        @4
        D=A
        @THIS
        A=D+M
        D=M
        @SP
        A=M
        M=D
        @SP
        M=M+1

        // push that 3
        @3
        D=A
        @THAT
        A=D+M
        D=M
        @SP
        A=M
        M=D
        @SP
        M=M+1
        
        // push temp 2
        @7
        D=M
        @SP
        A=M
        M=D
        @SP
        M=M+1
        
        // push pointer 1
        @4
        D=M
        @SP
        A=M
        M=D
        @SP
        M=M+1";
        check_translation(src, expected);
    }

    #[test]
    fn test_pop() {
        let src = "
        pop static 7
        pop local 6
        pop argument 5
        pop this 4
        pop that 3
        pop temp 2
        pop pointer 1";
        // pop static 1

        // SP-- ; D = *SP ; LCL+6 = D
        let expected = "
        // pop static 7
        @SP
        AM=M-1
        D=M
        @Test.7
        M=D

        // pop local 6
        @6
        D=A
        @LCL
        D=D+M
        @R13
        M=D
        @SP
        AM=M-1
        D=M
        @R13
        A=M
        M=D
        
        // pop argument 5
        @5
        D=A
        @ARG
        D=D+M
        @R13
        M=D
        @SP
        AM=M-1
        D=M
        @R13
        A=M
        M=D
        
        // pop this 4
        @4
        D=A
        @THIS
        D=D+M
        @R13
        M=D
        @SP
        AM=M-1
        D=M
        @R13
        A=M
        M=D
        
        // pop that 3
        @3
        D=A
        @THAT
        D=D+M
        @R13
        M=D
        @SP
        AM=M-1
        D=M
        @R13
        A=M
        M=D
        
        // pop temp 2
        @SP
        AM=M-1
        D=M
        @7
        M=D
        
        // pop pointer 1
        @SP
        AM=M-1
        D=M
        @4
        M=D";
        check_translation(src, expected);
    }

    #[test]
    fn test_arithmetic_instructions() {
        let src = "
        add
        sub
        eq
        gt
        lt
        neg
        and
        or
        not";

        let expected = "
        // add
        @SP
        AM=M-1
        D=M
        @SP
        AM=M-1
        M=D+M
        @SP
        M=M+1
        
        // sub
        @SP
        AM=M-1
        D=M
        @SP
        AM=M-1
        M=M-D
        @SP
        M=M+1
        
        // eq
        @SP
        AM=M-1
        D=M
        @SP
        AM=M-1
        D=M-D
        M=-1
        @Test:END_CMP.0
        D;JEQ
        @SP
        A=M
        M=0
        (Test:END_CMP.0)
        @SP
        M=M+1
        
        // gt
        @SP
        AM=M-1
        D=M
        @SP
        AM=M-1
        D=M-D
        M=-1
        @Test:END_CMP.1
        D;JGT
        @SP
        A=M
        M=0
        (Test:END_CMP.1)
        @SP
        M=M+1
        
        // lt
        @SP
        AM=M-1
        D=M
        @SP
        AM=M-1
        D=M-D
        M=-1
        @Test:END_CMP.2
        D;JLT
        @SP
        A=M
        M=0
        (Test:END_CMP.2)
        @SP
        M=M+1

        // neg
        @SP
        AM=M-1
        M=!M
        M=M+1
        @SP
        M=M+1

        // and
        @SP
        AM=M-1
        D=M
        @SP
        AM=M-1
        M=D&M
        @SP
        M=M+1

        // or
        @SP
        AM=M-1
        D=M
        @SP
        AM=M-1
        M=D|M
        @SP
        M=M+1
        
        // not
        @SP
        AM=M-1
        M=!M
        @SP
        M=M+1";
        check_translation(src, expected);
    }

    fn check_translation(vm_src: &str, expected_asm: &str) {
        let epilogue = "
        (VM_END_LOOP)
        @VM_END_LOOP
        0;JMP";

        let full_asm = format!("{}\n{}", expected_asm, epilogue);
        assert_eq!(strip_indent(&translate(vm_src)), strip_indent(&full_asm));
    }

    fn translate(vm_src: &str) -> String {
        let mut parser = Parser::new(Tokenizer::new(vm_src));
        let source_file = SourceFile::new(vm_src, "Test.vm");
        let mut cg = Codegen::new(&source_file);
        cg.generate(&parser.parse().unwrap()).unwrap().to_owned()
    }

    fn strip_indent(s: &str) -> String {
        let s = s.trim_start_matches('\n');

        let min_indent = s
            .lines()
            .map(|l| l.find(|c: char| !c.is_whitespace()))
            .filter_map(|c| c)
            .min()
            .unwrap_or(0);

        s.lines()
            .map(|line| &line[min(min_indent, line.len())..])
            .collect::<Vec<&str>>()
            .join("\n")
    }
}
