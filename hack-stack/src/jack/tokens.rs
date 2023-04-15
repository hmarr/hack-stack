use std::fmt;

use crate::common::{Span, Spanned};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Kind<'a> {
    Symbol(&'a str),
    Keyword(&'a str),
    Ident(&'a str),
    IntConst(&'a str),
    StrConst(&'a str),
    Comment(&'a str),
    EOF,
    Invalid(&'a str),
}

impl<'a> Kind<'a> {
    pub fn literal(&self) -> &'a str {
        match *self {
            Kind::Symbol(v) => v,
            Kind::Keyword(v) => v,
            Kind::Ident(v) => v,
            Kind::IntConst(v) => v,
            Kind::StrConst(v) => v,
            Kind::Comment(v) => v,
            Kind::EOF => "EOF",
            Kind::Invalid(s) => s,
        }
    }
}

impl<'a> fmt::Display for Kind<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.literal())
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

    pub fn invalid(s: &'a str, pos: usize) -> Token<'a> {
        Token {
            kind: Kind::Invalid(s),
            span: Span::new(pos, pos + 1),
        }
    }

    pub fn to_spanned_str(&self) -> Spanned<&'a str> {
        Spanned {
            item: self.kind.literal(),
            span: self.span,
        }
    }
}
