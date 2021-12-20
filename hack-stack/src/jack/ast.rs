use crate::common::Spanned;

#[derive(Debug, PartialEq)]
pub struct Class<'a> {
    pub name: Spanned<&'a str>,
    pub var_decs: Vec<ClassVarDec<'a>>,
    pub subroutine_decs: Vec<SubroutineDec<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct ClassVarDec<'a> {
    pub kind: ClassVarKind,
    pub var_dec: VarDec<'a>,
}

#[derive(Debug, PartialEq)]
pub enum ClassVarKind {
    Static,
    Field,
}

#[derive(Debug, PartialEq)]
pub struct Param<'a> {
    pub ty: Spanned<&'a str>,
    pub name: Spanned<&'a str>,
}

#[derive(Debug, PartialEq)]
pub enum SubroutineKind {
    Constructor,
    Function,
    Method,
}

#[derive(Debug, PartialEq)]
pub struct SubroutineDec<'a> {
    pub name: Spanned<&'a str>,
    pub return_type: Spanned<&'a str>,
    pub params: Vec<Param<'a>>,
    pub kind: Spanned<SubroutineKind>,
    pub statements: Vec<Stmt<'a>>,
}

#[derive(Debug, PartialEq)]
pub enum Stmt<'a> {
    Var(VarDec<'a>),
    Let(LetStmt<'a>),
    If(IfStmt<'a>),
    While(WhileStmt<'a>),
    Do(SubroutineCall<'a>),
    Return(ReturnStmt<'a>),
}

#[derive(Debug, PartialEq)]
pub struct VarDec<'a> {
    pub ty: Spanned<&'a str>,
    pub names: Vec<Spanned<&'a str>>,
}

#[derive(Debug, PartialEq)]
pub struct LetStmt<'a> {
    pub assignee: Assignee<'a>,
    pub expr: Spanned<Box<Expr<'a>>>,
}

#[derive(Debug, PartialEq)]
pub enum Assignee<'a> {
    Name(Spanned<&'a str>),
    Index(Index<'a>),
}

#[derive(Debug, PartialEq)]
pub struct IfStmt<'a> {
    pub cond: Spanned<Box<Expr<'a>>>,
    pub if_arm: Vec<Stmt<'a>>,
    pub else_arm: Vec<Stmt<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct WhileStmt<'a> {
    pub cond: Spanned<Box<Expr<'a>>>,
    pub body: Vec<Stmt<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct ReturnStmt<'a> {
    pub expr: Option<Spanned<Box<Expr<'a>>>>,
}

#[derive(Debug, PartialEq)]
pub enum Expr<'a> {
    IntLit(Spanned<u16>),
    StrLit(Spanned<&'a str>),
    BoolLit(Spanned<bool>),
    NullLit(Spanned<()>),
    Ident(Spanned<&'a str>),
    UnaryOp(UnaryOp<'a>),
    BinOp(BinOp<'a>),
    SubroutineCall(SubroutineCall<'a>),
    Index(Index<'a>),
}

#[derive(Debug, PartialEq)]
pub enum UnaryOpKind {
    Neg,
    Not,
}

#[derive(Debug, PartialEq)]
pub struct UnaryOp<'a> {
    pub op: Spanned<UnaryOpKind>,
    pub expr: Spanned<Box<Expr<'a>>>,
}

impl UnaryOpKind {
    pub fn precedence(&self) -> usize {
        match self {
            Self::Neg => 5,
            Self::Not => 5,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum BinOpKind {
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
    Lt,
    Gt,
    Eq,
}

impl BinOpKind {
    pub fn precedence(&self) -> usize {
        match self {
            Self::Add => 2,
            Self::Sub => 2,
            Self::Mul => 3,
            Self::Div => 3,
            // NOTE: in most C-derived languages, bitwise & and | are lower
            // precedence than other arithmetic operators, so this deviation
            // might be surprising. But hey, let's keep life interesting!
            Self::And => 4,
            Self::Or => 4,
            Self::Lt => 1,
            Self::Gt => 1,
            Self::Eq => 1,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct BinOp<'a> {
    pub op: Spanned<BinOpKind>,
    pub lhs: Spanned<Box<Expr<'a>>>,
    pub rhs: Spanned<Box<Expr<'a>>>,
}

#[derive(Debug, PartialEq)]
pub struct SubroutineCall<'a> {
    // TODO: `class` doesn't cover all - it could be another object, it could be
    // `this`, it could be a class
    pub class: Option<Spanned<&'a str>>,
    pub subroutine: Spanned<&'a str>,
    pub args: Vec<Spanned<Box<Expr<'a>>>>,
}

#[derive(Debug, PartialEq)]
pub struct Index<'a> {
    pub array_name: Spanned<&'a str>,
    pub index: Spanned<Box<Expr<'a>>>,
}
