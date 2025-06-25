use std::{borrow::Cow, collections::HashMap, fmt};

use lazy_static::lazy_static;
use miette::SourceSpan;

use crate::span::Span;

lazy_static! {
    static ref KEYWORDS: HashMap<&'static str, TokenKind> = HashMap::from([
        // Preprocessor Directives
        ("#define", TokenKind::KwDefine),
        // Assembler Directives
        (".section", TokenKind::KwSection),
        // Instructions
        ("nop", TokenKind::KwNop),
        ("mov", TokenKind::KwMov),
        ("ldr", TokenKind::KwLdr),
        ("str", TokenKind::KwStr),
        ("push", TokenKind::KwPush),
        ("pop", TokenKind::KwPop),
        ("hlt", TokenKind::KwHlt),
        // Data Declaration Directives
        ("db", TokenKind::KwDb),
        // Section Names
        ("text", TokenKind::SectionName),
        ("data", TokenKind::SectionName),
        // Registers
        ("b0", TokenKind::Register),
        ("w0", TokenKind::Register),
        ("d0", TokenKind::Register),
        ("q0", TokenKind::Register),
        ("b1", TokenKind::Register),
        ("w1", TokenKind::Register),
        ("d1", TokenKind::Register),
        ("q1", TokenKind::Register),
        ("ff0", TokenKind::Register),
        ("dd0", TokenKind::Register),
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
    KwSection,

    KwNop,
    KwMov,
    KwLdr,
    KwStr,
    KwPush,
    KwPop,
    KwHlt,

    KwDb,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub(crate) kind: TokenKind,
    pub(crate) literal: Cow<'static, str>,
    pub(crate) loc: Span,
}

impl Token {
    pub const BLANK: Self = Self {
        kind: TokenKind::Eof,
        literal: Cow::Borrowed(""),
        loc: Span { start: 0, end: 0 },
    };

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
