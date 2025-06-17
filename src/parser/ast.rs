use std::fmt;

use crate::vm::register::Register;

pub use crate::parser::{immediate::*, instruction::*};

#[derive(Debug, PartialEq)]
pub enum Statement {
    Label(String),
    Nop,
    Mov(Expression, Expression),
    Ldr(Expression, Expression, Expression), // DataSize, Register (Dest), Addressing (Src)
    Str(Expression, Expression, Expression), // DataSize, Register (Src), Addressing (Dest)
    Push(Expression, Expression),
    Pop(Expression, Expression),
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
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::Identifier(_) => todo!(),
            Expression::Register(_) => todo!(),
            Expression::IntegerLiteral(_) => todo!(),
            Expression::FloatLiteral(_) => todo!(),
            Expression::StringLiteral(_) => todo!(),
            Expression::DataSize(_) => todo!(),
        }
    }
}
