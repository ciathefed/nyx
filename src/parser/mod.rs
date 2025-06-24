use miette::{Diagnostic, Result, SourceSpan};

use crate::{
    lexer::{
        Lexer,
        token::{Token, TokenKind},
    },
    parser::ast::{DataSize, Expression, Statement},
    vm::register::Register,
};

pub mod ast;
mod immediate;

#[cfg(test)]
mod tests;

#[derive(Debug, thiserror::Error, Diagnostic)]
pub enum Error {
    #[diagnostic(code(parser::unexpected_token))]
    #[error("unexpected token: {token}")]
    UnexpectedToken {
        token: Token,
        #[source_code]
        src: String,
        #[label("unexpected token here")]
        span: SourceSpan,
    },

    #[diagnostic(code(parser::expected_token))]
    #[error("expected {expected}, got {got} instead")]
    Expected {
        expected: String,
        got: Token,
        #[source_code]
        src: String,
        #[label("unexpected token")]
        span: SourceSpan,
    },
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    prev_token: Token,
    cur_token: Token,
    peek_token: Token,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        let mut parser = Self {
            lexer,
            prev_token: Token::BLANK,
            cur_token: Token::BLANK,
            peek_token: Token::BLANK,
        };
        parser.next_token();
        parser.next_token();
        parser
    }

    pub fn parse(&mut self) -> Result<Vec<Statement>> {
        let mut stmts = vec![];
        while self.cur_token.kind != TokenKind::Eof {
            stmts.push(self.parse_statement()?);
        }
        Ok(stmts)
    }

    fn parse_statement(&mut self) -> Result<Statement> {
        let cur_span = self.cur_token.loc;
        match self.cur_token.kind {
            TokenKind::Identifier => {
                if self.peek_token_is(TokenKind::Colon) {
                    let ident = self.cur_token.literal.clone();
                    self.next_token();
                    self.next_token();
                    Ok(Statement::Label(
                        ident,
                        (cur_span.start, self.prev_token.loc.end).into(),
                    ))
                } else {
                    Err(Error::UnexpectedToken {
                        token: self.cur_token.clone(),
                        span: self.cur_token.source_span(),
                        src: self.lexer.input.to_string(),
                    })?
                }
            }
            TokenKind::KwNop => {
                self.next_token();
                Ok(Statement::Nop(cur_span))
            }
            TokenKind::KwMov => {
                self.next_token();

                let dest = self.parse_expression()?;
                self.expect_cur(TokenKind::Comma)?;
                let src = self.parse_expression()?;

                Ok(Statement::Mov(
                    dest,
                    src,
                    (cur_span.start, self.prev_token.loc.end).into(),
                ))
            }
            TokenKind::KwLdr => {
                self.next_token();

                let dest = self.parse_expression()?;
                self.expect_cur(TokenKind::Comma)?;
                let src = self.parse_expression()?;

                Ok(Statement::Ldr(
                    dest,
                    src,
                    (cur_span.start, self.prev_token.loc.end).into(),
                ))
            }
            TokenKind::KwStr => {
                self.next_token();

                let src = self.parse_expression()?;
                self.expect_cur(TokenKind::Comma)?;
                let dest = self.parse_expression()?;

                Ok(Statement::Str(
                    src,
                    dest,
                    (cur_span.start, self.prev_token.loc.end).into(),
                ))
            }
            TokenKind::KwPush => {
                self.next_token();

                let size = if self.cur_token.kind == TokenKind::DataSize {
                    Some(self.parse_expression()?)
                } else {
                    None
                };

                let src = self.parse_expression()?;

                Ok(Statement::Push(
                    size,
                    src,
                    (cur_span.start, self.prev_token.loc.end).into(),
                ))
            }
            TokenKind::KwPop => {
                self.next_token();

                let size = if self.cur_token.kind == TokenKind::DataSize {
                    Some(self.parse_expression()?)
                } else {
                    None
                };

                let dest = self.parse_expression()?;

                Ok(Statement::Pop(
                    size,
                    dest,
                    (cur_span.start, self.prev_token.loc.end).into(),
                ))
            }
            TokenKind::KwHlt => {
                self.next_token();
                Ok(Statement::Hlt(
                    (cur_span.start, self.prev_token.loc.end).into(),
                ))
            }
            TokenKind::KwDefine => {
                self.next_token();

                let name = self.parse_expression()?;
                let value = self.parse_expression()?;

                Ok(Statement::Define(
                    name,
                    value,
                    (cur_span.start, self.prev_token.loc.end).into(),
                ))
            }
            _ => {
                return Err(Error::UnexpectedToken {
                    token: self.cur_token.clone(),
                    span: self.cur_token.source_span(),
                    src: self.lexer.input.to_string(),
                })?;
            }
        }
    }

    fn parse_expression(&mut self) -> Result<Expression> {
        match self.cur_token.kind {
            TokenKind::Identifier => {
                let ident = self.cur_token.literal.clone();
                self.next_token();
                Ok(Expression::Identifier(ident))
            }
            TokenKind::Register => {
                let reg = match Register::try_from(self.cur_token.literal.as_str()) {
                    Ok(v) => v,
                    Err(_) => todo!(),
                };
                self.next_token();
                Ok(Expression::Register(reg))
            }
            TokenKind::Integer => {
                let int: i64 = match self.cur_token.literal.parse() {
                    Ok(v) => v,
                    Err(_) => todo!(),
                };
                self.next_token();
                Ok(Expression::IntegerLiteral(int))
            }
            TokenKind::Float => {
                let float: f64 = match self.cur_token.literal.parse() {
                    Ok(v) => v,
                    Err(_) => todo!(),
                };
                self.next_token();
                Ok(Expression::FloatLiteral(float))
            }
            TokenKind::String => {
                let string = self.cur_token.literal.clone();
                self.next_token();
                Ok(Expression::StringLiteral(string))
            }
            TokenKind::DataSize => {
                let token = self.cur_token.clone();
                self.next_token();
                match DataSize::try_from(token.literal.as_str()) {
                    Ok(size) => Ok(Expression::DataSize(size)),
                    Err(_) => Err(Error::UnexpectedToken {
                        token: token.clone(),
                        span: token.source_span(),
                        src: self.lexer.input.to_string(),
                    })?,
                }
            }
            TokenKind::LBracket => {
                self.next_token();
                let base = self.parse_expression()?;

                let offset = if self.cur_token_is(TokenKind::Comma) {
                    self.next_token();
                    let off = self.parse_expression()?;
                    Some(Box::new(off))
                } else {
                    None
                };

                if !self.cur_token_is(TokenKind::RBracket) {
                    return Err(Error::Expected {
                        expected: "]".to_string(),
                        got: self.cur_token.clone(),
                        span: self.cur_token.source_span(),
                        src: self.lexer.input.to_string(),
                    })?;
                }

                self.next_token();

                Ok(Expression::Address(Box::new(base), offset))
            }
            _ => todo!("parse_expression"),
        }
    }

    fn next_token(&mut self) {
        self.prev_token = self.cur_token.clone();
        self.cur_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }

    fn cur_token_is(&self, kind: TokenKind) -> bool {
        self.cur_token.kind == kind
    }

    fn peek_token_is(&self, kind: TokenKind) -> bool {
        self.peek_token.kind == kind
    }

    fn expect_cur(&mut self, kind: TokenKind) -> Result<()> {
        if self.cur_token_is(kind) {
            self.next_token();
            Ok(())
        } else {
            Err(Error::UnexpectedToken {
                token: self.peek_token.clone(),
                span: self.peek_token.source_span(),
                src: self.lexer.input.to_string(),
            })?
        }
    }
}
