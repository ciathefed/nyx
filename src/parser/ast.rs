use crate::{span::Span, vm::register::Register};

pub use crate::parser::immediate::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Label(String, Span),

    Error(Expression, Span),
    Define(Expression, Expression, Span),
    Include(Expression, Span),
    IfDef(Expression, Span),
    IfNDef(Expression, Span),
    Else(Span),
    EndIf(Span),

    Section(SectionType, Span),
    Entry(Expression, Span),
    Ascii(Expression, Span),
    Asciz(Expression, Span),

    Nop(Span),
    Mov(Expression, Expression, Span),
    Ldr(Expression, Expression, Span),
    Str(Expression, Expression, Span),
    Push(Option<Expression>, Expression, Span),
    Pop(Option<Expression>, Expression, Span),
    Add(Expression, Expression, Expression, Span),
    Sub(Expression, Expression, Expression, Span),
    Mul(Expression, Expression, Expression, Span),
    Div(Expression, Expression, Expression, Span),
    And(Expression, Expression, Expression, Span),
    Or(Expression, Expression, Expression, Span),
    Xor(Expression, Expression, Expression, Span),
    Shl(Expression, Expression, Expression, Span),
    Shr(Expression, Expression, Expression, Span),
    Cmp(Expression, Expression, Span),
    Jmp(Expression, Span),
    Jne(Expression, Span),
    Jeq(Expression, Span),
    Jlt(Expression, Span),
    Jgt(Expression, Span),
    Jle(Expression, Span),
    Jge(Expression, Span),
    Call(Expression, Span),
    Ret(Span),
    Inc(Expression, Span),
    Dec(Expression, Span),
    Syscall(Span),
    Hlt(Span),

    Db(Vec<Expression>, Span),
    Resb(Expression, Span),
}

impl Statement {
    pub fn span(&self) -> Span {
        match self {
            Statement::Error(_, span) => *span,
            Statement::Define(_, _, span) => *span,
            Statement::Include(_, span) => *span,
            Statement::IfDef(_, span) => *span,
            Statement::IfNDef(_, span) => *span,
            Statement::Else(span) => *span,
            Statement::EndIf(span) => *span,
            Statement::Section(_, span) => *span,
            Statement::Entry(_, span) => *span,
            Statement::Ascii(_, span) => *span,
            Statement::Asciz(_, span) => *span,
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
            Statement::Cmp(_, _, span) => *span,
            Statement::Jmp(_, span) => *span,
            Statement::Jne(_, span) => *span,
            Statement::Jeq(_, span) => *span,
            Statement::Jlt(_, span) => *span,
            Statement::Jgt(_, span) => *span,
            Statement::Jle(_, span) => *span,
            Statement::Jge(_, span) => *span,
            Statement::Call(_, span) => *span,
            Statement::Ret(span) => *span,
            Statement::Inc(_, span) => *span,
            Statement::Dec(_, span) => *span,
            Statement::Syscall(span) => *span,
            Statement::Hlt(span) => *span,
            Statement::Db(_, span) => *span,
            Statement::Resb(_, span) => *span,
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
    BinaryOp(Box<Expression>, BinaryOperator, Box<Expression>, Span),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SectionType {
    Text,
    Data,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Add,    // +
    Sub,    // -
    Mul,    // *
    Div,    // /
    BitOr,  // |
    BitAnd, // &
    BitXor, // ^
}
