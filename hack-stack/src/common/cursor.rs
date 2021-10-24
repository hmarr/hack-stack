use std::{iter::Peekable, str::Chars};

use super::Span;

pub const EOF_CHAR: char = '\0';

pub struct Cursor<'a> {
    pub pos: usize,
    pub c: char,
    src_iter: Peekable<Chars<'a>>,
}

impl<'a> Cursor<'a> {
    pub fn new(src: &'a str) -> Cursor<'a> {
        let mut src_iter = src.chars().peekable();
        let c = src_iter.next().unwrap_or(EOF_CHAR);
        Cursor {
            src_iter,
            pos: 0,
            c,
        }
    }

    pub fn advance(&mut self) -> char {
        match self.src_iter.next() {
            Some(c) => {
                self.pos += self.c.len_utf8();
                self.c = c;
            }
            None => {
                self.c = EOF_CHAR;
            }
        }
        self.c
    }

    pub fn peek(&mut self) -> char {
        *self.src_iter.peek().unwrap_or(&EOF_CHAR)
    }

    pub fn eat_while(&mut self, cond: fn(char) -> bool) -> Span {
        let start = self.pos;
        let mut length = if cond(self.c) {
            self.c.len_utf8()
        } else {
            return Span::new(start, start);
        };

        while cond(self.peek()) {
            length += match self.advance() {
                EOF_CHAR => 0,
                c => c.len_utf8(),
            }
        }
        Span::new(start, start + length)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor() {
        todo!();
    }
}
