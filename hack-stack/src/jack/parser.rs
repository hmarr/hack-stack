use super::ast::*;
use super::tokenizer::Tokenizer;
use super::tokens::{Kind, Token};
use crate::common::{Span, SpanError, Spanned};

type ParseResult<T> = Result<T, SpanError>;

pub struct Parser<'a> {
    tokenizer: Tokenizer<'a>,
    token: Token<'a>,
    prev_token: Token<'a>,
    peeked_token: Option<Token<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(mut tokenizer: Tokenizer<'a>) -> Parser<'a> {
        let mut token = tokenizer.next_token();
        while matches!(token.kind, Kind::Comment(_)) {
            token = tokenizer.next_token();
        }
        Parser {
            tokenizer,
            token,
            prev_token: Token::invalid("", 0),
            peeked_token: None,
        }
    }

    pub fn parse(&mut self) -> ParseResult<Class<'a>> {
        self.parse_class().and_then(|el| {
            if self.token.kind == Kind::EOF {
                Ok(el)
            } else {
                Err(self.unexpected_token_error("end of file"))
            }
        })
    }

    fn parse_class(&mut self) -> ParseResult<Class<'a>> {
        self.expect_keyword(&["class"])?;
        let name = self.expect_ident()?;
        self.expect_symbol("{")?;

        let mut var_decs: Vec<ClassVarDec<'a>> = Vec::new();
        let mut subroutine_decs: Vec<SubroutineDec<'a>> = Vec::new();
        loop {
            match self.token.kind {
                Kind::Keyword("field" | "static") => var_decs.push(self.parse_class_var_dec()?),
                Kind::Keyword("function" | "method" | "constructor") => {
                    subroutine_decs.push(self.parse_subroutine_dec()?);
                }
                _ => break,
            }
        }

        self.expect_symbol("}")?;

        Ok(Class {
            name,
            subroutine_decs,
            var_decs,
        })
    }

    fn parse_class_var_dec(&mut self) -> ParseResult<ClassVarDec<'a>> {
        let kind = match self.token.kind {
            Kind::Keyword("field") => ClassVarKind::Field,
            Kind::Keyword("static") => ClassVarKind::Static,
            _ => return Err(self.unexpected_token_error("`field' or `static'")),
        };

        Ok(ClassVarDec {
            kind,
            var_dec: self.parse_var_dec(&["field", "static"])?,
        })
    }

    fn parse_subroutine_dec(&mut self) -> ParseResult<SubroutineDec<'a>> {
        let kind = self
            .expect_keyword(&["function", "method", "constructor"])?
            .map(|&kind| match kind {
                "constructor" => SubroutineKind::Constructor,
                "function" => SubroutineKind::Function,
                "method" => SubroutineKind::Method,
                _ => unreachable!(),
            });

        let return_type = self.expect_type_name()?;
        let name = self.expect_ident()?;
        self.expect_symbol("(")?;
        let params = self.parse_parameter_list()?;
        self.expect_symbol(")")?;

        self.expect_symbol("{")?;
        let statements = self.parse_statements()?;
        self.expect_symbol("}")?;

        Ok(SubroutineDec {
            kind,
            name,
            params,
            return_type,
            statements,
        })
    }

    fn parse_parameter_list(&mut self) -> ParseResult<Vec<Param<'a>>> {
        let mut params: Vec<Param<'a>> = Vec::new();

        while !matches!(self.token.kind, Kind::Symbol(")")) {
            let ty = self.expect_type_name()?;
            let name = self.expect_ident()?;
            params.push(Param { ty, name });

            if matches!(self.token.kind, Kind::Symbol(",")) {
                self.advance();
            } else {
                break;
            }
        }

        Ok(params)
    }

    fn parse_statements(&mut self) -> ParseResult<Vec<Stmt<'a>>> {
        let mut stmts = Vec::new();

        loop {
            match self.token.kind {
                Kind::Keyword("var") => stmts.push(Stmt::Var(self.parse_var_dec(&["var"])?)),
                Kind::Keyword("let") => stmts.push(Stmt::Let(self.parse_let_statement()?)),
                Kind::Keyword("if") => stmts.push(Stmt::If(self.parse_if_statement()?)),
                Kind::Keyword("while") => stmts.push(Stmt::While(self.parse_while_statement()?)),
                Kind::Keyword("do") => stmts.push(Stmt::Do(self.parse_do_statement()?)),
                Kind::Keyword("return") => stmts.push(Stmt::Return(self.parse_return_statement()?)),
                _ => break,
            }
        }

        Ok(stmts)
    }

    fn parse_var_dec(&mut self, var_specifiers: &[&str]) -> ParseResult<VarDec<'a>> {
        self.expect_keyword(var_specifiers)?;

        let ty = self.expect_type_name()?;

        let mut names = vec![self.expect_ident()?];
        while matches!(self.token.kind, Kind::Symbol(",")) {
            self.advance();
            names.push(self.expect_ident()?);
        }

        self.expect_symbol(";")?;

        Ok(VarDec { ty, names })
    }

    fn parse_let_statement(&mut self) -> ParseResult<LetStmt<'a>> {
        self.expect_keyword(&["let"])?;

        let assignee = if matches!(self.peek().kind, Kind::Symbol("[")) {
            Assignee::Index(self.parse_index()?)
        } else {
            Assignee::Name(self.expect_ident()?)
        };

        self.expect_symbol("=")?;

        let expr = self.parse_expression(0)?;

        self.expect_symbol(";")?;

        Ok(LetStmt { assignee, expr })
    }

    fn parse_if_statement(&mut self) -> ParseResult<IfStmt<'a>> {
        self.expect_keyword(&["if"])?;
        self.expect_symbol("(")?;
        let cond = self.parse_expression(0)?;
        self.expect_symbol(")")?;

        self.expect_symbol("{")?;
        let if_arm = self.parse_statements()?;
        self.expect_symbol("}")?;

        let else_arm = match self.token.kind {
            Kind::Keyword("else") => {
                self.advance();
                self.expect_symbol("{")?;
                let stmts = self.parse_statements()?;
                self.expect_symbol("}")?;
                stmts
            }
            _ => vec![],
        };

        Ok(IfStmt {
            cond,
            if_arm,
            else_arm,
        })
    }

    fn parse_while_statement(&mut self) -> ParseResult<WhileStmt<'a>> {
        self.expect_keyword(&["while"])?;
        self.expect_symbol("(")?;
        let cond = self.parse_expression(0)?;
        self.expect_symbol(")")?;

        self.expect_symbol("{")?;
        let body = self.parse_statements()?;
        self.expect_symbol("}")?;

        Ok(WhileStmt { cond, body })
    }

    fn parse_do_statement(&mut self) -> ParseResult<SubroutineCall<'a>> {
        self.expect_keyword(&["do"])?;
        let call = self.parse_subroutine_call()?;
        self.expect_symbol(";")?;

        Ok(call)
    }

    fn parse_return_statement(&mut self) -> ParseResult<ReturnStmt<'a>> {
        self.expect_keyword(&["return"])?;

        let expr = match self.token.kind {
            Kind::Symbol(";") => None,
            _ => Some(self.parse_expression(0)?),
        };
        self.expect_symbol(";")?;

        Ok(ReturnStmt { expr })
    }

    fn parse_expression(&mut self, min_precedence: usize) -> ParseResult<Spanned<Box<Expr<'a>>>> {
        let span_start = self.token.span.start;
        let expr = match self.token.kind {
            Kind::IntConst(_) | Kind::StrConst(_) => self.parse_lit_expr()?,
            Kind::Keyword("true" | "false" | "null") => self.parse_lit_expr()?,
            Kind::Keyword("this") | Kind::Ident(_) => self.parse_name_expr()?,
            Kind::Symbol("-" | "~") => self.parse_unary_expr()?,
            Kind::Symbol("(") => self.parse_grouped_expr()?,
            _ => return Err(self.unexpected_token_error("term")),
        };
        let expr_span = Span::new(span_start, self.prev_token.span.end);
        let mut spanned_expr = Spanned {
            item: Box::new(expr),
            span: expr_span,
        };

        loop {
            let bin_op_kind = match self.token.kind {
                Kind::Symbol("+") => BinOpKind::Add,
                Kind::Symbol("-") => BinOpKind::Sub,
                Kind::Symbol("*") => BinOpKind::Mul,
                Kind::Symbol("/") => BinOpKind::Div,
                Kind::Symbol("&") => BinOpKind::And,
                Kind::Symbol("|") => BinOpKind::Or,
                Kind::Symbol("<") => BinOpKind::Lt,
                Kind::Symbol(">") => BinOpKind::Gt,
                Kind::Symbol("=") => BinOpKind::Eq,
                _ => return Ok(spanned_expr),
            };

            if bin_op_kind.precedence() < min_precedence {
                break;
            }

            let op_span = self.token.span;
            self.advance();

            let rhs = self.parse_expression(bin_op_kind.precedence())?;
            let span_end = rhs.span.end;

            spanned_expr = Spanned {
                item: Box::new(Expr::BinOp(BinOp {
                    lhs: spanned_expr,
                    op: Spanned {
                        item: bin_op_kind,
                        span: op_span,
                    },
                    rhs,
                })),
                span: Span::new(expr_span.start, span_end),
            };
        }

        Ok(spanned_expr)
    }

    fn parse_lit_expr(&mut self) -> ParseResult<Expr<'a>> {
        let expr = match self.token.kind {
            Kind::IntConst(_) => Expr::IntLit(self.token.to_spanned_str()),
            Kind::StrConst(_) => Expr::StrLit(self.token.to_spanned_str()),
            Kind::Keyword("true" | "false") => Expr::BoolLit(self.token.to_spanned_str()),
            Kind::Keyword("null") => Expr::NullLit(self.token.to_spanned_str()),
            _ => return Err(self.unexpected_token_error("literal")),
        };

        self.advance();
        Ok(expr)
    }

    fn parse_unary_expr(&mut self) -> ParseResult<Expr<'a>> {
        let op_kind = match self.token.kind.literal() {
            "-" => UnaryOpKind::Neg,
            "~" => UnaryOpKind::Not,
            _ => return Err(self.unexpected_token_error("unary operator")),
        };
        self.advance();

        let op_precedence = op_kind.precedence();
        Ok(Expr::UnaryOp(UnaryOp {
            op: Spanned {
                item: op_kind,
                span: self.prev_token.span,
            },
            expr: self.parse_expression(op_precedence)?,
        }))
    }

    fn parse_name_expr(&mut self) -> ParseResult<Expr<'a>> {
        match self.peek().kind {
            Kind::Symbol("[") => Ok(Expr::Index(self.parse_index()?)),
            Kind::Symbol("(" | ".") => Ok(Expr::SubroutineCall(self.parse_subroutine_call()?)),
            _ => {
                self.advance();
                Ok(Expr::Ident(self.prev_token.to_spanned_str()))
            }
        }
    }

    fn parse_subroutine_call(&mut self) -> ParseResult<SubroutineCall<'a>> {
        let (class, subroutine) = match self.peek().kind {
            Kind::Symbol("(") => {
                self.advance();
                (None, self.prev_token.to_spanned_str())
            }
            Kind::Symbol(".") => {
                let class = self.token;
                self.advance(); // class
                self.advance(); // .
                let subroutine = self.token;
                self.advance(); // subroutine
                (Some(class.to_spanned_str()), subroutine.to_spanned_str())
            }
            _ => return Err(self.unexpected_token_error("subroutine call")),
        };

        self.expect_symbol("(")?;
        let mut args: Vec<Spanned<Box<Expr<'a>>>> = Vec::new();
        while !matches!(self.token.kind, Kind::Symbol(")")) {
            args.push(self.parse_expression(0)?);

            if matches!(self.token.kind, Kind::Symbol(",")) {
                self.advance();
            } else {
                break;
            }
        }
        self.expect_symbol(")")?;

        Ok(SubroutineCall {
            class,
            subroutine,
            args,
        })
    }

    fn parse_index(&mut self) -> ParseResult<Index<'a>> {
        let array_name = self.token.to_spanned_str();
        self.advance();

        self.expect_symbol("[")?;
        let index = self.parse_expression(0)?;
        self.expect_symbol("]")?;

        Ok(Index { array_name, index })
    }

    fn parse_grouped_expr(&mut self) -> ParseResult<Expr<'a>> {
        self.expect_symbol("(")?;
        let expr = self.parse_expression(0)?;
        self.expect_symbol(")")?;

        Ok(*expr.item)
    }

    fn advance(&mut self) -> Token {
        self.prev_token = self.token;
        match self.peeked_token {
            Some(token) => {
                self.token = token;
                self.peeked_token = None;
            }
            None => {
                self.token = self.next_token();
            }
        }
        self.token
    }

    fn peek(&mut self) -> Token<'a> {
        match self.peeked_token {
            Some(token) => token,
            None => {
                self.peeked_token = Some(self.next_token());
                self.peeked_token.unwrap()
            }
        }
    }

    fn next_token(&mut self) -> Token<'a> {
        let mut token = self.tokenizer.next_token();
        while matches!(token.kind, Kind::Comment(_)) {
            token = self.tokenizer.next_token();
        }
        token
    }

    fn expect_type_name(&mut self) -> ParseResult<Spanned<&'a str>> {
        match self.token.kind {
            Kind::Keyword("void" | "int" | "char" | "boolean") | Kind::Ident(_) => {
                let tok = self.token;
                self.advance();
                Ok(Spanned {
                    item: tok.kind.literal(),
                    span: tok.span,
                })
            }
            _ => Err(self.unexpected_token_error("type")),
        }
    }

    fn expect_keyword(&mut self, allowed_values: &[&str]) -> ParseResult<Spanned<&'a str>> {
        match self.token {
            Token {
                kind: Kind::Keyword(lit),
                span,
            } if allowed_values.contains(&lit) => {
                self.advance();
                Ok(Spanned { item: lit, span })
            }
            _ => Err(self.unexpected_token_error(&format!("keyword {:?}", allowed_values))),
        }
    }

    fn expect_symbol(&mut self, sym_lit: &str) -> ParseResult<Spanned<&'a str>> {
        match self.token {
            Token {
                kind: Kind::Symbol(lit),
                span,
            } if sym_lit == lit => {
                self.advance();
                Ok(Spanned { item: lit, span })
            }
            _ => Err(self.unexpected_token_error(&format!("symbol {}", sym_lit))),
        }
    }

    fn expect_ident(&mut self) -> ParseResult<Spanned<&'a str>> {
        match self.token {
            Token {
                kind: Kind::Ident(lit),
                span,
            } => {
                self.advance();
                Ok(Spanned { item: lit, span })
            }
            _ => Err(self.unexpected_token_error(&format!("identifier"))),
        }
    }

    fn span_error(&self, msg: String, span: Span) -> SpanError {
        SpanError::new(msg, span)
    }

    fn unexpected_token_error(&self, expected: &str) -> SpanError {
        let msg = format!(
            "unexpected token `{}', expected {}",
            self.token.kind, expected
        );
        self.span_error(msg, self.token.span)
    }
}

