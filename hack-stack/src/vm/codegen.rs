use std::collections::HashSet;

use super::{ast, ir};
use crate::common::{SourceFile, SpanError};

pub struct Codegen<'a> {
    buf: String,
    source_file: Option<&'a SourceFile>,
    module_name: Option<String>,
    function_name: Option<String>,
    next_label_index: usize,
    emitted_return_def: bool,
    emitted_call_defs: HashSet<String>,
    errors: Vec<SpanError>,
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
    pub fn new(initialize_sp: bool) -> Self {
        let mut buf = String::new();

        if initialize_sp {
            buf.push_str("// SP=256\n");
            buf.push_str("@256\nD=A\n@0\nM=D");
            buf.push_str("\n\n");
        }

        Self {
            buf,
            function_name: None,
            source_file: None,
            module_name: None,
            next_label_index: 0,
            emitted_return_def: false,
            emitted_call_defs: HashSet::new(),
            errors: vec![],
        }
    }

    pub fn generate_from_function(
        &mut self,
        function: &ir::Function<'a>,
    ) -> Result<(), Vec<SpanError>> {
        self.generate_from_ir(function.source_file, function.name, &function.instructions)
    }

    pub fn generate_from_ir(
        &mut self,
        source_file: &'a SourceFile,
        function_name: &'a str,
        instructions: &[ir::Instruction],
    ) -> Result<(), Vec<SpanError>> {
        self.module_name = Some(source_file.name.replace(".vm", "").replace('/', ":"));
        self.function_name = Some(function_name.to_string());
        self.source_file = Some(source_file);
        self.errors.clear();

        for inst in instructions.iter() {
            match inst {
                ir::Instruction::SimpleInstruction(instruction) => {
                    self.generate_instruction(instruction);
                }
            }
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    pub fn finalize(mut self) -> Result<String, Vec<SpanError>> {
        if self.errors.is_empty() {
            // At the end of the program, enter an infinite loop to avoid running
            // the program counter into unknown territory
            self.buf.push_str("($vm.infinite_loop)\n");
            self.buf.push_str("@$vm.infinite_loop\n");
            self.buf.push_str("0;JMP\n");

            Ok(self.buf)
        } else {
            Err(self.errors)
        }
    }

    fn generate_instruction(&mut self, instruction: &ast::Instruction) {
        // Emit a comment showing the VM instruction to make the assembly easier to read
        self.buf.push_str("// ");
        self.emit(self.source_file.unwrap().str_for_span(instruction.span()));

        match &instruction {
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
            ast::Instruction::Goto(goto) => self.goto(goto),
            ast::Instruction::IfGoto(if_goto) => self.if_goto(if_goto),
            ast::Instruction::Label(label) => self.label(label),
            ast::Instruction::Function(function) => self.function(function),
            ast::Instruction::Return(_) => self.return_(),
            ast::Instruction::Call(call) => self.call(call),
        };
        self.buf.push('\n');
    }

    fn push(&mut self, inst: &ast::PushInstruction) {
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
                self.set_a(&format!("{}.{}", self.module_name(), inst.offset));
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
                    self.errors.push(SpanError::new(
                        "pointer offset must be 0 or 1".to_string(),
                        inst.span,
                    ));
                }
                self.set_a(&(POINTER_BASE_ADDR + inst.offset).to_string());
                self.emit("D=M");
                self.pushd();
            }
        }
    }

    fn pop(&mut self, inst: &ast::PopInstruction) {
        match inst.segment {
            ast::Segment::Constant => {
                self.errors.push(SpanError::new(
                    "cannot pop to virtual memory segment constant".to_string(),
                    inst.span,
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
                self.set_a(&format!("{}.{}", self.module_name(), inst.offset));
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
                    self.errors.push(SpanError::new(
                        "pointer offset must be 0 or 1".to_string(),
                        inst.span,
                    ));
                }
                self.popd(PopOp::Assign);
                self.set_a(&(POINTER_BASE_ADDR + inst.offset).to_string());
                self.emit("M=D");
            }
        }
    }

    fn binary_op(&mut self, op: PopOp) {
        // Assign the top-of-stack operand (operand 2) to D
        self.popd(PopOp::Assign);
        // At this point, we've decremented SP by one, which is where we want SP
        // to end up (as we're going from two operands to one return value).
        // Rather than popping the next operand then pushing the result, we just
        // decrement A and modify the memory location in-place.
        self.emit("A=A-1");
        match op {
            PopOp::Assign => self.emit("M=M"),
            PopOp::Add => self.emit("M=D+M"),
            PopOp::And => self.emit("M=D&M"),
            PopOp::Or => self.emit("M=D|M"),
            PopOp::MSubD => self.emit("M=M-D"),
        }
    }

    fn cmp(&mut self, jump_type: &str) {
        // Assign the top-of-stack operand (operand 2) to D
        self.popd(PopOp::Assign);
        // Subtract the D from the next operand (operand 1)
        self.popd(PopOp::MSubD);
        // Handle the truthy case, setting the top of the stack to 1 (true)
        self.emit("M=-1");

        // Generate unique lable to jump to the end
        let end_label = format!(
            "{}$cmp_end.{}",
            self.scope_identifier(),
            self.next_label_index
        );
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
    }

    fn neg(&mut self) {
        self.set_a("SP");
        self.emit("A=M-1");
        self.emit("M=-M");
    }

    fn not(&mut self) {
        self.set_a("SP");
        self.emit("A=M-1");
        self.emit("M=!M");
    }

    fn goto(&mut self, inst: &ast::GotoInstruction) {
        self.set_a(&format!("{}${}", self.scope_identifier(), inst.label));
        self.emit("0;JMP");
    }

    fn if_goto(&mut self, inst: &ast::IfGotoInstruction) {
        self.dec_deref_sp();
        self.emit("D=M");
        self.set_a(&format!("{}${}", self.scope_identifier(), inst.label));
        self.emit("D;JNE");
    }

    fn label(&mut self, inst: &ast::LabelInstruction) {
        self.emit(&format!("({}${})", self.scope_identifier(), inst.label));
    }

    fn function(&mut self, inst: &ast::FunctionInstruction) {
        self.function_name = Some(inst.name.to_string());

        self.emit(&format!("({})", inst.name));

        // Initialise each of the locals to zero
        for _ in 0..inst.locals {
            self.set_a("SP");
            self.emit("M=M+1");
            self.emit("A=M-1");
            self.emit("M=0");
        }
    }

    fn return_(&mut self) {
        if !self.emitted_return_def {
            self.return_def();
            self.emitted_return_def = true;
        }

        self.set_a("$vm.return");
        self.emit("0;JMP");
    }

    fn return_def(&mut self) {
        self.emit("($vm.return)");

        // Save return address to R13
        self.set_a("5");
        self.emit("D=A");
        self.set_a("LCL");
        self.emit("A=M-D");
        self.emit("D=M");
        self.set_a("R13");
        self.emit("M=D");

        // Copy the return value to *ARG (the top of the caller's stack)
        self.popd(PopOp::Assign);
        self.set_a("ARG");
        self.emit("A=M");
        self.emit("M=D");

        // Set SP=ARG+1
        self.emit("D=A+1");
        self.set_a("SP");
        self.emit("M=D");

        // Restore segment pointers from stack frame
        for segment in ["THAT", "THIS", "ARG", "LCL"] {
            self.set_a("LCL");
            self.emit("AM=M-1");
            self.emit("D=M");
            self.set_a(segment);
            self.emit("M=D");
        }

        // Jump to the return address
        self.set_a("R13");
        self.emit("A=M");
        self.emit("0;JMP");
    }

    fn call(&mut self, inst: &ast::CallInstruction) {
        // Push the return label (File.callingFunction$calledFunction$ret.n) to the stack
        let ret = &format!(
            "{}${}$ret.{}",
            self.scope_identifier(),
            inst.function,
            self.next_label_index
        );
        self.next_label_index += 1;
        self.set_a(ret);
        self.emit("D=A");

        let call_label = format!("{}${}$call", inst.function, inst.args);
        if !self.emitted_call_defs.contains(&call_label) {
            // Emit call definition
            self.emit(&format!("({})", &call_label));
            self.call_def(inst);
            self.emitted_call_defs.insert(call_label);
        } else {
            // Jump to existing call definition
            self.set_a(&call_label);
            self.emit("0;JMP");
        }

        // Return label - this is where we come back to once the function call ends
        self.emit(&format!("({})", ret));
    }

    fn call_def(&mut self, inst: &ast::CallInstruction) {
        // At the start of a call, D contains the return address
        self.pushd();

        // Save segment pointers to stack frame
        for segment in ["LCL", "ARG", "THIS", "THAT"] {
            self.set_a(segment);
            self.emit("D=M");
            self.pushd();
        }

        // Reposition ARG
        // Set D to SP value (A=*SP-1 after pushd)
        self.emit("D=A+1");
        // Subtract 4 (saved segment pointers) + 1 (return addr) + args to get the ARG pointer
        self.set_a(&(5 + inst.args).to_string());
        self.emit("D=D-A");
        self.set_a("ARG");
        self.emit("M=D");

        // Reposition LCL
        self.set_a("SP");
        self.emit("D=M");
        self.set_a("LCL");
        self.emit("M=D");

        // Jump to function
        self.set_a(inst.function);
        self.emit("0;JMP");
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
        self.emit("M=M+1");
        self.emit("A=M-1");
        self.emit("M=D");
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

    fn module_name(&self) -> &String {
        self.module_name.as_ref().unwrap()
    }

    fn scope_identifier(&self) -> &String {
        self.function_name
            .as_ref()
            .or(self.module_name.as_ref())
            .unwrap()
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
        M=M+1
        A=M-1
        M=D

        // push static 7
        @Test.7
        D=M
        @SP
        M=M+1
        A=M-1
        M=D
        
        // push local 6
        @6
        D=A
        @LCL
        A=D+M
        D=M
        @SP
        M=M+1
        A=M-1
        M=D

        // push argument 5
        @5
        D=A
        @ARG
        A=D+M
        D=M
        @SP
        M=M+1
        A=M-1
        M=D

        // push this 4
        @4
        D=A
        @THIS
        A=D+M
        D=M
        @SP
        M=M+1
        A=M-1
        M=D

        // push that 3
        @3
        D=A
        @THAT
        A=D+M
        D=M
        @SP
        M=M+1
        A=M-1
        M=D
        
        // push temp 2
        @7
        D=M
        @SP
        M=M+1
        A=M-1
        M=D
        
        // push pointer 1
        @4
        D=M
        @SP
        M=M+1
        A=M-1
        M=D";
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
        A=A-1
        M=D+M
        
        // sub
        @SP
        AM=M-1
        D=M
        A=A-1
        M=M-D
        
        // eq
        @SP
        AM=M-1
        D=M
        @SP
        AM=M-1
        D=M-D
        M=-1
        @Test$cmp_end.0
        D;JEQ
        @SP
        A=M
        M=0
        (Test$cmp_end.0)
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
        @Test$cmp_end.1
        D;JGT
        @SP
        A=M
        M=0
        (Test$cmp_end.1)
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
        @Test$cmp_end.2
        D;JLT
        @SP
        A=M
        M=0
        (Test$cmp_end.2)
        @SP
        M=M+1

        // neg
        @SP
        A=M-1
        M=-M

        // and
        @SP
        AM=M-1
        D=M
        A=A-1
        M=D&M

        // or
        @SP
        AM=M-1
        D=M
        A=A-1
        M=D|M
        
        // not
        @SP
        A=M-1
        M=!M";
        check_translation(src, expected);
    }

    #[test]
    fn test_branching() {
        let src = "
        goto FOO
        label FOO
        if-goto FOO";

        let expected = "
        // goto FOO
        @Test$FOO
        0;JMP

        // label FOO
        (Test$FOO)

        // if-goto FOO
        @SP
        AM=M-1
        D=M
        @Test$FOO
        D;JNE";
        check_translation(src, expected);
    }

    #[test]
    fn test_functions() {
        let src = "
        function Test.foo 2
        return
        function Test.bar 0
        call Test.foo 3
        call Test.foo 3
        return";

        let expected = "
        // function Test.foo 2
        (Test.foo)
        @SP
        M=M+1
        A=M-1
        M=0
        @SP
        M=M+1
        A=M-1
        M=0

        // return
        ($vm.return)
        @5
        D=A
        @LCL
        A=M-D
        D=M
        @R13
        M=D
        @SP
        AM=M-1
        D=M
        @ARG
        A=M
        M=D
        D=A+1
        @SP
        M=D
        @LCL
        AM=M-1
        D=M
        @THAT
        M=D
        @LCL
        AM=M-1
        D=M
        @THIS
        M=D
        @LCL
        AM=M-1
        D=M
        @ARG
        M=D
        @LCL
        AM=M-1
        D=M
        @LCL
        M=D
        @R13
        A=M
        0;JMP
        @$vm.return
        0;JMP

        // function Test.bar 0
        (Test.bar)

        // call Test.foo 3
        @Test.bar$Test.foo$ret.0
        D=A
        (Test.foo$3$call)
        @SP
        M=M+1
        A=M-1
        M=D
        @LCL
        D=M
        @SP
        M=M+1
        A=M-1
        M=D
        @ARG
        D=M
        @SP
        M=M+1
        A=M-1
        M=D
        @THIS
        D=M
        @SP
        M=M+1
        A=M-1
        M=D
        @THAT
        D=M
        @SP
        M=M+1
        A=M-1
        M=D
        D=A+1
        @8
        D=D-A
        @ARG
        M=D
        @SP
        D=M
        @LCL
        M=D
        @Test.foo
        0;JMP
        (Test.bar$Test.foo$ret.0)
        
        // call Test.foo 3
        @Test.bar$Test.foo$ret.1
        D=A
        @Test.foo$3$call
        0;JMP
        (Test.bar$Test.foo$ret.1)
        
        // return
        @$vm.return
        0;JMP";

        check_translation(src, expected);
    }

    fn check_translation(vm_src: &str, expected_asm: &str) {
        let epilogue = "
        ($vm.infinite_loop)
        @$vm.infinite_loop
        0;JMP";

        let full_asm = format!("{}\n{}", expected_asm, epilogue);
        assert_eq!(strip_indent(&translate(vm_src)), strip_indent(&full_asm));
    }

    fn translate(vm_src: &str) -> String {
        let mut parser = Parser::new(Tokenizer::new(vm_src));
        let source_file = SourceFile::new(vm_src.to_owned(), "Test.vm".to_owned());
        let mut cg = Codegen::new(false);
        cg.generate_from_ir(
            &source_file,
            "Test",
            &parser
                .parse()
                .unwrap()
                .into_iter()
                .map(ir::Instruction::SimpleInstruction)
                .collect::<Vec<_>>(),
        )
        .unwrap();
        cg.finalize().unwrap()
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
