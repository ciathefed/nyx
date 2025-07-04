use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::span::Span;
use miette::{Diagnostic, NamedSource, Result, SourceSpan};

use crate::parser::ast::{BinaryOperator, Expression, Statement};

mod utils;

#[cfg(test)]
mod tests;

#[derive(Debug, thiserror::Error, Diagnostic)]
pub enum Error {
    #[diagnostic(code(preprocessor::include_file_not_found))]
    #[error("Include file not found: {file}")]
    IncludeFileNotFound {
        file: String,
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("include file not found")]
        span: SourceSpan,
    },

    #[diagnostic(code(preprocessor::circular_include))]
    #[error("Circular include detected: {file}")]
    CircularInclude {
        file: String,
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("circular include")]
        span: SourceSpan,
    },

    #[diagnostic(code(preprocessor::include_read_error))]
    #[error("Failed to read include file: {file}")]
    IncludeReadError {
        file: String,
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("failed to read file")]
        span: SourceSpan,
    },

    #[diagnostic(code(preprocessor::unmatched_ifdef))]
    #[error("Unmatched #ifdef directive")]
    UnmatchedIfdef {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("unmatched #ifdef")]
        span: SourceSpan,
    },

    #[diagnostic(code(preprocessor::unmatched_ifndef))]
    #[error("Unmatched #ifndef directive")]
    UnmatchedIfndef {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("unmatched #ifndef")]
        span: SourceSpan,
    },

    #[diagnostic(code(preprocessor::unmatched_else))]
    #[error("Unmatched #else directive")]
    UnmatchedElse {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("unmatched #else")]
        span: SourceSpan,
    },

    #[diagnostic(code(preprocessor::unmatched_endif))]
    #[error("Unmatched #endif directive")]
    UnmatchedEndif {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("unmatched #endif")]
        span: SourceSpan,
    },

    #[diagnostic(code(preprocessor::invalid_define_key))]
    #[error("Invalid define key: expected identifier")]
    InvalidDefineKey {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("expected identifier")]
        span: SourceSpan,
    },

    #[diagnostic(code(preprocessor::invalid_include_path))]
    #[error("Invalid include path: expected string literal")]
    InvalidIncludePath {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("expected string literal")]
        span: SourceSpan,
    },

    #[diagnostic(code(preprocessor::invalid_conditional_expr))]
    #[error("Invalid conditional expression: expected identifier")]
    InvalidConditionalExpr {
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("expected identifier")]
        span: SourceSpan,
    },

    #[diagnostic(code(eval::invalid_operator_for_float))]
    #[error("Invalid operator {op:?} applied to float literals")]
    InvalidOperatorForFloat {
        op: BinaryOperator,
        #[source_code]
        src: Arc<NamedSource<String>>,
        #[label("invalid operator for float")]
        span: SourceSpan,
    },
}

#[derive(Debug, Clone)]
enum ConditionalType {
    IfDef,
    IfNDef,
}

#[derive(Debug, Clone)]
struct ConditionalInfo {
    condition_result: bool,
    seen_else: bool,
    conditional_type: ConditionalType,
    span: Span,
}

pub struct Preprocessor {
    program: Vec<Statement>,
    input: Arc<NamedSource<String>>,
    definitions: HashMap<String, Expression>,
    include_paths: Vec<PathBuf>,
    included_files: HashSet<PathBuf>,
}

impl Preprocessor {
    pub fn new(program: Vec<Statement>, input: Arc<NamedSource<String>>) -> Self {
        Self {
            program,
            input,
            definitions: utils::get_default_definitions(),
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
                Statement::Define(_, _, span) => {
                    return Err(Error::InvalidDefineKey {
                        src: self.input.clone(),
                        span: span.into(),
                    }
                    .into());
                }
                Statement::Include(Expression::StringLiteral(file_path), span) => {
                    let included_statements = self.process_include(&file_path, span)?;
                    processed_statements.extend(included_statements);
                }
                Statement::Include(_expr, span) => {
                    return Err(Error::InvalidIncludePath {
                        src: self.input.clone(),
                        span: span.into(),
                    }
                    .into());
                }
                other => processed_statements.push(other),
            }
        }

        let conditional_statements = self.process_conditionals(processed_statements)?;

        let mut final_statements = Vec::with_capacity(conditional_statements.len());