#[cfg(test)]
mod tests {
    use crate::common::Spanned;

    use super::*;

    fn parse(src: &str) -> Class {
        Parser::new(Tokenizer::new(src)).parse().unwrap()
    }

    #[test]
    fn test_empty_class() {
        let src = "class Foo { }";
        let expected = Class {
            name: Spanned::void("Foo"),
            var_decs: vec![],
            subroutine_decs: vec![],
        };
        assert_eq!(parse(src), expected)
    }

    #[test]
    fn test_class_var_decs() {
        let src = "
        class Foo {
            field int x, y;
            static Point p;
        }
        ";
        let expected = Class {
            name: Spanned::void("Foo"),
            var_decs: vec![
                ClassVarDec {
                    kind: ClassVarKind::Field,
                    var_dec: VarDec {
                        ty: Spanned::void("int"),
                        names: vec![Spanned::void("x"), Spanned::void("y")],
                    },
                },
                ClassVarDec {
                    kind: ClassVarKind::Static,
                    var_dec: VarDec {
                        ty: Spanned::void("Point"),
                        names: vec![Spanned::void("p")],
                    },
                },
            ],
            subroutine_decs: vec![],
        };
        assert_eq!(parse(src), expected);
    }

    #[test]
    fn test_functions() {
        let src = "
        class Foo {
            constructor Foo new(boolean x) {
            }
            
            method void bar(int x, String y) {
            }
        }
        ";

        let expected = Class {
            name: Spanned::void("Foo"),
            var_decs: vec![],
            subroutine_decs: vec![
                SubroutineDec {
                    kind: Spanned::void(SubroutineKind::Constructor),
                    return_type: Spanned::void("Foo"),
                    name: Spanned::void("new"),
                    params: vec![Param {
                        ty: Spanned::void("boolean"),
                        name: Spanned::void("x"),
                    }],
                    statements: vec![],
                },
                SubroutineDec {
                    kind: Spanned::void(SubroutineKind::Method),
                    return_type: Spanned::void("void"),
                    name: Spanned::void("bar"),
                    params: vec![
                        Param {
                            ty: Spanned::void("int"),
                            name: Spanned::void("x"),
                        },
                        Param {
                            ty: Spanned::void("String"),
                            name: Spanned::void("y"),
                        },
                    ],
                    statements: vec![],
                },
            ],
        };
        assert_eq!(parse(src), expected);
    }

