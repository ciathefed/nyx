use crate::vm::register::Register;

pub use crate::parser::immediate::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Define(Expression, Expression),
    Label(String),
    Nop,
    Mov(Expression, Expression),
    Ldr(Expression, Expression),
    Str(Expression, Expression),
    Push(Option<Expression>, Expression),
    Pop(Option<Expression>, Expression),
    Hlt,
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
