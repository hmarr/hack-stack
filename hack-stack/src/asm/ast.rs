use std::convert::TryFrom;

use crate::common::Span;

#[derive(Debug, PartialEq)]
pub struct Label<'a> {
    pub name: &'a str,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub enum Address<'a> {
    Value(u16),
    Symbol(&'a str),
}

#[derive(Debug, PartialEq)]
pub enum Bit {
    One,
    Zero,
}

impl TryFrom<&str> for Bit {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "0" => Ok(Bit::Zero),
            "1" => Ok(Bit::One),
            _ => Err(format!("invalid binary value {}, expected 0 or 1", value)),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Register {
    D,
    A,
    M,
}

impl TryFrom<&str> for Register {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "D" => Ok(Register::D),
            "A" => Ok(Register::A),
            "M" => Ok(Register::M),
            _ => Err(format!("invalid register {}, expected D, A, or M", value)),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Operand {
    Bit(Bit),
    Register(Register),
}

#[derive(Debug, PartialEq)]
pub enum UnaryOperator {
    Not,
    Minus,
}

#[derive(Debug, PartialEq)]
pub struct UnaryOperation {
    pub op: UnaryOperator,
    pub operand: Operand,
}

#[derive(Debug, PartialEq)]
pub enum BinaryOperator {
    Plus,
    Minus,
    And,
    Or,
}

#[derive(Debug, PartialEq)]
pub struct BinaryOperation {
    pub op: BinaryOperator,
    pub lhs: Register,
    pub rhs: Operand,
}

#[derive(Debug, PartialEq)]
pub enum Comp {
    Bit(Bit),
    Register(Register),
    UnaryOperation(UnaryOperation),
    BinaryOperation(BinaryOperation),
}

#[derive(Debug, PartialEq)]
pub enum Jump {
    JGT,
    JEQ,
    JGE,
    JLT,
    JNE,
    JLE,
    JMP,
}

impl TryFrom<&str> for Jump {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "JGT" => Ok(Jump::JGT),
            "JEQ" => Ok(Jump::JEQ),
            "JGE" => Ok(Jump::JGE),
            "JLT" => Ok(Jump::JLT),
            "JNE" => Ok(Jump::JNE),
            "JLE" => Ok(Jump::JLE),
            "JMP" => Ok(Jump::JMP),
            _ => Err(format!("invalid jump mnemonic {}", value)),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Dest {
    pub a: bool,
    pub d: bool,
    pub m: bool,
}

impl TryFrom<&str> for Dest {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut dest = Dest {
            a: false,
            d: false,
            m: false,
        };
        for c in value.chars() {
            match c {
                'A' => dest.a = true,
                'D' => dest.d = true,
                'M' => dest.m = true,
                _ => {
                    return Err(format!(
                        "invalid destination {}, expected a combination of A, D and M",
                        value
                    ))
                }
            }
        }
        Ok(dest)
    }
}

#[derive(Debug, PartialEq)]
pub struct AInstruction<'a> {
    pub addr: Address<'a>,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub struct CInstruction {
    pub dest: Option<Dest>,
    pub comp: Comp,
    pub jump: Option<Jump>,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub enum Instruction<'a> {
    Label(Label<'a>),
    A(AInstruction<'a>),
    C(CInstruction),
}

impl<'a> Instruction<'a> {
    pub fn span(&self) -> Span {
        match self {
            Self::Label(label) => label.span,
            Self::A(inst) => inst.span,
            Self::C(inst) => inst.span,
        }
    }
}