    #[test]
    fn test_statements() {
        let src = r#"
        class Foo {
            function integer bar() {
                var int x;
                var boolean y, z;
                if (true) {
                    let z = x;
                } else {
                    let a[0] = "baz";
                }
                do Sys.print("hi");
                while (false) {
                    return 1;
                }
                return;
            }
        }
        "#;
        let expected = Class {
            name: Spanned::void("Foo"),
            var_decs: vec![],
            subroutine_decs: vec![SubroutineDec {
                kind: Spanned::void(SubroutineKind::Function),
                return_type: Spanned::void("integer"),
                name: Spanned::void("bar"),
                params: vec![],
                statements: vec![
                    Stmt::Var(VarDec {
                        ty: Spanned::void("int"),
                        names: vec![Spanned::void("x")],
                    }),
                    Stmt::Var(VarDec {
                        ty: Spanned::void("boolean"),
                        names: vec![Spanned::void("y"), Spanned::void("z")],
                    }),
                    Stmt::If(IfStmt {
                        cond: Spanned::void(Box::new(Expr::BoolLit(Spanned::void("true")))),
                        if_arm: vec![Stmt::Let(LetStmt {
                            assignee: Assignee::Name(Spanned::void("z")),
                            expr: Spanned::void(Box::new(Expr::Ident(Spanned::void("x")))),
                        })],
                        else_arm: vec![Stmt::Let(LetStmt {
                            assignee: Assignee::Index(Index {
                                array_name: Spanned::void("a"),
                                index: Spanned::void(Box::new(Expr::IntLit(Spanned::void("0")))),
                            }),
                            expr: Spanned::void(Box::new(Expr::StrLit(Spanned::void("baz")))),
                        })],
                    }),
                    Stmt::Do(SubroutineCall {
                        class: Some(Spanned::void("Sys")),
                        subroutine: Spanned::void("print"),
                        args: vec![Spanned::void(Box::new(Expr::StrLit(Spanned::void("hi"))))],
                    }),
                    Stmt::While(WhileStmt {
                        cond: Spanned::void(Box::new(Expr::BoolLit(Spanned::void("false")))),
                        body: vec![Stmt::Return(ReturnStmt {
                            expr: Some(Spanned::void(Box::new(Expr::IntLit(Spanned::void("1"))))),
                        })],
                    }),
                    Stmt::Return(ReturnStmt { expr: None }),
                ],
            }],
        };
        assert_eq!(parse(src), expected);
    }

