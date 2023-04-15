use super::tokens::{Kind, Token};
use crate::common::{Cursor, EOF_CHAR};

pub struct Tokenizer<'a> {
    src: &'a str,
    cursor: Cursor<'a>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(src: &'a str) -> Tokenizer<'a> {
        Tokenizer {
            src,
            cursor: Cursor::new(src),
        }
    }

    pub fn next_token(&mut self) -> Token<'a> {
        self.eat_whitespace();

        let token = match self.cursor.c {
            '\n' | '@' | '=' | '+' | '-' | '&' | '|' | '!' | ';' | '(' | ')' => {
                let token = Token::from_char(self.cursor.pos, self.cursor.c);
                self.cursor.advance();
                token
            }
            '/' => {
                let token = match self.cursor.peek() {
                    '/' => self.tokenize_comment(),
                    _ => {
                        let token = Token::invalid(self.cursor.c, self.cursor.pos);
                        self.cursor.advance();
                        token
                    }
                };
                token
            }
            c if ident_start_char(c) => self.tokenize_identifier(),
            '0'..='9' => self.tokenize_number(),
            EOF_CHAR => {
                let token = Token::eof(self.cursor.pos);
                self.cursor.advance();
                token
            }
            c => {
                let token = Token::invalid(c, self.cursor.pos);
                self.cursor.advance();
                token
            }
        };

        token
    }

    fn tokenize_number(&mut self) -> Token<'a> {
        let span = self.cursor.eat_while(|c| c.is_numeric());
        Token {
            kind: Kind::Number(&self.src[span.start..span.end]),
            span,
        }
    }

    fn tokenize_identifier(&mut self) -> Token<'a> {
        let span = self.cursor.eat_while(ident_char);
        Token {
            kind: Kind::Identifier(&self.src[span.start..span.end]),
            span,
        }
    }

    fn tokenize_comment(&mut self) -> Token<'a> {
        let span = self.cursor.eat_while(|c| c != '\n' && c != EOF_CHAR);
        Token {
            kind: Kind::Comment(&self.src[span.start..span.end]),
            span,
        }
    }

    fn eat_whitespace(&mut self) {
        while self.cursor.c.is_whitespace() && self.cursor.c != '\n' {
            self.cursor.advance();
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
                kind: Kind::Eof, ..
            } => None,
            token => Some(token),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::Span;

    fn tokenize(s: &str) -> Vec<Token> {
        Tokenizer::new(s).collect()
    }

    #[test]
    fn test_empty_string() {
        let mut t = Tokenizer::new("");
        assert_eq!(
            t.next_token(),
            Token {
                kind: Kind::Eof,
                span: Span::new(0, 0)
            }
        );
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
                Kind::Eol,
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
                Kind::Eol,
                Kind::AtSign,
                Kind::Number("1"),
                Kind::Eol,
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
            vec![Kind::Comment("// foo"), Kind::Eol, Kind::Comment("// bar"),]
        );

        let tokens = tokenize(" /!foo");
        assert_eq!(
            tokens,
            vec![
                Token {
                    kind: Kind::Invalid('/'),
                    span: Span::new(1, 2)
                },
                Token {
                    kind: Kind::Not,
                    span: Span::new(2, 3)
                },
                Token {
                    kind: Kind::Identifier("foo"),
                    span: Span::new(3, 6)
                }
            ]
        );
    }
}
