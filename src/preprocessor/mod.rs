use std::collections::HashMap;

use anyhow::Result;

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
                Statement::Define(key_expr @ Expression::Identifier(_), value_expr) => {
                    self.definitions.insert(
                        if let Expression::Identifier(name) = key_expr {
                            name
                        } else {
                            unreachable!()
                        },
                        value_expr,
                    );
                }
                other => self.new_program.push(other),
            }
        }

        self.cur_program = std::mem::take(&mut self.new_program);
        self.new_program = Vec::new();

        for stmt in std::mem::take(&mut self.cur_program) {
            let new_stmt = match stmt {
                Statement::Mov(dest, src) => {
                    Statement::Mov(self.substitute_expr(dest), self.substitute_expr(src))
                }
                Statement::Ldr(dest, src) => {
                    Statement::Ldr(self.substitute_expr(dest), self.substitute_expr(src))
                }
                Statement::Str(dest, src) => {
                    Statement::Str(self.substitute_expr(dest), self.substitute_expr(src))
                }
                Statement::Push(opt_dest, src) => Statement::Push(
                    opt_dest.map(|e| self.substitute_expr(e)),
                    self.substitute_expr(src),
                ),
                Statement::Pop(opt_dest, src) => Statement::Pop(
                    opt_dest.map(|e| self.substitute_expr(e)),
                    self.substitute_expr(src),
                ),
                Statement::Define(key, value) => {
                    Statement::Define(self.substitute_expr(key), self.substitute_expr(value))
                }
                other => other,
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
