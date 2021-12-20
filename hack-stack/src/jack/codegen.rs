use crate::common::SpanError;

use super::{
    ast::*,
    symbol_table::{SymbolKind, SymbolTable},
};

struct VmWriter {
    buf: String,
}

impl VmWriter {
    fn new() -> VmWriter {
        VmWriter { buf: String::new() }
    }

    fn push_constant(&mut self, n: u16) {
        self.push("constant", n);
    }

    fn push(&mut self, segment: &str, index: u16) {
        self.emit(format!("push {} {}", segment, index));
    }

    fn pop(&mut self, segment: &str, index: u16) {
        self.emit(format!("pop {} {}", segment, index));
    }

    fn label(&mut self, label: &str) {
        self.emit(format!("label {}", label));
    }

    fn goto(&mut self, label: &str) {
        self.emit(format!("goto {}", label));
    }

    fn if_goto(&mut self, label: &str) {
        self.emit(format!("if-goto {}", label));
    }

    fn emit<T: AsRef<str>>(&mut self, str: T) {
        self.buf.push_str(str.as_ref());
        self.buf.push('\n');
    }
}

pub struct Codegen<'a> {
    vm_writer: VmWriter,
    next_label_index: usize,
    errors: Vec<SpanError>,
    func_sym_tab: SymbolTable<'a>,
}

impl<'a> Codegen<'a> {
    pub fn new() -> Self {
        Self {
            vm_writer: VmWriter::new(),
            errors: Vec::new(),
            next_label_index: 0,
            func_sym_tab: SymbolTable::new(),
        }
    }

    pub fn generate(&mut self, class: &'a Class) -> Result<&str, &Vec<SpanError>> {
        for dec in &class.subroutine_decs {
            self.compile_subroutine_dec(class, dec);
        }
        if self.errors.is_empty() {
            Ok(self.vm_writer.buf.as_str())
        } else {
            Err(&self.errors)
        }
    }

    fn compile_subroutine_dec(&mut self, class: &'a Class, dec: &'a SubroutineDec) {
        // Figure out the number of locals, which is necessary for the function declaration
        let locals = dec
            .statements
            .iter()
            .map(|s| match &s {
                &Stmt::Var(v) => v.names.len(),
                _ => 0,
            })
            .sum::<usize>();

        self.vm_writer.emit(&format!(
            "function {}.{} {}",
            class.name.item, dec.name.item, locals,
        ));

        self.func_sym_tab.reset();

        // Add parameters to symbol table
        for param in &dec.params {
            self.func_sym_tab
                .add(SymbolKind::Arg, param.ty.item, param.name.item);
        }

        match dec.kind.item {
            SubroutineKind::Constructor => {
                let fields = class
                    .var_decs
                    .iter()
                    .filter(|v| v.kind == ClassVarKind::Field)
                    .map(|v| v.var_dec.names.len())
                    .sum::<usize>();

                self.vm_writer.push_constant(fields as u16);
                self.vm_writer.emit("call Memory.alloc 1");
                self.vm_writer.pop("pointer", 0);

                // Add `this` to the symbol table
                self.func_sym_tab
                    .add(SymbolKind::This, class.name.item, "this");
            }
            SubroutineKind::Method => todo!(),
            SubroutineKind::Function => (),
        }

        // Compile each of the statements in the function
        for stmt in &dec.statements {
            self.compile_statement(stmt);
        }
    }

    fn compile_statement(&mut self, stmt: &'a Stmt) {
        match stmt {
            Stmt::Var(v) => self.handle_var_dec(&v),
            Stmt::Let(l) => self.compile_let(l),
            Stmt::If(i) => self.compile_if(i),
            Stmt::While(w) => self.compile_while(w),
            Stmt::Do(d) => self.compile_do(d),
            Stmt::Return(s) => self.compile_return(&s),
        }
    }

    fn handle_var_dec(&mut self, var_dec: &'a VarDec) {
        for name in &var_dec.names {
            if self.func_sym_tab.get(name.item).is_none() {
                self.func_sym_tab
                    .add(SymbolKind::Var, var_dec.ty.item, name.item);
            } else {
                self.errors.push(SpanError::new(
                    format!("redefinition of variable {}", name.item),
                    name.span,
                ));
            }
        }
    }

    fn compile_let(&mut self, stmt: &'a LetStmt) {
        self.compile_expression(&stmt.expr.item);

        match &stmt.assignee {
            Assignee::Name(name) => {
                if let Some(entry) = self.func_sym_tab.get(name.item) {
                    self.vm_writer.pop(entry.kind.segment_name(), entry.index);
                } else {
                    self.errors.push(SpanError {
                        msg: format!("variable {} not declared", name.item),
                        span: name.span,
                    })
                }
            }
            Assignee::Index(_) => todo!(),
        }
    }

