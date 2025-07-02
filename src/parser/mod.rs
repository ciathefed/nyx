use miette::{Diagnostic, NamedSource, Result, SourceSpan};

use crate::{
    lexer::{
        Lexer,
        token::{Token, TokenKind},
    },
    parser::ast::{DataSize, Expression, SectionType, Statement},
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
        src: NamedSource<String>,
        #[label("unexpected token here")]
        span: SourceSpan,
    },

    #[diagnostic(code(parser::expected_token))]
    #[error("expected {expected}, got {got} instead")]
    Expected {
        expected: String,
        got: Token,
        #[source_code]
        src: NamedSource<String>,
        #[label("unexpected token")]
        span: SourceSpan,
    },
}

pub struct Parser {
    lexer: Lexer,
    prev_token: Token,
    cur_token: Token,
    peek_token: Token,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
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
            TokenKind::KwSection => {
                self.next_token();

                let section_type = match self.cur_token.kind {
                    TokenKind::SectionName => match self.cur_token.literal.as_ref() {
                        "text" => SectionType::Text,
                        "data" => SectionType::Data,
                        _ => {
                            return Err(Error::UnexpectedToken {
                                token: self.cur_token.clone(),
                                span: self.cur_token.source_span(),
                                src: self.lexer.input.clone(),
                            })?;
                        }
                    },
                    _ => {
                        return Err(Error::Expected {
                            expected: "section name (text or data)".to_string(),
                            got: self.cur_token.clone(),
                            span: self.cur_token.source_span(),
                            src: self.lexer.input.clone(),
                        })?;
                    }
                };

                self.next_token();

                Ok(Statement::Section(
                    section_type,
                    (cur_span.start, self.prev_token.loc.end).into(),
                ))
            }
            TokenKind::Identifier => {
                if self.peek_token_is(TokenKind::Colon) {
                    let ident = self.cur_token.literal.to_string();
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
                        src: self.lexer.input.clone(),
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
            TokenKind::KwAdd => {
                self.next_token();
                let dest = self.parse_expression()?;
                self.expect_cur(TokenKind::Comma)?;
                let lhs = self.parse_expression()?;
                self.expect_cur(TokenKind::Comma)?;
                let rhs = self.parse_expression()?;

                Ok(Statement::Add(
                    dest,
                    lhs,
                    rhs,
                    (cur_span.start, self.prev_token.loc.end).into(),
                ))
            }
            TokenKind::KwSub => {
                self.next_token();
                let dest = self.parse_expression()?;
                self.expect_cur(TokenKind::Comma)?;
                let lhs = self.parse_expression()?;
                self.expect_cur(TokenKind::Comma)?;
                let rhs = self.parse_expression()?;

                Ok(Statement::Sub(
                    dest,
                    lhs,
                    rhs,
                    (cur_span.start, self.prev_token.loc.end).into(),
                ))
            }
            TokenKind::KwMul => {
                self.next_token();
                let dest = self.parse_expression()?;
                self.expect_cur(TokenKind::Comma)?;
                let lhs = self.parse_expression()?;
                self.expect_cur(TokenKind::Comma)?;
                let rhs = self.parse_expression()?;

                Ok(Statement::Mul(
                    dest,
                    lhs,
                    rhs,
                    (cur_span.start, self.prev_token.loc.end).into(),
                ))
            }
            TokenKind::KwDiv => {
                self.next_token();
                let dest = self.parse_expression()?;
                self.expect_cur(TokenKind::Comma)?;
                let lhs = self.parse_expression()?;
                self.expect_cur(TokenKind::Comma)?;
                let rhs = self.parse_expression()?;

                Ok(Statement::Div(
                    dest,
                    lhs,
                    rhs,
                    (cur_span.start, self.prev_token.loc.end).into(),
                ))
            }
            TokenKind::KwAnd => {
                self.next_token();
                let dest = self.parse_expression()?;
                self.expect_cur(TokenKind::Comma)?;
                let lhs = self.parse_expression()?;
                self.expect_cur(TokenKind::Comma)?;
                let rhs = self.parse_expression()?;

                Ok(Statement::And(
                    dest,
                    lhs,
                    rhs,
                    (cur_span.start, self.prev_token.loc.end).into(),
                ))
            }
            TokenKind::KwOr => {
                self.next_token();
                let dest = self.parse_expression()?;
                self.expect_cur(TokenKind::Comma)?;
                let lhs = self.parse_expression()?;
                self.expect_cur(TokenKind::Comma)?;
                let rhs = self.parse_expression()?;

                Ok(Statement::Or(
                    dest,
                    lhs,
                    rhs,
                    (cur_span.start, self.prev_token.loc.end).into(),
                ))
            }
            TokenKind::KwXor => {
                self.next_token();
                let dest = self.parse_expression()?;
                self.expect_cur(TokenKind::Comma)?;
                let lhs = self.parse_expression()?;
                self.expect_cur(TokenKind::Comma)?;
                let rhs = self.parse_expression()?;

                Ok(Statement::Xor(
                    dest,
                    lhs,
                    rhs,
                    (cur_span.start, self.prev_token.loc.end).into(),
                ))
            }
            TokenKind::KwShl => {
                self.next_token();
                let dest = self.parse_expression()?;
                self.expect_cur(TokenKind::Comma)?;
                let lhs = self.parse_expression()?;
                self.expect_cur(TokenKind::Comma)?;
                let rhs = self.parse_expression()?;

                Ok(Statement::Shl(
                    dest,
                    lhs,
                    rhs,
                    (cur_span.start, self.prev_token.loc.end).into(),
                ))
            }
            TokenKind::KwShr => {
                self.next_token();
                let dest = self.parse_expression()?;
                self.expect_cur(TokenKind::Comma)?;
                let lhs = self.parse_expression()?;
                self.expect_cur(TokenKind::Comma)?;
                let rhs = self.parse_expression()?;

                Ok(Statement::Shr(
                    dest,
                    lhs,
                    rhs,
                    (cur_span.start, self.prev_token.loc.end).into(),
                ))
            }
            TokenKind::KwCmp => {
                self.next_token();
                let lhs = self.parse_expression()?;
                self.expect_cur(TokenKind::Comma)?;
                let rhs = self.parse_expression()?;

                Ok(Statement::Cmp(
                    lhs,
                    rhs,
                    (cur_span.start, self.prev_token.loc.end).into(),
                ))
            }
            TokenKind::KwJmp => {
                self.next_token();
                let expr = self.parse_expression()?;

                Ok(Statement::Jmp(
                    expr,
                    (cur_span.start, self.prev_token.loc.end).into(),
                ))
            }
            TokenKind::KwJeq => {
                self.next_token();
                let expr = self.parse_expression()?;

                Ok(Statement::Jeq(
                    expr,
                    (cur_span.start, self.prev_token.loc.end).into(),
                ))
            }
            TokenKind::KwJne => {
                self.next_token();
                let expr = self.parse_expression()?;

                Ok(Statement::Jne(
                    expr,
                    (cur_span.start, self.prev_token.loc.end).into(),
                ))
            }
            TokenKind::KwJlt => {
                self.next_token();
                let expr = self.parse_expression()?;

                Ok(Statement::Jlt(
                    expr,
                    (cur_span.start, self.prev_token.loc.end).into(),
                ))
            }
            TokenKind::KwJgt => {
                self.next_token();
                let expr = self.parse_expression()?;

                Ok(Statement::Jgt(
                    expr,
                    (cur_span.start, self.prev_token.loc.end).into(),
                ))
            }
            TokenKind::KwJle => {
                self.next_token();
                let expr = self.parse_expression()?;

                Ok(Statement::Jle(
                    expr,
                    (cur_span.start, self.prev_token.loc.end).into(),
                ))
            }
            TokenKind::KwJge => {
                self.next_token();
                let expr = self.parse_expression()?;

                Ok(Statement::Jge(
                    expr,
                    (cur_span.start, self.prev_token.loc.end).into(),
                ))
            }
            TokenKind::KwCall => {
                self.next_token();
                let expr = self.parse_expression()?;

                Ok(Statement::Call(
                    expr,
                    (cur_span.start, self.prev_token.loc.end).into(),
                ))
            }
            TokenKind::KwRet => {
                self.next_token();
                Ok(Statement::Ret(
                    (cur_span.start, self.prev_token.loc.end).into(),
                ))
            }
            TokenKind::KwSyscall => {
                self.next_token();
                Ok(Statement::Syscall(
                    (cur_span.start, self.prev_token.loc.end).into(),
                ))
            }
            TokenKind::KwHlt => {
                self.next_token();
                Ok(Statement::Hlt(
                    (cur_span.start, self.prev_token.loc.end).into(),
                ))
            }
            TokenKind::KwDb => {
                self.next_token();
                let mut exprs = vec![];
                loop {
                    exprs.push(self.parse_expression()?);
                    if self.cur_token_is(TokenKind::Comma) {
                        self.next_token();
                        continue;
                    }
                    break;
                }

                Ok(Statement::Db(
                    exprs,
                    (cur_span.start, self.prev_token.loc.end).into(),
                ))
            }
            _ => {
                return Err(Error::UnexpectedToken {
                    token: self.cur_token.clone(),
                    span: self.cur_token.source_span(),
                    src: self.lexer.input.clone(),
                })?;
            }
        }
    }

