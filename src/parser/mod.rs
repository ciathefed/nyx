use anyhow::Result;

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
mod instruction;

#[cfg(test)]
mod tests;

#[derive(thiserror::Error, Debug)]
pub enum Erorr {
    #[error("unexpected token: {0}")]
    UnexpectedToken(Token),
    #[error("expected {0}, got {1} instead")]
    Expected(String, Token),
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    cur_token: Token,
    peek_token: Token,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        let mut parser = Self {
            lexer,
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
        match self.cur_token.kind {
            TokenKind::Identifier => {
                if self.peek_token_is(TokenKind::Colon) {
                    let ident = self.cur_token.literal.clone();
                    self.next_token();
                    self.next_token();
                    Ok(Statement::Label(ident))
                } else {
                    Err(Erorr::UnexpectedToken(self.cur_token.clone()).into())
                }
            }
            TokenKind::KwNop => {
                self.next_token();
                Ok(Statement::Nop)
            }
            TokenKind::KwMov => {
                self.next_token();
                let dest = self.parse_expression()?;
                self.expect_cur(TokenKind::Comma)?;
                let src = self.parse_expression()?;
                Ok(Statement::Mov(dest, src))
            }
            TokenKind::KwLdr => {
                self.next_token();

                let size = if self.cur_token.kind == TokenKind::DataSize {
                    self.parse_expression()?
                } else {
                    Expression::DataSize(DataSize::QWord)
                };

                let dest = self.parse_expression()?;
                self.expect_cur(TokenKind::Comma)?;
                let src = self.parse_expression()?;

                Ok(Statement::Ldr(size, dest, src))
            }
            TokenKind::KwStr => {
                self.next_token();

                let size = if self.cur_token.kind == TokenKind::DataSize {
                    self.parse_expression()?
                } else {
                    Expression::DataSize(DataSize::QWord)
                };

                let src = self.parse_expression()?;
                self.expect_cur(TokenKind::Comma)?;
                let dest = self.parse_expression()?;

                Ok(Statement::Str(size, src, dest))
            }
            TokenKind::KwPush => {
                self.next_token();
                let size = self.parse_expression()?;
                self.next_token();
                let src = self.parse_expression()?;
                Ok(Statement::Push(size, src))
            }
            TokenKind::KwPop => {
                self.next_token();
                let size = self.parse_expression()?;
                self.next_token();
                let dest = self.parse_expression()?;
                Ok(Statement::Pop(size, dest))
            }
            TokenKind::KwHlt => {
                self.next_token();
                Ok(Statement::Hlt)
            }
            _ => return Err(Erorr::UnexpectedToken(self.cur_token.clone()).into()),
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
                    Err(_) => Err(Erorr::UnexpectedToken(token).into()),
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
                    return Err(Erorr::Expected("]".to_string(), self.cur_token.clone()).into());
                }

                self.next_token();

                Ok(Expression::Address(Box::new(base), offset))
            }
            _ => todo!("parse_expression"),
        }
    }

    fn next_token(&mut self) {
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
            Err(Erorr::UnexpectedToken(self.peek_token.clone()).into())
        }
    }

    fn expect_peek(&mut self, kind: TokenKind) -> Result<()> {
        if self.peek_token_is(kind) {
            self.next_token();
            Ok(())
        } else {
            Err(Erorr::UnexpectedToken(self.peek_token.clone()).into())
        }
    }
}
