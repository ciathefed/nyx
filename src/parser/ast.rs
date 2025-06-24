use crate::{span::Span, vm::register::Register};

pub use crate::parser::immediate::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Define(Expression, Expression, Span),
    Label(String, Span),
    Nop(Span),
    Mov(Expression, Expression, Span),
    Ldr(Expression, Expression, Span),
    Str(Expression, Expression, Span),
    Push(Option<Expression>, Expression, Span),
    Pop(Option<Expression>, Expression, Span),
    Hlt(Span),
}

impl Statement {
    pub fn span(&self) -> Span {
        match self {
            Statement::Define(_, _, span) => *span,
            Statement::Label(_, span) => *span,
            Statement::Nop(span) => *span,
            Statement::Mov(_, _, span) => *span,
            Statement::Ldr(_, _, span) => *span,
            Statement::Str(_, _, span) => *span,
            Statement::Push(_, _, span) => *span,
            Statement::Pop(_, _, span) => *span,
            Statement::Hlt(span) => *span,
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
