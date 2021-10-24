use std::fmt::{self, Write};

use crate::common::Span;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Kind<'a> {
    Comment(&'a str),
    Number(&'a str),
    Identifier(&'a str),
    AtSign,
    Equals,
    Plus,
    Minus,
    Not,
    And,
    Or,
    Semicolon,
    LParen,
    RParen,
    EOL,
    EOF,
    Invalid(char),
}

impl<'a> fmt::Display for Kind<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            &Kind::Comment(v) => v,
            &Kind::Number(v) => v,
            &Kind::Identifier(v) => v,
            &Kind::AtSign => "@",
            &Kind::Equals => "=",
            &Kind::Plus => "+",
            &Kind::Minus => "-",
            &Kind::Not => "!",
            &Kind::And => "&",
            &Kind::Or => "|",
            &Kind::Semicolon => ";",
            &Kind::LParen => "(",
            &Kind::RParen => ")",
            &Kind::EOL => "<newline>",
            &Kind::EOF => "<eof>",
            &Kind::Invalid(c) => {
                return f.write_char(c);
            }
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
    pub fn from_char(pos: usize, c: char) -> Token<'a> {
        let kind = match c {
            '\n' => Kind::EOL,
            '@' => Kind::AtSign,
            '=' => Kind::Equals,
            '+' => Kind::Plus,
            '-' => Kind::Minus,
            '!' => Kind::Not,
            '&' => Kind::And,
            '|' => Kind::Or,
            ';' => Kind::Semicolon,
            '(' => Kind::LParen,
            ')' => Kind::RParen,
            v => Kind::Invalid(v),
        };
        let span = Span::new(pos, pos + 1);
        Token { kind, span }
    }

    pub fn eof(pos: usize) -> Token<'a> {
        Token {
            kind: Kind::EOF,
            span: Span::new(pos, pos),
        }
    }

    pub fn invalid(c: char, pos: usize) -> Token<'a> {
        Token {
            kind: Kind::Invalid(c),
            span: Span::new(pos, pos + 1),
        }
    }
}
