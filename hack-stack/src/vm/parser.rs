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
            prev_token: Token {
                kind: Kind::Invalid(""),
                span: Span::new(0, 0),
            },
            peeked_token: None,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<ast::Instruction>, Vec<SpanError>> {
        let mut instructions = vec![];

        let mut errors = vec![];
        loop {
            match self.parse_instruction() {
                Ok(Some(instruction)) => instructions.push(instruction),
                Ok(None) => break,
                Err(e) => {
                    // When we get an error, skip to the next line to try to recover
                    while !matches!(self.token.kind, Kind::EOL | Kind::EOF) {
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

    fn parse_instruction(&mut self) -> ParseResult<Option<ast::Instruction>> {
        while matches!(self.token.kind, Kind::EOL | Kind::Comment(_)) {
            self.advance();
        }

        match self.token.kind {
            Kind::EOF => Ok(None),
            Kind::Instruction("push") => Ok(Some(self.parse_push()?)),
            Kind::Instruction("pop") => Ok(Some(self.parse_pop()?)),
            Kind::Instruction(_) => Ok(Some(self.parse_arithmetic_command()?)),
            _ => Err(self.unexpected_token_error("instruction")),
        }
    }

    fn parse_push(&mut self) -> ParseResult<ast::Instruction> {
        let start = self.token.span.start;
        self.expect(Kind::Instruction("push"))?;

        let segment = self.parse_segment()?;
        let offset = self.parse_number()?;
        let span = Span::new(start, self.prev_token.span.end);

        self.eat_terminator()?;
        Ok(ast::Instruction::Push(ast::PushInstruction {
            segment,
            offset,
            span,
        }))
    }

    fn parse_pop(&mut self) -> ParseResult<ast::Instruction> {
        let start = self.token.span.start;
        self.expect(Kind::Instruction("pop"))?;

        let segment = self.parse_segment()?;
        if segment == ast::Segment::Constant {
            return Err(self.span_error(
                "cannot pop to constant virtual memory segment".to_owned(),
                self.prev_token.span,
            ));
        }
        let offset = self.parse_number()?;
        let span = Span::new(start, self.prev_token.span.end);

        self.eat_terminator()?;
        Ok(ast::Instruction::Pop(ast::PopInstruction {
            segment,
            offset,
            span,
        }))
    }

    fn parse_segment(&mut self) -> ParseResult<ast::Segment> {
        if let Kind::Segment(name) = self.token.kind {
            let segment = match name {
                "constant" => ast::Segment::Constant,
                "local" => ast::Segment::Local,
                "argument" => ast::Segment::Argument,
                "static" => ast::Segment::Static,
                "this" => ast::Segment::This,
                "that" => ast::Segment::That,
                "temp" => ast::Segment::Temp,
                "pointer" => ast::Segment::Pointer,
                _ => return Err(self.unexpected_token_error("memory segment")),
            };
            self.advance();
            Ok(segment)
        } else {
            Err(self.unexpected_token_error("memory segment"))
        }
    }

    fn parse_arithmetic_command(&mut self) -> ParseResult<ast::Instruction> {
        if let Kind::Instruction(name) = self.token.kind {
            let instruction = match name {
                "add" => ast::Instruction::Add(self.token.span),
                "sub" => ast::Instruction::Sub(self.token.span),
                "eq" => ast::Instruction::Eq(self.token.span),
                "gt" => ast::Instruction::Gt(self.token.span),
                "lt" => ast::Instruction::Lt(self.token.span),
                "neg" => ast::Instruction::Neg(self.token.span),
                "and" => ast::Instruction::And(self.token.span),
                "or" => ast::Instruction::Or(self.token.span),
                "not" => ast::Instruction::Not(self.token.span),
                _ => return Err(self.unexpected_token_error("instruction")),
            };
            self.advance();
            self.eat_terminator()?;
            Ok(instruction)
        } else {
            Err(self.unexpected_token_error("instruction"))
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
                kind: Kind::EOL | Kind::EOF,
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
    fn test_push_pop() {
        let mut parser = Parser::new(Tokenizer::new("push local 1\npop static 3"));
        assert_eq!(
            parser.parse(),
            Ok(vec![
                ast::Instruction::Push(ast::PushInstruction {
                    segment: ast::Segment::Local,
                    offset: 1,
                    span: Span::new(0, 12)
                }),
                ast::Instruction::Pop(ast::PopInstruction {
                    segment: ast::Segment::Static,
                    offset: 3,
                    span: Span::new(13, 25)
                }),
            ])
        );

        let mut parser = Parser::new(Tokenizer::new("push local"));
        assert_eq!(
            parser.parse(),
            Err(vec![SpanError::new(
                String::from("unexpected token `<eof>', expected number"),
                Span::new(10, 10)
            )])
        );

        let mut parser = Parser::new(Tokenizer::new("pop constant 5"));
        assert_eq!(
            parser.parse(),
            Err(vec![SpanError::new(
                String::from("cannot pop to constant virtual memory segment"),
                Span::new(4, 12)
            )])
        );
    }

    #[test]
    fn test_simple_instructions() {
        let mut parser = Parser::new(Tokenizer::new("add // foo\nsub"));
        assert_eq!(
            parser.parse(),
            Ok(vec![
                ast::Instruction::Add(Span::new(0, 3)),
                ast::Instruction::Sub(Span::new(11, 14)),
            ])
        );

        let mut parser = Parser::new(Tokenizer::new("add constant 1"));
        assert_eq!(
            parser.parse(),
            Err(vec![SpanError::new(
                String::from("unexpected token `constant', expected newline"),
                Span::new(4, 12)
            )])
        );
    }
}
