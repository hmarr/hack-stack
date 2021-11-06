use crate::common::Span;

#[derive(Debug, PartialEq)]
pub enum Instruction<'a> {
    Push(PushInstruction),
    Pop(PopInstruction),
    Add(Span),
    Sub(Span),
    Eq(Span),
    Gt(Span),
    Lt(Span),
    Neg(Span),
    And(Span),
    Or(Span),
    Not(Span),
    Goto(GotoInstruction<'a>),
    IfGoto(IfGotoInstruction<'a>),
    Label(LabelInstruction<'a>),
}

impl<'a> Instruction<'a> {
    pub fn span(&self) -> Span {
        match self {
            Self::Push(push) => push.span,
            Self::Pop(pop) => pop.span,
            Self::Add(span) => *span,
            Self::Sub(span) => *span,
            Self::Eq(span) => *span,
            Self::Gt(span) => *span,
            Self::Lt(span) => *span,
            Self::Neg(span) => *span,
            Self::And(span) => *span,
            Self::Or(span) => *span,
            Self::Not(span) => *span,
            Self::Goto(goto) => goto.span,
            Self::IfGoto(if_goto) => if_goto.span,
            Self::Label(label) => label.span,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct PushInstruction {
    pub segment: Segment,
    pub offset: u16,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub struct PopInstruction {
    pub segment: Segment,
    pub offset: u16,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub struct GotoInstruction<'a> {
    pub label: &'a str,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub struct IfGotoInstruction<'a> {
    pub label: &'a str,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub struct LabelInstruction<'a> {
    pub label: &'a str,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub enum Segment {
    Constant,
    Local,
    Argument,
    Static,
    This,
    That,
    Temp,
    Pointer,
}
