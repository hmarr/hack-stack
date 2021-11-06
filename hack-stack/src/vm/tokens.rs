use std::fmt;

use crate::common::Span;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Kind<'a> {
    Comment(&'a str),
    Instruction(&'a str),
    Segment(&'a str),
    Number(&'a str),
    Ident(&'a str),
    EOL,
    EOF,
    Invalid(&'a str),
}

impl<'a> fmt::Display for Kind<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            &Kind::Comment(v) => v,
            &Kind::Number(v) => v,
            &Kind::Instruction(v) => v,
            &Kind::Segment(v) => v,
            &Kind::Ident(v) => v,
            &Kind::EOL => "<newline>",
            &Kind::EOF => "<eof>",
            &Kind::Invalid(s) => s,
        };
        f.write_str(s)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Token<'a> {
    pub kind: Kind<'a>,
    pub span: Span,
}

impl<'a> Token<'a> {
    pub fn eof(pos: usize) -> Token<'a> {
        Token {
            kind: Kind::EOF,
            span: Span::new(pos, pos),
        }
    }
}
