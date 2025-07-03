use std::{borrow::Cow, collections::HashMap, fmt};

use lazy_static::lazy_static;
use miette::SourceSpan;

use crate::span::Span;

lazy_static! {
    static ref KEYWORDS: HashMap<&'static str, TokenKind> = HashMap::from([
        // Preprocessor Directives
        ("#define", TokenKind::KwDefine),
        ("#include", TokenKind::KwInclude),
        // Assembler Directives
        (".section", TokenKind::KwSection),
        (".entry", TokenKind::KwEntry),
        (".ascii", TokenKind::KwAscii),
        (".asciz", TokenKind::KwAsciz),
        // Instructions
        ("nop", TokenKind::KwNop),
        ("mov", TokenKind::KwMov),
        ("ldr", TokenKind::KwLdr),
        ("str", TokenKind::KwStr),
        ("push", TokenKind::KwPush),
        ("pop", TokenKind::KwPop),
        ("add", TokenKind::KwAdd),
        ("sub", TokenKind::KwSub),
        ("mul", TokenKind::KwMul),
        ("div", TokenKind::KwDiv),
        ("and", TokenKind::KwAnd),
        ("or", TokenKind::KwOr),
        ("xor", TokenKind::KwXor),
        ("shl", TokenKind::KwShl),
        ("shr", TokenKind::KwShr),
        ("cmp", TokenKind::KwCmp),
        ("jmp", TokenKind::KwJmp),
        ("jeq", TokenKind::KwJeq),
        ("jne", TokenKind::KwJne),
        ("jlt", TokenKind::KwJlt),
        ("jgt", TokenKind::KwJgt),
        ("jle", TokenKind::KwJle),
        ("jge", TokenKind::KwJge),
        ("call", TokenKind::KwCall),
        ("ret", TokenKind::KwRet),
        ("inc", TokenKind::KwInc),
        ("dec", TokenKind::KwDec),
        ("syscall", TokenKind::KwSyscall),
        ("hlt", TokenKind::KwHlt),
        // Data Declaration Directives
        ("db", TokenKind::KwDb),
        ("resb", TokenKind::KwResb),
        // Section Names
        ("text", TokenKind::SectionName),
        ("data", TokenKind::SectionName),
        // Registers b0..b15
        ("b0", TokenKind::Register), ("b1", TokenKind::Register), ("b2", TokenKind::Register), ("b3", TokenKind::Register),
        ("b4", TokenKind::Register), ("b5", TokenKind::Register), ("b6", TokenKind::Register), ("b7", TokenKind::Register),
        ("b8", TokenKind::Register), ("b9", TokenKind::Register), ("b10", TokenKind::Register), ("b11", TokenKind::Register),
        ("b12", TokenKind::Register), ("b13", TokenKind::Register), ("b14", TokenKind::Register), ("b15", TokenKind::Register),
        // Registers w0..w15
        ("w0", TokenKind::Register), ("w1", TokenKind::Register), ("w2", TokenKind::Register), ("w3", TokenKind::Register),
        ("w4", TokenKind::Register), ("w5", TokenKind::Register), ("w6", TokenKind::Register), ("w7", TokenKind::Register),
        ("w8", TokenKind::Register), ("w9", TokenKind::Register), ("w10", TokenKind::Register), ("w11", TokenKind::Register),
        ("w12", TokenKind::Register), ("w13", TokenKind::Register), ("w14", TokenKind::Register), ("w15", TokenKind::Register),
        // Registers d0..d15
        ("d0", TokenKind::Register), ("d1", TokenKind::Register), ("d2", TokenKind::Register), ("d3", TokenKind::Register),
        ("d4", TokenKind::Register), ("d5", TokenKind::Register), ("d6", TokenKind::Register), ("d7", TokenKind::Register),
        ("d8", TokenKind::Register), ("d9", TokenKind::Register), ("d10", TokenKind::Register), ("d11", TokenKind::Register),
        ("d12", TokenKind::Register), ("d13", TokenKind::Register), ("d14", TokenKind::Register), ("d15", TokenKind::Register),
        // Registers q0..q15
        ("q0", TokenKind::Register), ("q1", TokenKind::Register), ("q2", TokenKind::Register), ("q3", TokenKind::Register),
        ("q4", TokenKind::Register), ("q5", TokenKind::Register), ("q6", TokenKind::Register), ("q7", TokenKind::Register),
        ("q8", TokenKind::Register), ("q9", TokenKind::Register), ("q10", TokenKind::Register), ("q11", TokenKind::Register),
        ("q12", TokenKind::Register), ("q13", TokenKind::Register), ("q14", TokenKind::Register), ("q15", TokenKind::Register),
        // Registers ff0..ff15
        ("ff0", TokenKind::Register), ("ff1", TokenKind::Register), ("ff2", TokenKind::Register), ("ff3", TokenKind::Register),
        ("ff4", TokenKind::Register), ("ff5", TokenKind::Register), ("ff6", TokenKind::Register), ("ff7", TokenKind::Register),
        ("ff8", TokenKind::Register), ("ff9", TokenKind::Register), ("ff10", TokenKind::Register), ("ff11", TokenKind::Register),
        ("ff12", TokenKind::Register), ("ff13", TokenKind::Register), ("ff14", TokenKind::Register), ("ff15", TokenKind::Register),
        // Registers dd0..dd15
        ("dd0", TokenKind::Register), ("dd1", TokenKind::Register), ("dd2", TokenKind::Register), ("dd3", TokenKind::Register),
        ("dd4", TokenKind::Register), ("dd5", TokenKind::Register), ("dd6", TokenKind::Register), ("dd7", TokenKind::Register),
        ("dd8", TokenKind::Register), ("dd9", TokenKind::Register), ("dd10", TokenKind::Register), ("dd11", TokenKind::Register),
        ("dd12", TokenKind::Register), ("dd13", TokenKind::Register), ("dd14", TokenKind::Register), ("dd15", TokenKind::Register),
        // Special Registers
        ("ip", TokenKind::Register),
        ("sp", TokenKind::Register),
        ("bp", TokenKind::Register),
        // Data Sizes
        ("byte", TokenKind::DataSize),
        ("word", TokenKind::DataSize),
        ("dword", TokenKind::DataSize),
        ("qword", TokenKind::DataSize),
        ("float", TokenKind::DataSize),
        ("double", TokenKind::DataSize),
    ]);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Eof,
    Illegal,

    Identifier,
    Register,
    Integer,
    Hexadecimal,
    Binary,
    Octal,
    Float,
    String,
    DataSize,
    SectionName,

    Colon,
    Comma,
    Plus,
    Minus,
    LBracket,
    RBracket,

    KwDefine,
    KwInclude,

    KwSection,
    KwEntry,
    KwAscii,
    KwAsciz,

    KwNop,
    KwMov,
    KwLdr,
    KwStr,
    KwPush,
    KwPop,
    KwAdd,
    KwSub,
    KwMul,
    KwDiv,
    KwAnd,
    KwOr,
    KwXor,
    KwShl,
    KwShr,
    KwCmp,
    KwJmp,
    KwJeq,
    KwJne,
    KwJlt,
    KwJgt,
    KwJle,
    KwJge,
    KwCall,
    KwRet,
    KwInc,
    KwDec,
    KwSyscall,
    KwHlt,

    KwDb,
    KwResb,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub(crate) kind: TokenKind,
    pub(crate) literal: Cow<'static, str>,
    pub(crate) loc: Span,
}

impl Token {
    pub fn new<L: Into<Span>>(kind: TokenKind, literal: &str, loc: L) -> Self {
        Token {
            kind,
            literal: Cow::Owned(literal.to_string()),
            loc: loc.into(),
        }
    }

    pub fn new_static<L: Into<Span>>(kind: TokenKind, literal: &'static str, loc: L) -> Self {
        Token {
            kind,
            literal: Cow::Borrowed(literal),
            loc: loc.into(),
        }
    }

    pub fn new_owned<L: Into<Span>>(kind: TokenKind, literal: String, loc: L) -> Self {
        Token {
            kind,
            literal: Cow::Owned(literal),
            loc: loc.into(),
        }
    }

    pub fn source_span(&self) -> SourceSpan {
        let len = self.loc.end - self.loc.start;
        (self.loc.start, len).into()
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            TokenKind::Eof => write!(f, "EOF"),
            TokenKind::String => write!(f, "\"{}\"", self.literal),
            _ => write!(f, "{}", self.literal),
        }
    }
}

pub fn lookup_ident(ident: &str) -> TokenKind {
    *KEYWORDS
        .get(ident.to_lowercase().as_str())
        .unwrap_or(&TokenKind::Identifier)
}
