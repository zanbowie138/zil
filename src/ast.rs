use crate::lexer::Span;
use regex::Regex;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    Nil,
    Bool(bool),
    /// Value and the base it was written in.
    Int(i64, u32),
    Float(f64),
    Str(Rc<str>),
    Regex(Rc<Regex>),
    Ident(String),
    List(Vec<Expr>),
    Map(Vec<(String, Expr)>),
    /// Number literal with a unit attached: `5 km`, `60 km/h`.
    Qty(Box<Expr>, UnitSpec),
    Unary(UnOp, Box<Expr>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    To(Box<Expr>, Target),
    Assign(Box<Expr>, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Field(Box<Expr>, String),
    Index(Box<Expr>, Box<Expr>),
    Slice(Box<Expr>, Option<Box<Expr>>, Option<Box<Expr>>),
    Fn(Rc<FnDef>),
    If(Box<Expr>, Box<Expr>, Option<Box<Expr>>),
    While(Box<Expr>, Box<Expr>),
    For(String, Box<Expr>, Box<Expr>),
    Block(Vec<Expr>),
    Return(Option<Box<Expr>>),
}

/// Unit names with powers, e.g. `km/h^2` = [("km", 1), ("h", -2)].
pub type UnitSpec = Vec<(String, i8)>;

#[derive(Debug, Clone)]
pub enum Target {
    Unit(UnitSpec),
    /// Several units, largest first: `to d h min`, `to ft in`.
    Units(Vec<UnitSpec>),
    /// A keyword some module registered, with an optional int argument: `hex`, `hex(32)`, `UTC`.
    Named(String, Option<i64>),
    /// A string such as a time zone name: `to "Asia/Tokyo"`.
    Str(String),
}

/// How an integer displays: `base`, and `width` bits of two's complement (0 = plain signed).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Radix {
    pub base: u32,
    pub width: u32,
}

impl Radix {
    pub const DEC: Radix = Radix { base: 10, width: 0 };
}

#[derive(Debug)]
pub struct FnDef {
    pub params: Vec<String>,
    pub body: Expr,
}

#[derive(Debug, Clone, Copy)]
pub enum UnOp {
    Neg,
    Not,
    BitNot,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    IntDiv,
    Rem,
    Pow,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    Range,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}
