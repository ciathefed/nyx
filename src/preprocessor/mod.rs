use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::lexer::Lexer;
use crate::parser::Parser;
use miette::{Diagnostic, NamedSource, Result};

use crate::parser::ast::{Expression, Statement};

#[cfg(test)]
mod tests;

#[derive(Debug, thiserror::Error, Diagnostic)]
pub enum Error {
    #[diagnostic(code(preproccessor::include_file_not_found))]
    #[error("Include file not found: {0}")]
    IncludeFileNotFound(String),

    #[diagnostic(code(preproccessor::circular_include))]
    #[error("Circular include detected: {0}")]
    CircularInclude(String),

    #[diagnostic(code(preproccessor::include_read_error))]
    #[error("Failed to read include file: {0}")]
    IncludeReadError(String),
}

pub struct Preprocessor {
    program: Vec<Statement>,
    definitions: HashMap<String, Expression>,
    include_paths: Vec<PathBuf>,
    included_files: HashSet<PathBuf>,
}

impl Preprocessor {
    pub fn new(program: Vec<Statement>) -> Self {
        Self {
            program,
            definitions: HashMap::new(),
            include_paths: vec![PathBuf::from("")],
            included_files: HashSet::new(),
        }
    }

    pub fn with_include_paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.include_paths = paths;
        self
    }

    pub fn process(&mut self) -> Result<Vec<Statement>> {
        let mut processed_statements = Vec::new();

        for stmt in std::mem::take(&mut self.program) {
            match stmt {
                Statement::Define(Expression::Identifier(name), value, _) => {
                    self.definitions.insert(name, value);
                }
                Statement::Include(Expression::StringLiteral(file_path), _) => {
                    let included_statements = self.process_include(&file_path)?;
                    processed_statements.extend(included_statements);
                }
                other => processed_statements.push(other),
            }
        }

        let mut final_statements = Vec::with_capacity(processed_statements.len());

        for stmt in processed_statements {
            let new_stmt = match stmt {
                Statement::Label(name, span) => Statement::Label(name, span),
                Statement::Define(key, val, span) => {
                    Statement::Define(self.substitute_expr(key), self.substitute_expr(val), span)
                }
                Statement::Section(section_type, span) => Statement::Section(section_type, span),
                Statement::Nop(span) => Statement::Nop(span),
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
                Statement::Add(dest, lhs, rhs, span) => Statement::Add(
                    self.substitute_expr(dest),
                    self.substitute_expr(lhs),
                    self.substitute_expr(rhs),
                    span,
                ),
                Statement::Sub(dest, lhs, rhs, span) => Statement::Sub(
                    self.substitute_expr(dest),
                    self.substitute_expr(lhs),
                    self.substitute_expr(rhs),
                    span,
                ),
                Statement::Mul(dest, lhs, rhs, span) => Statement::Mul(
                    self.substitute_expr(dest),
                    self.substitute_expr(lhs),
                    self.substitute_expr(rhs),
                    span,
                ),
                Statement::Div(dest, lhs, rhs, span) => Statement::Div(
                    self.substitute_expr(dest),
                    self.substitute_expr(lhs),
                    self.substitute_expr(rhs),
                    span,
                ),
                Statement::And(dest, lhs, rhs, span) => Statement::And(
                    self.substitute_expr(dest),
                    self.substitute_expr(lhs),
                    self.substitute_expr(rhs),
                    span,
                ),
                Statement::Or(dest, lhs, rhs, span) => Statement::Or(
                    self.substitute_expr(dest),
                    self.substitute_expr(lhs),
                    self.substitute_expr(rhs),
                    span,
                ),
                Statement::Xor(dest, lhs, rhs, span) => Statement::Xor(
                    self.substitute_expr(dest),
                    self.substitute_expr(lhs),
                    self.substitute_expr(rhs),
                    span,
                ),
                Statement::Shl(dest, lhs, rhs, span) => Statement::Shl(
                    self.substitute_expr(dest),
                    self.substitute_expr(lhs),
                    self.substitute_expr(rhs),
                    span,
                ),
                Statement::Shr(dest, lhs, rhs, span) => Statement::Shr(
                    self.substitute_expr(dest),
                    self.substitute_expr(lhs),
                    self.substitute_expr(rhs),
                    span,
                ),
                Statement::Cmp(lhs, rhs, span) => {
                    Statement::Cmp(self.substitute_expr(lhs), self.substitute_expr(rhs), span)
                }
                Statement::Jmp(expr, span) => Statement::Jmp(self.substitute_expr(expr), span),
                Statement::Jeq(expr, span) => Statement::Jeq(self.substitute_expr(expr), span),
                Statement::Jne(expr, span) => Statement::Jne(self.substitute_expr(expr), span),
                Statement::Jlt(expr, span) => Statement::Jlt(self.substitute_expr(expr), span),
                Statement::Jgt(expr, span) => Statement::Jgt(self.substitute_expr(expr), span),
                Statement::Jle(expr, span) => Statement::Jle(self.substitute_expr(expr), span),
                Statement::Jge(expr, span) => Statement::Jge(self.substitute_expr(expr), span),
                Statement::Call(expr, span) => Statement::Call(self.substitute_expr(expr), span),
                Statement::Ret(span) => Statement::Ret(span),
                Statement::Inc(expr, span) => Statement::Inc(self.substitute_expr(expr), span),
                Statement::Dec(expr, span) => Statement::Dec(self.substitute_expr(expr), span),
                Statement::Syscall(span) => Statement::Syscall(span),
                Statement::Hlt(span) => Statement::Hlt(span),
                Statement::Db(exprs, span) => Statement::Db(
                    exprs.into_iter().map(|e| self.substitute_expr(e)).collect(),
                    span,
                ),
                Statement::Resb(expr, span) => Statement::Resb(self.substitute_expr(expr), span),
                Statement::Include(_, _) => {
                    continue;
                }
            };

            final_statements.push(new_stmt);
        }

        Ok(final_statements)
    }

    fn process_include(&mut self, file_path: &str) -> Result<Vec<Statement>> {
        let mut found_path = None;
        for include_dir in &self.include_paths {
            let candidate = include_dir.join(file_path);
            if candidate.exists() {
                found_path = Some(candidate);
                break;
            }
        }

        let path = found_path.ok_or_else(|| Error::IncludeFileNotFound(file_path.to_string()))?;

        if self.included_files.contains(&path) {
            return Err(Error::CircularInclude(path.display().to_string()).into());
        }

        let content = fs::read_to_string(&path)
            .map_err(|_| Error::IncludeReadError(path.display().to_string()))?;

        self.included_files.insert(path.clone());

        let included_statements = self.parse_file_content(&content, &path)?;

        let mut sub_preprocessor = Preprocessor {
            program: included_statements,
            definitions: self.definitions.clone(),
            include_paths: self.include_paths.clone(),
            included_files: self.included_files.clone(),
        };

        let processed = sub_preprocessor.process()?;

        self.definitions.extend(sub_preprocessor.definitions);
        self.included_files.extend(sub_preprocessor.included_files);

        Ok(processed)
    }

    fn parse_file_content(&self, content: &str, path: &Path) -> Result<Vec<Statement>> {
        let named_source =
            NamedSource::new(path.to_string_lossy().to_string(), content.to_string());
        let lexer = Lexer::new(named_source);
        let mut parser = Parser::new(lexer);

        parser.parse().map_err(|e| e.into())
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
