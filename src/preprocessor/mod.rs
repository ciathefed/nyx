use std::collections::HashMap;

use miette::Result;

use crate::parser::ast::{Expression, Statement};

#[cfg(test)]
mod tests;

pub struct Preprocessor {
    program: Vec<Statement>,
    definitions: HashMap<String, Expression>,
}

impl Preprocessor {
    pub fn new(program: Vec<Statement>) -> Self {
        Self {
            program,
            definitions: HashMap::new(),
        }
    }

    pub fn process(&mut self) -> Result<Vec<Statement>> {
        let mut processed_statements = Vec::new();

        for stmt in std::mem::take(&mut self.program) {
            match stmt {
                Statement::Define(Expression::Identifier(name), value, _) => {
                    self.definitions.insert(name, value);
                }
                other => processed_statements.push(other),
            }
        }

        let mut final_statements = Vec::with_capacity(processed_statements.len());

        for stmt in processed_statements {
            let new_stmt = match stmt {
                Statement::Mov(dest, src, span) => {
                    Statement::Mov(self.substitute_expr(dest), self.substitute_expr(src), span)
                }
                Statement::Ldr(dest, src, span) => {
                    Statement::Ldr(self.substitute_expr(dest), self.substitute_expr(src), span)
                }
                Statement::Str(dest, src, span) => {
                    Statement::Str(self.substitute_expr(dest), self.substitute_expr(src), span)
                }
                Statement::Push(size, src, span) => Statement::Push(
                    size.map(|e| self.substitute_expr(e)),
                    self.substitute_expr(src),
                    span,
                ),
                Statement::Pop(size, dst, span) => Statement::Pop(
                    size.map(|e| self.substitute_expr(e)),
                    self.substitute_expr(dst),
                    span,
                ),
                Statement::Define(key, val, span) => {
                    Statement::Define(self.substitute_expr(key), self.substitute_expr(val), span)
                }
                Statement::Db(exprs, span) => Statement::Db(
                    exprs.into_iter().map(|e| self.substitute_expr(e)).collect(),
                    span,
                ),
                Statement::Label(name, span) => Statement::Label(name, span),
                Statement::Nop(span) => Statement::Nop(span),
                Statement::Hlt(span) => Statement::Hlt(span),
            };

            final_statements.push(new_stmt);
        }

        Ok(final_statements)
    }

    fn substitute_expr(&self, expr: Expression) -> Expression {
        match expr {
            Expression::Identifier(name) => {
                if let Some(replacement) = self.definitions.get(&name) {
                    self.substitute_expr(replacement.clone())
                } else {
                    Expression::Identifier(name)
                }
            }
            Expression::Address(base, offset_opt) => {
                let new_base = Box::new(self.substitute_expr(*base));
                let new_offset = offset_opt.map(|offset| Box::new(self.substitute_expr(*offset)));
                Expression::Address(new_base, new_offset)
            }
            Expression::Register(reg) => Expression::Register(reg),
            Expression::IntegerLiteral(val) => Expression::IntegerLiteral(val),
            Expression::FloatLiteral(val) => Expression::FloatLiteral(val),
            Expression::StringLiteral(val) => Expression::StringLiteral(val),
            Expression::DataSize(size) => Expression::DataSize(size),
        }
    }
}
