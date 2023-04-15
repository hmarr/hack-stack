use std::convert::TryFrom;

use super::ast;
use super::tokenizer::Tokenizer;
use super::tokens::{Kind, Token};
use crate::common::{Span, SpanError};

type ParseResult<T> = Result<T, SpanError>;

pub struct Parser<'a> {
    tokenizer: Tokenizer<'a>,
    token: Token<'a>,
    prev_token: Token<'a>,
    peeked_token: Option<Token<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(mut tokenizer: Tokenizer<'a>) -> Parser<'a> {
        let token = tokenizer.next_token();
        Parser {
            tokenizer,
            token,
            prev_token: Token::invalid('\0', 0),
            peeked_token: None,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<ast::Instruction<'a>>, Vec<SpanError>> {
        let mut instructions = vec![];

        let mut errors = vec![];
        loop {
            match self.parse_instruction() {
                Ok(Some(instruction)) => instructions.push(instruction),
                Ok(None) => break,
                Err(e) => {
                    // When we get an error, skip to the next line to try to recover
                    while !matches!(self.token.kind, Kind::Eol | Kind::Eof) {
                        self.advance();
                    }
                    errors.push(e)
                }
            }
        }

        if errors.is_empty() {
            Ok(instructions)
        } else {
            Err(errors)
        }
    }

    fn parse_instruction(&mut self) -> ParseResult<Option<ast::Instruction<'a>>> {
        while matches!(self.token.kind, Kind::Eol | Kind::Comment(_)) {
            self.advance();
        }

        match self.token.kind {
            Kind::Eof => Ok(None),
            Kind::LParen => Ok(Some(self.parse_label()?)),
            Kind::AtSign => Ok(Some(self.parse_a_instruction()?)),
            Kind::Number(_) | Kind::Identifier(_) | Kind::Minus | Kind::Not => {
                Ok(Some(self.parse_c_instruction()?))
            }
            _ => Err(self.unexpected_token_error("instruction")),
        }
    }

    fn parse_label(&mut self) -> ParseResult<ast::Instruction<'a>> {
        let start = self.token.span.start;
        self.expect(Kind::LParen)?;

        if let Kind::Identifier(name) = self.token.kind {
            self.advance();

            let span = Span::new(start, self.token.span.end);
            let label = ast::Instruction::Label(ast::Label { name, span });

            self.expect(Kind::RParen)?;
            self.eat_terminator()?;

            Ok(label)
        } else {
            Err(self.unexpected_token_error("label name"))
        }
    }

    fn parse_a_instruction(&mut self) -> ParseResult<ast::Instruction<'a>> {
        let start = self.token.span.start;
        self.expect(Kind::AtSign)?;

        let span = Span::new(start, self.token.span.end);
        let addr = match self.token.kind {
            Kind::Number(_) => {
                let num = self.parse_number()?;
                // The instruction uses 1 bit so we only have 15 bits available to use
                if num >= 0x8000 {
                    return Err(self.span_error(
                        format!("number {} outside range 0-32767", num),
                        self.prev_token.span,
                    ));
                }
                ast::Address::Value(num)
            }
            Kind::Identifier(s) => {
                self.advance();
                ast::Address::Symbol(s)
            }
            _ => return Err(self.unexpected_token_error("number or symbol")),
        };

        self.eat_terminator()?;
        Ok(ast::Instruction::A(ast::AInstruction { addr, span }))
    }

    fn parse_c_instruction(&mut self) -> ParseResult<ast::Instruction<'a>> {
        let start = self.token.span.start;
        let dest = self.parse_dest()?;
        let comp = self.parse_comp()?;
        let jump = self.parse_jump()?;
        let span = Span::new(start, self.prev_token.span.end);

        self.eat_terminator()?;

        Ok(ast::Instruction::C(ast::CInstruction {
            dest,
            comp,
            jump,
            span,
        }))
    }

    fn parse_dest(&mut self) -> ParseResult<Option<ast::Dest>> {
        if self.peek().kind != Kind::Equals {
            return Ok(None);
        }

        if let Kind::Identifier(ident) = self.token.kind {
            let dest = ast::Dest::try_from(ident).map_err(|e| self.error(e))?;
            self.advance();
            self.expect(Kind::Equals)?;
            Ok(Some(dest))
        } else {
            Err(self.error(format!(
                "expected destination to be register, found `{}'",
                self.token.kind
            )))
        }
    }

    fn parse_comp(&mut self) -> ParseResult<ast::Comp> {
        if self.token.kind == Kind::Not || self.token.kind == Kind::Minus {
            return Ok(ast::Comp::UnaryOperation(self.parse_unary_operation()?));
        }

        match self.peek().kind {
            Kind::Plus | Kind::Minus | Kind::And | Kind::Or => {
                Ok(ast::Comp::BinaryOperation(self.parse_binary_operation()?))
            }
            _ => match self.token.kind {
                Kind::Identifier(_) => Ok(ast::Comp::Register(self.parse_register()?)),
                Kind::Number(_) => Ok(ast::Comp::Bit(self.parse_bit()?)),
                _ => Err(self.unexpected_token_error("computation operation")),
            },
        }
    }

    fn parse_jump(&mut self) -> ParseResult<Option<ast::Jump>> {
        if !self.eat(Kind::Semicolon) {
            return Ok(None);
        }

        if let Kind::Identifier(ident) = self.token.kind {
            let jump = ast::Jump::try_from(ident).map_err(|e| self.error(e))?;
            self.advance();
            Ok(Some(jump))
        } else {
            Ok(None)
        }
    }

    fn parse_unary_operation(&mut self) -> ParseResult<ast::UnaryOperation> {
        let op = match self.token.kind {
            Kind::Not => ast::UnaryOperator::Not,
            Kind::Minus => ast::UnaryOperator::Minus,
            _ => {
                return Err(self.error(format!(
                    "invalid unary operator {}, expected ! or -",
                    self.token.kind
                )))
            }
        };

        self.advance();
        let operand = self.parse_unary_operand()?;
        Ok(ast::UnaryOperation { op, operand })
    }

    fn parse_unary_operand(&mut self) -> ParseResult<ast::Operand> {
        match self.token.kind {
            Kind::Number(_) => Ok(ast::Operand::Bit(self.parse_bit()?)),
            Kind::Identifier(_) => Ok(ast::Operand::Register(self.parse_register()?)),
            _ => Err(self.error(format!(
                "invalid unary operand {}, expected 0, 1, or register",
                self.token.kind
            ))),
        }
    }

    fn parse_binary_operation(&mut self) -> ParseResult<ast::BinaryOperation> {
        let lhs = self.parse_register()?;

        let op = match self.token.kind {
            Kind::Plus => ast::BinaryOperator::Plus,
            Kind::Minus => ast::BinaryOperator::Minus,
            Kind::And => ast::BinaryOperator::And,
            Kind::Or => ast::BinaryOperator::Or,
            _ => return Err(self.unexpected_token_error("+, -, &, or |")),
        };
        self.advance();

        let rhs = match self.token.kind {
            Kind::Number(_) => ast::Operand::Bit(self.parse_bit()?),
            Kind::Identifier(_) => ast::Operand::Register(self.parse_register()?),
            _ => return Err(self.unexpected_token_error("0, 1, or register")),
        };

        Ok(ast::BinaryOperation { lhs, op, rhs })
    }

    fn parse_bit(&mut self) -> ParseResult<ast::Bit> {
        if let Kind::Number(n) = self.token.kind {
            let bit = ast::Bit::try_from(n).map_err(|e| self.error(e))?;
            self.advance();
            Ok(bit)
        } else {
            Err(self.unexpected_token_error("0 or 1"))
        }
    }

    fn parse_register(&mut self) -> ParseResult<ast::Register> {
        if let Kind::Identifier(i) = self.token.kind {
            let bit = ast::Register::try_from(i).map_err(|e| self.error(e))?;
            self.advance();
            Ok(bit)
        } else {
            Err(self.unexpected_token_error("D, A, or M"))
        }
    }

    fn parse_number(&mut self) -> ParseResult<u16> {
        let num = self.token_to_number()?;
        self.advance();
        Ok(num)
    }

    fn token_to_number(&mut self) -> ParseResult<u16> {
        match self.token {
            Token {
                kind: Kind::Number(n),
                ..
            } => n
                .parse::<u16>()
                .map_err(|_| self.error(format!("invalid number {}", n))),
            _ => Err(self.unexpected_token_error("number")),
        }
    }

    fn expect(&mut self, kind: Kind) -> ParseResult<()> {
        if !self.eat(kind) {
            Err(self.unexpected_token_error(&format!("{:?}", kind)))
        } else {
            Ok(())
        }
    }

    fn eat(&mut self, kind: Kind) -> bool {
        match self.token {
            token if token.kind == kind => {
                self.advance();
                true
            }
            _ => false,
        }
    }

    fn eat_terminator(&mut self) -> ParseResult<()> {
        match self.token {
            Token {
                kind: Kind::Eol | Kind::Eof,
                ..
            } => {
                self.advance();
                Ok(())
            }
            _ => Err(self.unexpected_token_error("newline")),
        }
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

    fn span_error(&self, msg: String, span: Span) -> SpanError {
        SpanError::new(msg, span)
    }

    fn error(&self, msg: String) -> SpanError {
        self.span_error(msg, self.token.span)
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
    use crate::common::Span;

    use super::*;

    #[test]
    fn test_empty_string() {
        let mut parser = Parser::new(Tokenizer::new(""));
        assert_eq!(parser.parse(), Ok(vec![]));
    }

    #[test]
    fn test_label() {
        let mut parser = Parser::new(Tokenizer::new("(LOOP)\n(END)"));
        assert_eq!(
            parser.parse(),
            Ok(vec![
                ast::Instruction::Label(ast::Label {
                    name: "LOOP",
                    span: Span::new(0, 6)
                }),
                ast::Instruction::Label(ast::Label {
                    name: "END",
                    span: Span::new(7, 12)
                })
            ])
        );

        let mut parser = Parser::new(Tokenizer::new("(123)"));
        assert_eq!(
            parser.parse(),
            Err(vec![SpanError::new(
                String::from("unexpected token `123', expected label name"),
                Span::new(1, 4)
            )])
        );
    }

    #[test]
    fn test_a_instruction() {
        let mut parser = Parser::new(Tokenizer::new("@123"));
        assert_eq!(
            parser.parse(),
            Ok(vec![ast::Instruction::A(ast::AInstruction {
                addr: ast::Address::Value(123),
                span: Span::new(0, 4)
            })])
        );

        let mut parser = Parser::new(Tokenizer::new("@LOOP"));
        assert_eq!(
            parser.parse(),
            Ok(vec![ast::Instruction::A(ast::AInstruction {
                addr: ast::Address::Symbol("LOOP"),
                span: Span::new(0, 5)
            }),])
        );

        let mut parser = Parser::new(Tokenizer::new("@123 @456"));
        assert_eq!(
            parser.parse(),
            Err(vec![SpanError::new(
                String::from("unexpected token `@', expected newline"),
                Span::new(5, 6)
            )])
        );

        let mut parser = Parser::new(Tokenizer::new("@32768"));
        assert_eq!(
            parser.parse(),
            Err(vec![SpanError::new(
                String::from("number 32768 outside range 0-32767"),
                Span::new(1, 6)
            )])
        );
    }

    #[test]
    fn test_valid_c_instructions() {
        let mut parser = Parser::new(Tokenizer::new("1"));
        assert_eq!(
            parser.parse(),
            Ok(vec![ast::Instruction::C(ast::CInstruction {
                dest: None,
                comp: ast::Comp::Bit(ast::Bit::One),
                jump: None,
                span: Span::new(0, 1)
            }),])
        );

        let mut parser = Parser::new(Tokenizer::new("D=1"));
        assert_eq!(
            parser.parse(),
            Ok(vec![ast::Instruction::C(ast::CInstruction {
                dest: Some(ast::Dest {
                    d: true,
                    a: false,
                    m: false
                }),
                comp: ast::Comp::Bit(ast::Bit::One),
                jump: None,
                span: Span::new(0, 3)
            }),])
        );

        let mut parser = Parser::new(Tokenizer::new("!M"));
        assert_eq!(
            parser.parse(),
            Ok(vec![ast::Instruction::C(ast::CInstruction {
                dest: None,
                comp: ast::Comp::UnaryOperation(ast::UnaryOperation {
                    op: ast::UnaryOperator::Not,
                    operand: ast::Operand::Register(ast::Register::M)
                }),
                jump: None,
                span: Span::new(0, 2)
            }),])
        );

        let mut parser = Parser::new(Tokenizer::new("D|0"));
        assert_eq!(
            parser.parse(),
            Ok(vec![ast::Instruction::C(ast::CInstruction {
                dest: None,
                comp: ast::Comp::BinaryOperation(ast::BinaryOperation {
                    lhs: ast::Register::D,
                    op: ast::BinaryOperator::Or,
                    rhs: ast::Operand::Bit(ast::Bit::Zero),
                }),
                jump: None,
                span: Span::new(0, 3)
            }),])
        );

        let mut parser = Parser::new(Tokenizer::new("M=!A"));
        assert_eq!(
            parser.parse(),
            Ok(vec![ast::Instruction::C(ast::CInstruction {
                dest: Some(ast::Dest {
                    d: false,
                    a: false,
                    m: true
                }),
                comp: ast::Comp::UnaryOperation(ast::UnaryOperation {
                    op: ast::UnaryOperator::Not,
                    operand: ast::Operand::Register(ast::Register::A),
                }),
                jump: None,
                span: Span::new(0, 4)
            }),])
        );

        let mut parser = Parser::new(Tokenizer::new("A=M+1"));
        assert_eq!(
            parser.parse(),
            Ok(vec![ast::Instruction::C(ast::CInstruction {
                dest: Some(ast::Dest {
                    d: false,
                    a: true,
                    m: false
                }),
                comp: ast::Comp::BinaryOperation(ast::BinaryOperation {
                    lhs: ast::Register::M,
                    op: ast::BinaryOperator::Plus,
                    rhs: ast::Operand::Bit(ast::Bit::One),
                }),
                jump: None,
                span: Span::new(0, 5)
            }),])
        );

        let mut parser = Parser::new(Tokenizer::new("AM=0"));
        assert_eq!(
            parser.parse(),
            Ok(vec![ast::Instruction::C(ast::CInstruction {
                dest: Some(ast::Dest {
                    d: false,
                    a: true,
                    m: true
                }),
                comp: ast::Comp::Bit(ast::Bit::Zero),
                jump: None,
                span: Span::new(0, 4)
            }),])
        );

        let mut parser = Parser::new(Tokenizer::new("0;JMP"));
        assert_eq!(
            parser.parse(),
            Ok(vec![ast::Instruction::C(ast::CInstruction {
                dest: None,
                comp: ast::Comp::Bit(ast::Bit::Zero),
                jump: Some(ast::Jump::JMP),
                span: Span::new(0, 5)
            }),])
        );

        let mut parser = Parser::new(Tokenizer::new("DA = M & 1 ; JGT "));
        assert_eq!(
            parser.parse(),
            Ok(vec![ast::Instruction::C(ast::CInstruction {
                dest: Some(ast::Dest {
                    d: true,
                    a: true,
                    m: false
                }),
                comp: ast::Comp::BinaryOperation(ast::BinaryOperation {
                    lhs: ast::Register::M,
                    op: ast::BinaryOperator::And,
                    rhs: ast::Operand::Bit(ast::Bit::One),
                }),
                jump: Some(ast::Jump::JGT),
                span: Span::new(0, 16)
            }),])
        );
    }

    #[test]
    fn test_invalid_c_instructions() {
        let mut parser = Parser::new(Tokenizer::new("1=1"));
        assert_eq!(
            parser.parse(),
            Err(vec![SpanError::new(
                String::from("expected destination to be register, found `1'"),
                Span::new(0, 1)
            )])
        );

        let mut parser = Parser::new(Tokenizer::new("D1=1"));
        assert_eq!(
            parser.parse(),
            Err(vec![SpanError::new(
                String::from("invalid destination D1, expected a combination of A, D and M"),
                Span::new(0, 2)
            )])
        );
    }

    #[test]
    fn test_multiple_instructions() {
        let mut parser = Parser::new(Tokenizer::new("@123\nA=M+1//foo"));
        assert_eq!(
            parser.parse(),
            Ok(vec![
                ast::Instruction::A(ast::AInstruction {
                    addr: ast::Address::Value(123),
                    span: Span::new(0, 4)
                }),
                ast::Instruction::C(ast::CInstruction {
                    dest: Some(ast::Dest {
                        d: false,
                        a: true,
                        m: false
                    }),
                    comp: ast::Comp::BinaryOperation(ast::BinaryOperation {
                        lhs: ast::Register::M,
                        op: ast::BinaryOperator::Plus,
                        rhs: ast::Operand::Bit(ast::Bit::One),
                    }),
                    jump: None,
                    span: Span::new(5, 10)
                }),
            ])
        );
    }
}
