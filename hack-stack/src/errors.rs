use crate::tokenize::Span;

#[derive(Debug, PartialEq)]
pub struct SpanError {
    pub msg: String,
    pub span: Span,
}

impl SpanError {
    pub fn new(msg: String, span: Span) -> Self {
        SpanError { msg, span }
    }
}