        for stmt in conditional_statements {
            let new_stmt = match stmt {
                Statement::Label(name, span) => Statement::Label(name, span),
                Statement::Define(key, val, span) => {
                    Statement::Define(self.substitute_expr(key)?, self.substitute_expr(val)?, span)
                }
                Statement::Include(_, _) => {
                    continue;
                }
                Statement::IfDef(_, _)
                | Statement::IfNDef(_, _)
                | Statement::Else(_)
                | Statement::EndIf(_) => {
                    continue;
                }
                Statement::Section(section_type, span) => Statement::Section(section_type, span),
                Statement::Entry(expr, span) => Statement::Entry(self.substitute_expr(expr)?, span),
                Statement::Ascii(expr, span) => Statement::Ascii(self.substitute_expr(expr)?, span),
                Statement::Asciz(expr, span) => Statement::Asciz(self.substitute_expr(expr)?, span),
                Statement::Nop(span) => Statement::Nop(span),
                Statement::Mov(dest, src, span) => Statement::Mov(
                    self.substitute_expr(dest)?,
                    self.substitute_expr(src)?,
                    span,
                ),
                Statement::Ldr(dest, src, span) => Statement::Ldr(
                    self.substitute_expr(dest)?,
                    self.substitute_expr(src)?,
                    span,
                ),
                Statement::Str(dest, src, span) => Statement::Str(
                    self.substitute_expr(dest)?,
                    self.substitute_expr(src)?,
                    span,
                ),
                Statement::Push(size, src, span) => Statement::Push(
                    if let Some(e) = size {
                        Some(self.substitute_expr(e)?)
                    } else {
                        None
                    },
                    self.substitute_expr(src)?,
                    span,
                ),
                Statement::Pop(size, dst, span) => Statement::Pop(
                    if let Some(e) = size {
                        Some(self.substitute_expr(e)?)
                    } else {
                        None
                    },
                    self.substitute_expr(dst)?,
                    span,
                ),
                Statement::Add(dest, lhs, rhs, span) => Statement::Add(
                    self.substitute_expr(dest)?,
                    self.substitute_expr(lhs)?,
                    self.substitute_expr(rhs)?,
                    span,
                ),
                Statement::Sub(dest, lhs, rhs, span) => Statement::Sub(
                    self.substitute_expr(dest)?,
                    self.substitute_expr(lhs)?,
                    self.substitute_expr(rhs)?,
                    span,
                ),
                Statement::Mul(dest, lhs, rhs, span) => Statement::Mul(
                    self.substitute_expr(dest)?,
                    self.substitute_expr(lhs)?,
                    self.substitute_expr(rhs)?,
                    span,
                ),
                Statement::Div(dest, lhs, rhs, span) => Statement::Div(
                    self.substitute_expr(dest)?,
                    self.substitute_expr(lhs)?,
                    self.substitute_expr(rhs)?,
                    span,
                ),
                Statement::And(dest, lhs, rhs, span) => Statement::And(
                    self.substitute_expr(dest)?,
                    self.substitute_expr(lhs)?,
                    self.substitute_expr(rhs)?,
                    span,
                ),
                Statement::Or(dest, lhs, rhs, span) => Statement::Or(
                    self.substitute_expr(dest)?,
                    self.substitute_expr(lhs)?,
                    self.substitute_expr(rhs)?,
                    span,
                ),
                Statement::Xor(dest, lhs, rhs, span) => Statement::Xor(
                    self.substitute_expr(dest)?,
                    self.substitute_expr(lhs)?,
                    self.substitute_expr(rhs)?,
                    span,
                ),
                Statement::Shl(dest, lhs, rhs, span) => Statement::Shl(
                    self.substitute_expr(dest)?,
                    self.substitute_expr(lhs)?,
                    self.substitute_expr(rhs)?,
                    span,
                ),
                Statement::Shr(dest, lhs, rhs, span) => Statement::Shr(
                    self.substitute_expr(dest)?,
                    self.substitute_expr(lhs)?,
                    self.substitute_expr(rhs)?,
                    span,
                ),
                Statement::Cmp(lhs, rhs, span) => {
                    Statement::Cmp(self.substitute_expr(lhs)?, self.substitute_expr(rhs)?, span)
                }
                Statement::Jmp(expr, span) => Statement::Jmp(self.substitute_expr(expr)?, span),
                Statement::Jeq(expr, span) => Statement::Jeq(self.substitute_expr(expr)?, span),
                Statement::Jne(expr, span) => Statement::Jne(self.substitute_expr(expr)?, span),
                Statement::Jlt(expr, span) => Statement::Jlt(self.substitute_expr(expr)?, span),
                Statement::Jgt(expr, span) => Statement::Jgt(self.substitute_expr(expr)?, span),
                Statement::Jle(expr, span) => Statement::Jle(self.substitute_expr(expr)?, span),
                Statement::Jge(expr, span) => Statement::Jge(self.substitute_expr(expr)?, span),
                Statement::Call(expr, span) => Statement::Call(self.substitute_expr(expr)?, span),
                Statement::Ret(span) => Statement::Ret(span),
                Statement::Inc(expr, span) => Statement::Inc(self.substitute_expr(expr)?, span),
                Statement::Dec(expr, span) => Statement::Dec(self.substitute_expr(expr)?, span),
                Statement::Syscall(span) => Statement::Syscall(span),
                Statement::Hlt(span) => Statement::Hlt(span),
                Statement::Db(exprs, span) => Statement::Db(
                    {
                        let mut new_exprs = vec![];
                        for expr in exprs {
                            new_exprs.push(self.substitute_expr(expr)?);
                        }
                        new_exprs
                    },
                    span,
                ),
                Statement::Resb(expr, span) => Statement::Resb(self.substitute_expr(expr)?, span),
            };

            final_statements.push(new_stmt);
        }

