mod span;
mod token;

pub use span::Span;
use std::{iter::Peekable, str::Chars};
pub use token::{Kind, Token};

pub const EOF_CHAR: char = '\0';

pub struct Tokenizer<'a> {
    src: &'a str,
    src_iter: Peekable<Chars<'a>>,
    pos: usize,
    c: char,
    lines: Vec<usize>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(src: &'a str) -> Tokenizer<'a> {
        let mut iter = src.chars().peekable();
        let c = iter.next().unwrap_or(EOF_CHAR);
        Tokenizer {
            src,
            src_iter: iter,
            pos: 0,
            c,
            lines: vec![],
        }
    }

    pub fn next_token(&mut self) -> Token<'a> {
        self.eat_whitespace();

        let token = match self.c {
            '\n' | '@' | '=' | '+' | '-' | '&' | '|' | '!' | ';' | '(' | ')' => {
                Token::from_char(self.pos, self.c)
            }
            '/' => match self.peek() {
                '/' => self.tokenize_comment(),
                c => {
                    self.advance();
                    Token::invalid(c, self.pos)
                }
            },
            c if ident_start_char(c) => self.tokenize_identifier(),
            '0'..='9' => self.tokenize_number(),
            EOF_CHAR => Token::eof(self.pos + 1),
            c => Token::invalid(c, self.pos),
        };

        self.advance();

        token
    }

    pub fn loc_for_byte_pos(&self, pos: usize) -> (usize, usize) {
        let mut line_start = 0;
        for (line, &next_newline) in self.lines.iter().enumerate() {
            if next_newline >= pos {
                let char_pos = self.src[line_start..pos].chars().count() + 1;
                return (line + 1, char_pos);
            }
            line_start = next_newline + 1;
        }

        let char_pos = self.src[line_start..pos].chars().count() + 1;
        (self.lines.len() + 1, char_pos)
    }

    fn advance(&mut self) -> char {
        match self.src_iter.next() {
            Some(c) => {
                if self.c == '\n' {
                    self.lines.push(self.pos);
                }
                self.pos += self.c.len_utf8();
                self.c = c;
            }
            None => {
                self.c = EOF_CHAR;
            }
        }
        self.c
    }

    fn peek(&mut self) -> char {
        *self.src_iter.peek().unwrap_or(&EOF_CHAR)
    }

    fn tokenize_number(&mut self) -> Token<'a> {
        let start = self.pos;
        let mut length = self.c.len_utf8();

        while self.peek().is_numeric() {
            self.advance();
            length += self.c.len_utf8();
        }

        let span = Span::new(start, start + length);
        Token {
            kind: Kind::Number(&self.src[span.start..span.end]),
            span,
        }
    }

    fn tokenize_identifier(&mut self) -> Token<'a> {
        let start = self.pos;
        let mut length = self.c.len_utf8();

        while ident_char(self.peek()) {
            self.advance();
            length += self.c.len_utf8();
        }

        let span = Span::new(start, start + length);
        Token {
            kind: Kind::Identifier(&self.src[span.start..span.end]),
            span,
        }
    }

    fn tokenize_comment(&mut self) -> Token<'a> {
        let start = self.pos;
        let mut length = self.c.len_utf8();

        while self.peek() != '\n' && self.peek() != EOF_CHAR {
            self.advance();
            length += self.c.len_utf8();
        }

        let span = Span::new(start, start + length);
        Token {
            kind: Kind::Comment(&self.src[span.start..span.end]),
            span,
        }
    }

    fn eat_whitespace(&mut self) {
        while self.c.is_whitespace() && self.c != '\n' {
            self.advance();
        }
    }
}

fn ident_char(c: char) -> bool {
    match c {
        c if c.is_alphanumeric() => true,
        '_' | '.' | '$' | ':' => true,
        _ => false,
    }
}
fn ident_start_char(c: char) -> bool {
    match c {
        c if c.is_alphabetic() => true,
        '_' | '.' | '$' | ':' => true,
        _ => false,
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_token() {
            Token {
                kind: Kind::EOF, ..
            } => None,
            token => Some(token),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokenize(s: &str) -> Vec<Token> {
        Tokenizer::new(s).collect()
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(tokenize(""), vec![]);
    }

    #[test]
    fn test_spans() {
        assert_eq!(
            tokenize(" @café 1"),
            vec![
                Token {
                    kind: Kind::AtSign,
                    span: Span::new(1, 2)
                },
                Token {
                    kind: Kind::Identifier("café"),
                    span: Span::new(2, 7)
                },
                Token {
                    kind: Kind::Number("1"),
                    span: Span::new(8, 9)
                },
            ]
        );
    }

    #[test]
    fn test_a_instructions() {
        let tokens = tokenize("@0 @-123\n  @my$var");
        assert_eq!(
            tokens.iter().map(|t| t.kind).collect::<Vec<Kind>>(),
            vec![
                Kind::AtSign,
                Kind::Number("0"),
                Kind::AtSign,
                Kind::Minus,
                Kind::Number("123"),
                Kind::EOL,
                Kind::AtSign,
                Kind::Identifier("my$var"),
            ]
        );
    }

    #[test]
    fn test_c_instructions() {
        let tokens = tokenize("D=-1 M=!A D=M|A 0;JMP");
        assert_eq!(
            tokens.iter().map(|t| t.kind).collect::<Vec<Kind>>(),
            vec![
                Kind::Identifier("D"),
                Kind::Equals,
                Kind::Minus,
                Kind::Number("1"),
                Kind::Identifier("M"),
                Kind::Equals,
                Kind::Not,
                Kind::Identifier("A"),
                Kind::Identifier("D"),
                Kind::Equals,
                Kind::Identifier("M"),
                Kind::Or,
                Kind::Identifier("A"),
                Kind::Number("0"),
                Kind::Semicolon,
                Kind::Identifier("JMP"),
            ]
        );
    }

    #[test]
    fn test_labels() {
        let tokens = tokenize("(LOOP)\n@1\n (END) ");
        assert_eq!(
            tokens.iter().map(|t| t.kind).collect::<Vec<Kind>>(),
            vec![
                Kind::LParen,
                Kind::Identifier("LOOP"),
                Kind::RParen,
                Kind::EOL,
                Kind::AtSign,
                Kind::Number("1"),
                Kind::EOL,
                Kind::LParen,
                Kind::Identifier("END"),
                Kind::RParen
            ]
        );
    }

    #[test]
    fn test_comments() {
        let tokens = tokenize("// foo\n// bar");
        assert_eq!(
            tokens.iter().map(|t| t.kind).collect::<Vec<Kind>>(),
            vec![Kind::Comment("// foo"), Kind::EOL, Kind::Comment("// bar"),]
        );

        let tokens = tokenize(" /!foo");
        assert_eq!(
            tokens,
            vec![
                Token {
                    kind: Kind::Invalid('!'),
                    span: Span::new(2, 3)
                },
                Token {
                    kind: Kind::Identifier("foo"),
                    span: Span::new(3, 6)
                }
            ]
        );
    }

    #[test]
    fn test_loc_for_byte_pos() {
        let mut t = Tokenizer::new("á\néf\n\ng");
        while t.next_token().kind != Kind::EOF {}

        assert_eq!(t.loc_for_byte_pos(0), (1, 1)); // á
        assert_eq!(t.loc_for_byte_pos(2), (1, 2)); // \n
        assert_eq!(t.loc_for_byte_pos(3), (2, 1)); // é
        assert_eq!(t.loc_for_byte_pos(5), (2, 2)); // f
        assert_eq!(t.loc_for_byte_pos(8), (4, 1)); // g
    }
}