    #[test]
    fn test_expressions() {
        let src = "
        class Foo {
            function integer bar() {
                return 1;
                return ~1;
                return \"hello\";
                return true;
                return null;
                return this;
                return x;
                return x[1];
                return baz(1);
                return Foo.quux(1, false);
            }
        }
        ";
        let expected = Class {
            name: Spanned::void("Foo"),
            var_decs: vec![],
            subroutine_decs: vec![SubroutineDec {
                kind: Spanned::void(SubroutineKind::Function),
                return_type: Spanned::void("integer"),
                name: Spanned::void("bar"),
                params: vec![],
                statements: vec![
                    Stmt::Return(ReturnStmt {
                        expr: Some(Spanned::void(Box::new(Expr::IntLit(Spanned::void("1"))))),
                    }),
                    Stmt::Return(ReturnStmt {
                        expr: Some(Spanned::void(Box::new(Expr::UnaryOp(UnaryOp {
                            op: Spanned::void(UnaryOpKind::Not),
                            expr: Spanned::void(Box::new(Expr::IntLit(Spanned::void("1")))),
                        })))),
                    }),
                    Stmt::Return(ReturnStmt {
                        expr: Some(Spanned::void(Box::new(Expr::StrLit(Spanned::void(
                            "hello",
                        ))))),
                    }),
                    Stmt::Return(ReturnStmt {
                        expr: Some(Spanned::void(Box::new(Expr::BoolLit(Spanned::void(
                            "true",
                        ))))),
                    }),
                    Stmt::Return(ReturnStmt {
                        expr: Some(Spanned::void(Box::new(Expr::NullLit(Spanned::void(
                            "null",
                        ))))),
                    }),
                    Stmt::Return(ReturnStmt {
                        expr: Some(Spanned::void(Box::new(Expr::Ident(Spanned::void("this"))))),
                    }),
                    Stmt::Return(ReturnStmt {
                        expr: Some(Spanned::void(Box::new(Expr::Ident(Spanned::void("x"))))),
                    }),
                    Stmt::Return(ReturnStmt {
                        expr: Some(Spanned::void(Box::new(Expr::Index(Index {
                            array_name: Spanned::void("x"),
                            index: Spanned::void(Box::new(Expr::IntLit(Spanned::void("1")))),
                        })))),
                    }),
                    Stmt::Return(ReturnStmt {
                        expr: Some(Spanned::void(Box::new(Expr::SubroutineCall(
                            SubroutineCall {
                                class: None,
                                subroutine: Spanned::void("baz"),
                                args: vec![Spanned::void(Box::new(Expr::IntLit(Spanned::void(
                                    "1",
                                ))))],
                            },
                        )))),
                    }),
                    Stmt::Return(ReturnStmt {
                        expr: Some(Spanned::void(Box::new(Expr::SubroutineCall(
                            SubroutineCall {
                                class: Some(Spanned::void("Foo")),
                                subroutine: Spanned::void("quux"),
                                args: vec![
                                    Spanned::void(Box::new(Expr::IntLit(Spanned::void("1")))),
                                    Spanned::void(Box::new(Expr::BoolLit(Spanned::void("false")))),
                                ],
                            },
                        )))),
                    }),
                ],
            }],
        };
        assert_eq!(parse(src), expected);
    }