    fn compile_if(&mut self, if_stmt: &'a IfStmt) {
        let else_label = self.generate_label("IF_ELSE");
        let end_label = self.generate_label("IF_END");

        self.compile_expression(&if_stmt.cond.item);
        self.vm_writer.emit("not");
        self.vm_writer.if_goto(&else_label);

        for stmt in &if_stmt.if_arm {
            self.compile_statement(stmt);
        }
        self.vm_writer.goto(&end_label);

        self.vm_writer.label(&else_label);
        for stmt in &if_stmt.else_arm {
            self.compile_statement(stmt);
        }

        self.vm_writer.label(&end_label);
    }

    fn compile_while(&mut self, while_stmt: &'a WhileStmt) {
        let start_label = self.generate_label("WHILE_START");
        let end_label = self.generate_label("WHILE_END");

        self.vm_writer.label(&start_label);
        self.compile_expression(&while_stmt.cond.item);
        self.vm_writer.emit("not");
        self.vm_writer.if_goto(&end_label);

        for stmt in &while_stmt.body {
            self.compile_statement(stmt);
        }
        self.vm_writer.goto(&start_label);

        self.vm_writer.label(&end_label);
    }

    fn compile_do(&mut self, call: &'a SubroutineCall) {
        self.compile_subroutine_call(call);
        self.vm_writer.emit("pop temp 0");
    }

    fn compile_return(&mut self, stmt: &'a ReturnStmt) {
        if let Some(ref expr) = stmt.expr {
            self.compile_expression(expr.item.as_ref());
        } else {
            self.vm_writer.emit("push constant 0");
        }
        self.vm_writer.emit("return");
    }

    fn compile_subroutine_call(&mut self, call: &'a SubroutineCall) {
        match &call.class {
            Some(class_name) => {
                for arg in &call.args {
                    self.compile_expression(&arg.item);
                }
                self.vm_writer.emit(format!(
                    "call {}.{} {}",
                    class_name.item,
                    call.subroutine.item,
                    call.args.len()
                ));
            }
            None => todo!(),
        }
    }

    fn compile_expression(&mut self, exp: &'a Expr) {
        match exp {
            Expr::IntLit(lit) => self.vm_writer.push_constant(lit.item),
            Expr::StrLit(_) => todo!(),
            Expr::BoolLit(val) => match val.item {
                true => {
                    // push_constant(0xffff) doesn't work as addresses are in the range [0, 2^15)
                    self.vm_writer.push_constant(0);
                    self.vm_writer.emit("not");
                }
                false => self.vm_writer.push_constant(0),
            },
            Expr::UnaryOp(un_op) => {
                self.compile_expression(&un_op.expr.item);
                self.vm_writer.emit(match un_op.op.item {
                    UnaryOpKind::Neg => "neg",
                    UnaryOpKind::Not => "not",
                });
            }
            Expr::BinOp(bin_op) => {
                self.compile_expression(&bin_op.lhs.item);
                self.compile_expression(&bin_op.rhs.item);
                self.vm_writer.emit(match bin_op.op.item {
                    BinOpKind::Add => "add",
                    BinOpKind::Sub => "sub",
                    BinOpKind::Mul => "call Math.multiply 2",
                    BinOpKind::Div => "call Math.divide 2",
                    BinOpKind::And => "and",
                    BinOpKind::Or => "or",
                    BinOpKind::Lt => "lt",
                    BinOpKind::Gt => "gt",
                    BinOpKind::Eq => "eq",
                })
            }
            Expr::NullLit(_) => self.vm_writer.push_constant(0),
            Expr::Ident(name) => {
                if let Some(entry) = self.func_sym_tab.get(name.item) {
                    self.vm_writer.push(entry.kind.segment_name(), entry.index);
                } else {
                    self.errors.push(SpanError {
                        msg: format!("variable {} not declared", name.item),
                        span: name.span,
                    })
                }
            }
            Expr::SubroutineCall(c) => self.compile_subroutine_call(c),
            Expr::Index(_) => todo!(),
        }
    }

