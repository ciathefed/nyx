use std::collections::HashMap;

use lazy_static::lazy_static;

use crate::span::Span;

lazy_static! {
    static ref KEYWORDS: HashMap<&'static str, TokenKind> = HashMap::from([
        // Keywords
        ("hlt", TokenKind::KwHlt),
        ("mov", TokenKind::KwMov),
        ("ldr", TokenKind::KwLdr),
        ("str", TokenKind::KwStr),
        ("push", TokenKind::KwPush),
        ("pop", TokenKind::KwPop),
        // Registers
        ("b0", TokenKind::Register),
        ("w0", TokenKind::Register),
        ("d0", TokenKind::Register),
        ("q0", TokenKind::Register),
        ("ff0", TokenKind::Register),
        ("dd0", TokenKind::Register),
        ("ip", TokenKind::Register),
        ("sp", TokenKind::Register),
        ("bp", TokenKind::Register),
    ]);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Eof,
    Illegal,

    Identifier,
    Register,
    Integer,
    Float,
    String,

    Colon,
    Comma,
    Plus,
    Minus,

    KwHlt,
    KwMov,
    KwLdr,
    KwStr,
    KwPush,
    KwPop,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub(crate) kind: TokenKind,
    pub(crate) literal: String,
    pub(crate) loc: Span,
}

impl Token {
    pub const BLANK: Self = Self {
        kind: TokenKind::Eof,
        literal: String::new(),
        loc: Span { start: 0, end: 0 },
    };

    pub fn new<T: ToString>(kind: TokenKind, literal: T, loc: Span) -> Self {
        Token {
            kind,
            literal: literal.to_string(),
            loc,
        }
    }
}

pub fn lookup_ident(ident: &str) -> TokenKind {
    *KEYWORDS
        .get(ident.to_lowercase().as_str())
        .unwrap_or(&TokenKind::Identifier)
}
