use crate::{span::Span, vm::register::Register};

pub use crate::parser::immediate::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Define(Expression, Expression, Span),

    Section(SectionType, Span),

    Label(String, Span),
    Nop(Span),
    Mov(Expression, Expression, Span),
    Ldr(Expression, Expression, Span),
    Str(Expression, Expression, Span),
    Push(Option<Expression>, Expression, Span),
    Pop(Option<Expression>, Expression, Span),

    // Arithmetic Instructions
    Add(Expression, Expression, Expression, Span),
    Sub(Expression, Expression, Expression, Span),
    Mul(Expression, Expression, Expression, Span),
    Div(Expression, Expression, Expression, Span),

    // Bitwise Instructions
    And(Expression, Expression, Expression, Span),
    Or(Expression, Expression, Expression, Span),
    Xor(Expression, Expression, Expression, Span),
    Shl(Expression, Expression, Expression, Span),
    Shr(Expression, Expression, Expression, Span),

    Syscall(Span),
    Hlt(Span),

    Db(Vec<Expression>, Span),
}

impl Statement {
    pub fn span(&self) -> Span {
        match self {
            Statement::Define(_, _, span) => *span,
            Statement::Section(_, span) => *span,
            Statement::Label(_, span) => *span,
            Statement::Nop(span) => *span,
            Statement::Mov(_, _, span) => *span,
            Statement::Ldr(_, _, span) => *span,
            Statement::Str(_, _, span) => *span,
            Statement::Push(_, _, span) => *span,
            Statement::Pop(_, _, span) => *span,
            Statement::Add(_, _, _, span) => *span,
            Statement::Sub(_, _, _, span) => *span,
            Statement::Mul(_, _, _, span) => *span,
            Statement::Div(_, _, _, span) => *span,
            Statement::And(_, _, _, span) => *span,
            Statement::Or(_, _, _, span) => *span,
            Statement::Xor(_, _, _, span) => *span,
            Statement::Shl(_, _, _, span) => *span,
            Statement::Shr(_, _, _, span) => *span,
            Statement::Hlt(span) => *span,
            Statement::Syscall(span) => *span,
            Statement::Db(_, span) => *span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Identifier(String),
    Register(Register),
    IntegerLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    DataSize(DataSize),
    Address(Box<Expression>, Option<Box<Expression>>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SectionType {
    Text,
    Data,
}
