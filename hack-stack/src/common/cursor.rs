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
                if self.c != EOF_CHAR {
                    self.pos += self.c.len_utf8();
                    self.c = EOF_CHAR;
                }
            }
        }
        self.c
    }

    pub fn peek(&mut self) -> char {
        *self.src_iter.peek().unwrap_or(&EOF_CHAR)
    }

    pub fn eat_while(&mut self, cond: fn(char) -> bool) -> Span {
        let start = self.pos;
        let mut length = 0;

        while cond(self.c) {
            length += self.c.len_utf8();
            self.advance();
        }
        Span::new(start, start + length)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor() {
        let src = "soñar";
        let mut cursor = Cursor::new(src);
        assert_eq!(cursor.c, 's');
        assert_eq!(cursor.pos, 0);

        assert_eq!(cursor.advance(), 'o');
        assert_eq!(cursor.advance(), 'ñ');
        assert_eq!(cursor.c, 'ñ');
        assert_eq!(cursor.pos, 2);

        assert_eq!(cursor.advance(), 'a');
        assert_eq!(cursor.c, 'a');
        assert_eq!(cursor.pos, 4);

        assert_eq!(cursor.advance(), 'r');
        assert_eq!(cursor.pos, 5);

        assert_eq!(cursor.advance(), EOF_CHAR);
        assert_eq!(cursor.c, EOF_CHAR);
        assert_eq!(cursor.pos, 6);
    }
}