        Ok(final_statements)
    }

    fn process_conditionals(&self, statements: Vec<Statement>) -> Result<Vec<Statement>> {
        let mut result = Vec::new();
        let mut stack: Vec<ConditionalInfo> = Vec::new();
        let mut i = 0;

        while i < statements.len() {
            match &statements[i] {
                Statement::IfDef(expr, span) => {
                    let condition_name = match expr {
                        Expression::Identifier(name) => name,
                        _ => {
                            return Err(Error::InvalidConditionalExpr {
                                src: self.input.clone(),
                                span: (*span).into(),
                            }
                            .into());
                        }
                    };

                    let is_defined = self.definitions.contains_key(condition_name);
                    stack.push(ConditionalInfo {
                        condition_result: is_defined,
                        seen_else: false,
                        conditional_type: ConditionalType::IfDef,
                        span: *span,
                    });
                    i += 1;
                }
                Statement::IfNDef(expr, span) => {
                    let condition_name = match expr {
                        Expression::Identifier(name) => name,
                        _ => {
                            return Err(Error::InvalidConditionalExpr {
                                src: self.input.clone(),
                                span: (*span).into(),
                            }
                            .into());
                        }
                    };
                    let is_defined = self.definitions.contains_key(condition_name);
                    stack.push(ConditionalInfo {
                        condition_result: !is_defined,
                        seen_else: false,
                        conditional_type: ConditionalType::IfNDef,
                        span: *span,
                    });
                    i += 1;
                }
                Statement::Else(span) => {
                    if let Some(info) = stack.last_mut() {
                        if info.seen_else {
                            return Err(Error::UnmatchedElse {
                                src: self.input.clone(),
                                span: (*span).into(),
                            }
                            .into());
                        }
                        info.seen_else = true;
                        i += 1;
                    } else {
                        return Err(Error::UnmatchedElse {
                            src: self.input.clone(),
                            span: (*span).into(),
                        }
                        .into());
                    }
                }
                Statement::EndIf(span) => {
                    if stack.pop().is_none() {
                        return Err(Error::UnmatchedEndif {
                            src: self.input.clone(),
                            span: (*span).into(),
                        }
                        .into());
                    }
                    i += 1;
                }
                _ => {
                    if self.should_include_statement_with_info(&stack) {
                        result.push(statements[i].clone());
                    }
                    i += 1;
                }
            }
        }

        if !stack.is_empty() {
            let last_unmatched = stack.last().unwrap();
            return match last_unmatched.conditional_type {
                ConditionalType::IfDef => Err(Error::UnmatchedIfdef {
                    src: self.input.clone(),
                    span: last_unmatched.span.into(),
                }
                .into()),
                ConditionalType::IfNDef => Err(Error::UnmatchedIfndef {
                    src: self.input.clone(),
                    span: last_unmatched.span.into(),
                }
                .into()),
            };
        }

        Ok(result)
    }

    fn should_include_statement_with_info(&self, stack: &[ConditionalInfo]) -> bool {
        for info in stack.iter().rev() {
            if info.seen_else {
                if info.condition_result {
                    return false;
                }
            } else {
                if !info.condition_result {
                    return false;
                }
            }
        }
        true
    }

    fn process_include(&mut self, file_path: &str, span: Span) -> Result<Vec<Statement>> {
        let mut found_path = None;
        for include_dir in &self.include_paths {
            let candidate = include_dir.join(file_path);
            if candidate.exists() {
                found_path = Some(candidate);
                break;
            }
        }

        let path = found_path.ok_or_else(|| Error::IncludeFileNotFound {
            file: file_path.to_string(),
            src: self.input.clone(),
            span: span.into(),
        })?;

        if self.included_files.contains(&path) {
            return Err(Error::CircularInclude {
                file: path.display().to_string(),
                src: self.input.clone(),
                span: span.into(),
            }
            .into());
        }

        let content = fs::read_to_string(&path).map_err(|_| Error::IncludeReadError {
            file: path.display().to_string(),
            src: self.input.clone(),
            span: span.into(),
        })?;

        self.included_files.insert(path.clone());

        let included_statements = self.parse_file_content(&content, &path)?;

        let mut sub_preprocessor = Preprocessor {
            program: included_statements,
            input: Arc::new(NamedSource::new(path.display().to_string(), content)),
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
        let lexer = Lexer::new(Arc::new(NamedSource::new(
            path.to_string_lossy().to_string(),
            content.to_string(),
        )));
        let mut parser = Parser::new(lexer);

        parser.parse().map_err(|e| e.into())
    }

    fn substitute_expr(&self, expr: Expression) -> Result<Expression> {
        match expr {
            Expression::Identifier(name) => {
                if let Some(replacement) = self.definitions.get(&name) {
                    self.substitute_expr(replacement.clone())
                } else {
                    Ok(Expression::Identifier(name))
                }
            }
            Expression::Address(base, offset_opt) => {
                let new_base = Box::new(self.substitute_expr(*base)?);
                let new_offset = if let Some(offset) = offset_opt {
                    Some(Box::new(self.substitute_expr(*offset)?))
                } else {
                    None
                };
                Ok(Expression::Address(new_base, new_offset))
            }
            Expression::Register(reg) => Ok(Expression::Register(reg)),
            Expression::IntegerLiteral(val) => Ok(Expression::IntegerLiteral(val)),
            Expression::FloatLiteral(val) => Ok(Expression::FloatLiteral(val)),
            Expression::StringLiteral(val) => Ok(Expression::StringLiteral(val)),
            Expression::DataSize(size) => Ok(Expression::DataSize(size)),
            Expression::BinaryOp(lhs, op, rhs, span) => {
                let lhs = self.substitute_expr(*lhs)?;
                let rhs = self.substitute_expr(*rhs)?;
                match (lhs, rhs) {
                    (Expression::IntegerLiteral(l), Expression::IntegerLiteral(r)) => {
                        Ok(Expression::IntegerLiteral(match op {
                            BinaryOperator::Add => l + r,
                            BinaryOperator::Sub => l - r,
                            BinaryOperator::Mul => l * r,
                            BinaryOperator::Div => l / r,
                            BinaryOperator::BitOr => l | r,
                            BinaryOperator::BitAnd => l & r,
                            BinaryOperator::BitXor => l ^ r,
                        }))
                    }
                    (Expression::FloatLiteral(l), Expression::FloatLiteral(r)) => {
                        Ok(Expression::FloatLiteral(match op {
                            BinaryOperator::Add => l + r,
                            BinaryOperator::Sub => l - r,
                            BinaryOperator::Mul => l * r,
                            BinaryOperator::Div => l / r,
                            _ => {
                                return Err(Error::InvalidOperatorForFloat {
                                    op: op.clone(),
                                    src: self.input.clone(),
                                    span: span.into(),
                                }
                                .into());
                            }
                        }))
                    }
                    (lhs, rhs) => Ok(Expression::BinaryOp(Box::new(lhs), op, Box::new(rhs), span)),
                }
            }
        }
    }
}