    fn generate_label(&mut self, prefix: &str) -> String {
        let label = format!("{}_{}", prefix, self.next_label_index);
        self.next_label_index += 1;
        label
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jack::{Parser, Tokenizer};

    #[test]
    fn test_function_def() {
        let src = r#"
        class Test {
          function void test() {
            var int x, y;
            var Point z;
          }
        }
        "#;

        let vm_code = r#"
        function Test.test 3
        "#;
        assert_eq!(
            normalize_whitespace(compile(src)),
            normalize_whitespace(vm_code)
        );
    }

    #[test]
    fn test_function_call() {
        let src = r#"
        class Test {
          function void test() {
            do Math.multiply(5, 3);
          }
        }
        "#;

        let vm_code = r#"
        function Test.test 0
        push constant 5
        push constant 3
        call Math.multiply 2
        pop temp 0
        "#;
        assert_eq!(
            normalize_whitespace(compile(src)),
            normalize_whitespace(vm_code)
        );
    }

    #[test]
    fn test_constructor() {
        let src = r#"
        class Test {
          field int x;
          constructor Test new() {
            return this;
          }
        }
        "#;

        let vm_code = r#"
        function Test.new 0
        push constant 1
        call Memory.alloc 1
        pop pointer 0
        push pointer 0
        return
        "#;
        assert_eq!(
            normalize_whitespace(compile(src)),
            normalize_whitespace(vm_code)
        );
    }

    #[test]
    fn test_function_name_resolution() {
        let src = r#"
        class Test {
          function int test(int a, boolean b) {
            var int x, y;
            var boolean z;
            let x = 1;
            let z = false & b;
            let y = a + 2;
            return x;
          }
        }
        "#;

        let vm_code = r#"
        function Test.test 3
        push constant 1
        pop local 0
        push constant 0
        push argument 1
        and
        pop local 2
        push argument 0
        push constant 2
        add
        pop local 1
        push local 0
        return
        "#;
        assert_eq!(
            normalize_whitespace(compile(src)),
            normalize_whitespace(vm_code)
        );
    }

    #[test]
    fn test_class_name_resolution() {
        // let src = r#"
        // class Test {
        //   static int a;
        //   field int x, y;

        //   function int test(int a, boolean b) {
        //     ...
        //   }
        // }
        // "#;

        // let vm_code = r#"
        // ...
        // "#;

        // assert_eq!(
        //     normalize_whitespace(compile(src)),
        //     normalize_whitespace(vm_code)
        // );
    }

    #[test]
    fn test_expressions() {
        let src = r#"
        class Test {
          function void test() {
            return (-1 + 2) * 3 / 4 - 5;
          }
        }
        "#;

        let vm_code = r#"
        function Test.test 0
        push constant 1
        neg
        push constant 2
        add
        push constant 3
        call Math.multiply 2
        push constant 4
        call Math.divide 2
        push constant 5
        sub
        return
        "#;
        assert_eq!(
            normalize_whitespace(compile(src)),
            normalize_whitespace(vm_code)
        );
    }

    #[test]
    fn test_while() {
        let src = r#"
        class Test {
          function void test() {
            while (false) {
                return 1;
            }
            return 2;
          }
        }
        "#;

        let vm_code = r#"
        function Test.test 0
        label WHILE_START_0
        push constant 0
        not
        if-goto WHILE_END_1
        push constant 1
        return
        goto WHILE_START_0
        label WHILE_END_1
        push constant 2
        return
        "#;
        assert_eq!(
            normalize_whitespace(compile(src)),
            normalize_whitespace(vm_code)
        );
    }

    #[test]
    fn test_if() {
        let src = r#"
        class Test {
          function void test() {
            if (false) {
                return 1;
            } else {
                return 2;
            }
          }
        }
        "#;

        let vm_code = r#"
        function Test.test 0
        push constant 0
        not
        if-goto IF_ELSE_0
        push constant 1
        return
        goto IF_END_1
        label IF_ELSE_0
        push constant 2
        return
        label IF_END_1
        "#;
        assert_eq!(
            normalize_whitespace(compile(src)),
            normalize_whitespace(vm_code)
        );
    }

    #[test]
    fn test_return() {
        let src = r#"
        class Test {
          function void test() {
            return 1;
            return;
          }
        }
        "#;

        let vm_code = r#"
        function Test.test 0
        push constant 1
        return
        push constant 0
        return
        "#;
        assert_eq!(
            normalize_whitespace(compile(src)),
            normalize_whitespace(vm_code)
        );
    }

    fn compile(jack_src: &str) -> String {
        let class_node = Parser::new(Tokenizer::new(jack_src)).parse().unwrap();
        Codegen::new().generate(&class_node).unwrap().into()
    }

    fn normalize_whitespace<S: AsRef<str>>(s: S) -> String {
        let lines = s
            .as_ref()
            .lines()
            .filter(|l| l.find(|c: char| !c.is_whitespace()).is_some());

        let min_indent = lines
            .clone()
            .map(|l| l.find(|c: char| !c.is_whitespace()).unwrap_or(0))
            .min()
            .unwrap_or(0);

        lines
            .map(|l| l[min_indent..].to_owned())
            .collect::<Vec<_>>()
            .join("\n")
    }
}
