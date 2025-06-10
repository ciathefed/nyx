use crate::vm::register::Register;

#[derive(Debug, PartialEq)]
pub enum Statement {
    Label(String),
    Hlt,
    Mov(Expression, Expression),
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    Identifier(String),
    Register(Register),
    IntegerLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
}
