use super::tokens::{Kind, Token};
use crate::common::{Cursor, Span, EOF_CHAR};

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

        let start_pos = self.cursor.pos;
        let token = match self.cursor.c {
            '\n' => {
                self.cursor.advance();
                Token {
                    kind: Kind::EOL,
                    span: Span::new(start_pos, start_pos + 1),
                }
            }
            '/' => {
                let token = match self.cursor.peek() {
                    '/' => self.tokenize_comment(),
                    _ => {
                        self.cursor.advance();
                        Token {
                            kind: Kind::Invalid(&self.src[start_pos..start_pos + 1]),
                            span: Span::new(start_pos, start_pos + 1),
                        }
                    }
                };
                token
            }
            '0'..='9' => self.tokenize_number(),
            c if ident_char(c) => self.tokenize_keyword_or_ident(),
            EOF_CHAR => {
                self.cursor.advance();
                Token::eof(start_pos)
            }
            _ => {
                self.cursor.advance();
                Token {
                    kind: Kind::Invalid(&self.src[start_pos..start_pos + 1]),
                    span: Span::new(start_pos, start_pos + 1),
                }
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

    fn tokenize_keyword_or_ident(&mut self) -> Token<'a> {
        let span = self.cursor.eat_while(|c| ident_char(c));
        let ident = &self.src[span.start..span.end];
        let kind = match ident {
            "push" | "pop" | "add" | "sub" | "neg" | "and" | "or" | "not" | "eq" | "lt" | "gt"
            | "label" | "goto" | "if-goto" => Kind::Instruction(ident),
            "constant" | "local" | "argument" | "static" | "this" | "that" | "temp" | "pointer" => {
                Kind::Segment(ident)
            }
            _ => Kind::Ident(ident),
        };
        Token { kind, span }
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
        '_' | '.' | '$' | ':' | '-' => true,
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
                kind: Kind::EOF,
                span: Span::new(0, 0)
            }
        );
    }

    #[test]
    fn test_spans() {
        assert_eq!(
            tokenize(" push static 1"),
            vec![
                Token {
                    kind: Kind::Instruction("push"),
                    span: Span::new(1, 5)
                },
                Token {
                    kind: Kind::Segment("static"),
                    span: Span::new(6, 12)
                },
                Token {
                    kind: Kind::Number("1"),
                    span: Span::new(13, 14)
                },
            ]
        );
    }

    #[test]
    fn test_instructions() {
        let tokens = tokenize("push local 1\n pop constant 250 \nadd\nsub\ngoto FOO");
        assert_eq!(
            tokens.iter().map(|t| t.kind).collect::<Vec<Kind>>(),
            vec![
                Kind::Instruction("push"),
                Kind::Segment("local"),
                Kind::Number("1"),
                Kind::EOL,
                Kind::Instruction("pop"),
                Kind::Segment("constant"),
                Kind::Number("250"),
                Kind::EOL,
                Kind::Instruction("add"),
                Kind::EOL,
                Kind::Instruction("sub"),
                Kind::EOL,
                Kind::Instruction("goto"),
                Kind::Ident("FOO"),
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

        let tokens = tokenize(" /push");
        assert_eq!(
            tokens,
            vec![
                Token {
                    kind: Kind::Invalid("/"),
                    span: Span::new(1, 2)
                },
                Token {
                    kind: Kind::Instruction("push"),
                    span: Span::new(2, 6)
                }
            ]
        );
    }
}