    #[test]
    fn test_expression_precedence() {
        let src = "
        class Foo {
            function integer bar() {
                return -1 + 2 & 3 * -4;
                return -(1 + 2) * 3;
            }
        }
        ";
        let expected = Class {
            name: Spanned::void("Foo"),
            var_decs: vec![],
            subroutine_decs: vec![SubroutineDec {
                kind: Spanned::void(SubroutineKind::Function),
                return_type: Spanned::void("integer"),
                name: Spanned::void("bar"),
                params: vec![],
                statements: vec![
                    Stmt::Return(ReturnStmt {
                        expr: Some(Spanned::void(Box::new(Expr::BinOp(BinOp {
                            op: Spanned::void(BinOpKind::Add),
                            lhs: Spanned::void(Box::new(Expr::UnaryOp(UnaryOp {
                                op: Spanned::void(UnaryOpKind::Neg),
                                expr: Spanned::void(Box::new(Expr::IntLit(Spanned::void("1")))),
                            }))),
                            rhs: Spanned::void(Box::new(Expr::BinOp(BinOp {
                                op: Spanned::void(BinOpKind::Mul),
                                lhs: Spanned::void(Box::new(Expr::BinOp(BinOp {
                                    op: Spanned::void(BinOpKind::And),
                                    lhs: Spanned::void(Box::new(Expr::IntLit(Spanned::void("2")))),
                                    rhs: Spanned::void(Box::new(Expr::IntLit(Spanned::void("3")))),
                                }))),
                                rhs: Spanned::void(Box::new(Expr::UnaryOp(UnaryOp {
                                    op: Spanned::void(UnaryOpKind::Neg),
                                    expr: Spanned::void(Box::new(Expr::IntLit(Spanned::void("4")))),
                                }))),
                            }))),
                        })))),
                    }),
                    Stmt::Return(ReturnStmt {
                        expr: Some(Spanned::void(Box::new(Expr::BinOp(BinOp {
                            op: Spanned::void(BinOpKind::Mul),
                            lhs: Spanned::void(Box::new(Expr::UnaryOp(UnaryOp {
                                op: Spanned::void(UnaryOpKind::Neg),
                                expr: Spanned::void(Box::new(Expr::BinOp(BinOp {
                                    op: Spanned::void(BinOpKind::Add),
                                    lhs: Spanned::void(Box::new(Expr::IntLit(Spanned::void("1")))),
                                    rhs: Spanned::void(Box::new(Expr::IntLit(Spanned::void("2")))),
                                }))),
                            }))),
                            rhs: Spanned::void(Box::new(Expr::IntLit(Spanned::void("3")))),
                        })))),
                    }),
                ],
            }],
        };
        assert_eq!(parse(src), expected);
    }
}
