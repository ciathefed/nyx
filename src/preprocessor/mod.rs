use std::collections::HashMap;

use miette::Result;

use crate::parser::ast::{Expression, Statement};

#[cfg(test)]
mod tests;

pub struct PreProcessor {
    cur_program: Vec<Statement>,
    new_program: Vec<Statement>,
    definitions: HashMap<String, Expression>,
}

impl PreProcessor {
    pub fn new(program: Vec<Statement>) -> Self {
        Self {
            cur_program: program,
            new_program: Vec::new(),
            definitions: HashMap::new(),
        }
    }

    pub fn process(&mut self) -> Result<Vec<Statement>> {
        for stmt in std::mem::take(&mut self.cur_program) {
            match stmt {
                Statement::Define(Expression::Identifier(name), value, _) => {
                    self.definitions.insert(name, value);
                }
                other => self.new_program.push(other),
            }
        }

        self.cur_program = std::mem::take(&mut self.new_program);
        self.new_program = Vec::new();

        for stmt in std::mem::take(&mut self.cur_program) {
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
                Statement::Label(name, span) => Statement::Label(name, span),
                Statement::Nop(span) => Statement::Nop(span),
                Statement::Hlt(span) => Statement::Hlt(span),
                Statement::Db(exprs, span) => Statement::Db(
                    exprs.into_iter().map(|e| self.substitute_expr(e)).collect(),
                    span,
                ),
            };

            self.new_program.push(new_stmt);
        }

        Ok(self.new_program.clone())
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
            other => other,
        }
    }
}