    fn parse_expression(&mut self) -> Result<Expression> {
        match self.cur_token.kind {
            TokenKind::Identifier => {
                let ident = self.cur_token.literal.to_string();
                self.next_token();
                Ok(Expression::Identifier(ident))
            }
            TokenKind::Register => {
                let reg = match Register::try_from(self.cur_token.literal.as_ref()) {
                    Ok(v) => v,
                    Err(_) => {
                        return Err(Error::UnexpectedToken {
                            token: self.cur_token.clone(),
                            span: self.cur_token.source_span(),
                            src: self.lexer.input.clone(),
                        })?;
                    }
                };
                self.next_token();
                Ok(Expression::Register(reg))
            }
            TokenKind::Integer => {
                let int: i64 = match self.cur_token.literal.parse() {
                    Ok(v) => v,
                    Err(_) => {
                        return Err(Error::UnexpectedToken {
                            token: self.cur_token.clone(),
                            span: self.cur_token.source_span(),
                            src: self.lexer.input.clone(),
                        })?;
                    }
                };
                self.next_token();
                Ok(Expression::IntegerLiteral(int))
            }
            TokenKind::Hexadecimal => {
                let int: i64 = match i64::from_str_radix(&self.cur_token.literal[2..], 16) {
                    Ok(v) => v,
                    Err(_) => {
                        return Err(Error::UnexpectedToken {
                            token: self.cur_token.clone(),
                            span: self.cur_token.source_span(),
                            src: self.lexer.input.clone(),
                        })?;
                    }
                };
                self.next_token();
                Ok(Expression::IntegerLiteral(int))
            }
            TokenKind::Binary => {
                let int: i64 = match i64::from_str_radix(&self.cur_token.literal[2..], 2) {
                    Ok(v) => v,
                    Err(_) => {
                        return Err(Error::UnexpectedToken {
                            token: self.cur_token.clone(),
                            span: self.cur_token.source_span(),
                            src: self.lexer.input.clone(),
                        })?;
                    }
                };
                self.next_token();
                Ok(Expression::IntegerLiteral(int))
            }
            TokenKind::Octal => {
                let int: i64 = match i64::from_str_radix(&self.cur_token.literal[2..], 8) {
                    Ok(v) => v,
                    Err(_) => {
                        return Err(Error::UnexpectedToken {
                            token: self.cur_token.clone(),
                            span: self.cur_token.source_span(),
                            src: self.lexer.input.clone(),
                        })?;
                    }
                };
                self.next_token();
                Ok(Expression::IntegerLiteral(int))
            }
            TokenKind::Float => {
                let float: f64 = match self.cur_token.literal.parse() {
                    Ok(v) => v,
                    Err(_) => {
                        return Err(Error::UnexpectedToken {
                            token: self.cur_token.clone(),
                            span: self.cur_token.source_span(),
                            src: self.lexer.input.clone(),
                        })?;
                    }
                };
                self.next_token();
                Ok(Expression::FloatLiteral(float))
            }
            TokenKind::String => {
                let string = self.cur_token.literal.to_string();
                self.next_token();
                Ok(Expression::StringLiteral(string))
            }
            TokenKind::DataSize => {
                let literal = self.cur_token.literal.to_string();
                let span = self.cur_token.source_span();
                let token = self.cur_token.clone();
                self.next_token();
                match DataSize::try_from(literal.as_str()) {
                    Ok(size) => Ok(Expression::DataSize(size)),
                    Err(_) => Err(Error::UnexpectedToken {
                        token,
                        span,
                        src: self.lexer.input.clone(),
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
                        src: self.lexer.input.clone(),
                    })?;
                }

                self.next_token();

                Ok(Expression::Address(Box::new(base), offset))
            }
            _ => {
                return Err(Error::UnexpectedToken {
                    token: self.cur_token.clone(),
                    span: self.cur_token.source_span(),
                    src: self.lexer.input.clone(),
                })?;
            }
        }
    }

    fn next_token(&mut self) {
        std::mem::swap(&mut self.prev_token, &mut self.cur_token);
        std::mem::swap(&mut self.cur_token, &mut self.peek_token);
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
                src: self.lexer.input.clone(),
            })?
        }
    }
}
