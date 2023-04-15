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
            '/' => {
                let token = match self.cursor.peek() {
                    '/' => self.tokenize_line_comment(),
                    '*' => self.tokenize_block_comment(),
                    _ => self.tokenize_symbol(),
                };
                token
            }
            c if symbol_char(c) => self.tokenize_symbol(),
            c if ident_start_char(c) => self.tokenize_keyword_or_identifier(),
            '0'..='9' => self.tokenize_integer_constant(),
            '"' => self.tokenize_string_constant(),
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

    fn tokenize_integer_constant(&mut self) -> Token<'a> {
        let span = self.cursor.eat_while(|c| c.is_numeric());
        Token {
            kind: Kind::IntConst(&self.src[span.start..span.end]),
            span,
        }
    }

    fn tokenize_string_constant(&mut self) -> Token<'a> {
        assert!(self.cursor.c == '"');

        let start_pos = self.cursor.pos;
        self.cursor.advance();

        let span = self
            .cursor
            .eat_while(|c| c != '"' && c != '\n' && c != EOF_CHAR);
        // Add the opening quote to the span
        let span = Span::new(start_pos, span.end);

        // If we reached the end and the next character isn't a double quote, it's
        // an invalid string. In the future it'd be good to emit a diagnostic error
        // here, but currently error reporting isn't wired up to the lexer.
        if self.cursor.c != '"' {
            Token {
                kind: Kind::Invalid(&self.src[span.start..span.end]),
                span,
            }
        } else {
            // Eat the closing quote and add it to the span
            self.cursor.advance();
            let span = Span::new(span.start, self.cursor.pos);
            // According to the spec, we shouldn't include the quotes in the token literal
            let literal = &self.src[span.start + 1..span.end - 1];
            Token {
                kind: Kind::StrConst(literal),
                span,
            }
        }
    }

    fn tokenize_keyword_or_identifier(&mut self) -> Token<'a> {
        let span = self.cursor.eat_while(ident_char);
        let ident = &self.src[span.start..span.end];
        let kind = match ident {
            "class" | "constructor" | "method" | "function" | "int" | "boolean" | "char"
            | "void" | "var" | "static" | "field" | "let" | "do" | "if" | "else" | "while"
            | "return" | "true" | "false" | "null" | "this" => Kind::Keyword(ident),
            _ => Kind::Ident(ident),
        };
        Token { kind, span }
    }

    fn tokenize_symbol(&mut self) -> Token<'a> {
        assert!(symbol_char(self.cursor.c));

        let span = Span::new(self.cursor.pos, self.cursor.pos + 1);
        self.cursor.advance();
        Token {
            kind: Kind::Symbol(&self.src[span.start..span.end]),
            span,
        }
    }

    fn tokenize_line_comment(&mut self) -> Token<'a> {
        let span = self.cursor.eat_while(|c| c != '\n' && c != EOF_CHAR);
        Token {
            kind: Kind::Comment(&self.src[span.start..span.end]),
            span,
        }
    }

    fn tokenize_block_comment(&mut self) -> Token<'a> {
        let start = self.cursor.pos;
        let mut length = 0;

        while !(self.cursor.c == '*' && self.cursor.peek() == '/') && self.cursor.c != EOF_CHAR {
            length += self.cursor.c.len_utf8();
            self.cursor.advance();
        }
        let span = Span::new(start, start + length);

        if self.cursor.c == '*' && self.cursor.peek() == '/' {
            self.cursor.advance();
            self.cursor.advance();
            let span = Span::new(start, self.cursor.pos);

            Token {
                kind: Kind::Comment(&self.src[span.start..span.end]),
                span,
            }
        } else {
            Token {
                kind: Kind::Invalid(&self.src[span.start..span.end]),
                span,
            }
        }
    }

    fn eat_whitespace(&mut self) {
        while self.cursor.c.is_whitespace() {
            self.cursor.advance();
        }
    }
}

fn ident_start_char(c: char) -> bool {
    match c {
        c if c.is_alphabetic() => true,
        '_' => true,
        _ => false,
    }
}

fn ident_char(c: char) -> bool {
    match c {
        c if c.is_alphanumeric() => true,
        '_' => true,
        _ => false,
    }
}

#[allow(clippy::match_like_matches_macro)]
fn symbol_char(c: char) -> bool {
    match c {
        '(' | ')' | '[' | ']' | '{' | '}' | ',' | ';' | '=' | '.' | '+' | '-' | '*' | '/' | '&'
        | '|' | '~' | '<' | '>' => true,
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
            tokenize(" 1 + \"foo\""),
            vec![
                Token {
                    kind: Kind::IntConst("1"),
                    span: Span::new(1, 2)
                },
                Token {
                    kind: Kind::Symbol("+"),
                    span: Span::new(3, 4)
                },
                Token {
                    kind: Kind::StrConst("foo"),
                    span: Span::new(5, 10)
                },
            ]
        );
    }

    #[test]
    fn test_keywords() {
        let tokens = tokenize("class\nvar function");
        assert_eq!(
            tokens.iter().map(|t| t.kind).collect::<Vec<Kind>>(),
            vec![
                Kind::Keyword("class"),
                Kind::Keyword("var"),
                Kind::Keyword("function"),
            ]
        );
    }

    #[test]
    fn test_symbols() {
        let tokens = tokenize("+- / []");
        assert_eq!(
            tokens.iter().map(|t| t.kind).collect::<Vec<Kind>>(),
            vec![
                Kind::Symbol("+"),
                Kind::Symbol("-"),
                Kind::Symbol("/"),
                Kind::Symbol("["),
                Kind::Symbol("]"),
            ]
        );
    }

    #[test]
    fn test_string_constants() {
        assert_eq!(
            tokenize("\"foo\"\n\"bar"),
            vec![
                Token {
                    kind: Kind::StrConst("foo"),
                    span: Span::new(0, 5)
                },
                Token {
                    kind: Kind::Invalid("\"bar"),
                    span: Span::new(6, 10)
                }
            ]
        );
    }

    #[test]
    fn test_comments() {
        let tokens = tokenize("// foo\n// bar");
        assert_eq!(
            tokens.iter().map(|t| t.kind).collect::<Vec<Kind>>(),
            vec![Kind::Comment("// foo"), Kind::Comment("// bar"),]
        );

        assert_eq!(
            tokenize(" /* foo\nbar*/ "),
            vec![Token {
                kind: Kind::Comment("/* foo\nbar*/"),
                span: Span::new(1, 13),
            }]
        );

        assert_eq!(
            tokenize(" /* "),
            vec![Token {
                kind: Kind::Invalid("/* "),
                span: Span::new(1, 4),
            }]
        );
    }
}
