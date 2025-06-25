use std::{collections::HashMap, ops::Deref};

use miette::{Diagnostic, NamedSource, Result, SourceSpan};

use crate::{
    compiler::{bytecode::Bytecode, opcode::Opcode},
    parser::ast::{DataSize, Expression, Statement},
};

pub mod bytecode;
pub mod opcode;

#[cfg(test)]
mod tests;

pub const ADDRESSING_VARIANT_1: u8 = 0x00; // [REGISTER, Option<INTEGER>]
pub const ADDRESSING_VARIANT_2: u8 = 0x01; // [INTEGER, Option<INTEGER>]

#[allow(dead_code)]
#[derive(Debug, thiserror::Error, Diagnostic)]
pub enum Error {
    #[diagnostic(code(compiler::invalid_register))]
    #[error("invalid register in {inst}")]
    InvalidRegister {
        inst: &'static str,
        #[source_code]
        src: NamedSource<String>,
        #[label("invalid register used here")]
        span: SourceSpan,
    },

    #[diagnostic(code(compiler::invalid_data_size))]
    #[error("invalid data size in {inst}")]
    InvalidDataSize {
        inst: &'static str,
        #[source_code]
        src: NamedSource<String>,
        #[label("invalid data size")]
        span: SourceSpan,
    },

    #[diagnostic(code(compiler::invalid_operands))]
    #[error("invalid operands in {inst}: {details}")]
    InvalidOperands {
        inst: &'static str,
        details: String,
        #[source_code]
        src: NamedSource<String>,
        #[label("invalid operands")]
        span: SourceSpan,
    },

    #[diagnostic(code(compiler::undefined_label))]
    #[error("undefined label in {inst}: {label}")]
    UndefinedLabel {
        inst: &'static str,
        label: String,
        #[source_code]
        src: NamedSource<String>,
        #[label("undefined label used here")]
        span: SourceSpan,
    },

    #[diagnostic(code(compiler::unsupported_op))]
    #[error("unsupported operation {inst}")]
    UnsupportedOperation {
        inst: String,
        #[source_code]
        src: NamedSource<String>,
        #[label("unsupported operation")]
        span: SourceSpan,
    },

    #[diagnostic(code(compiler::fixup_fail))]
    #[error("fixup failed in {inst} for label: {label}")]
    FixupFailure {
        inst: &'static str,
        label: String,
        #[source_code]
        src: NamedSource<String>,
        #[label("fixup failure target")]
        span: SourceSpan,
    },

    #[diagnostic(code(compiler::invalid_expression))]
    #[error("invalid expression in {inst}")]
    InvalidExpression {
        inst: &'static str,
        #[source_code]
        src: NamedSource<String>,
        #[label("invalid expression")]
        span: SourceSpan,
    },
}

pub struct Compiler {
    program: Vec<Statement>,
    bytecode: Bytecode,
    labels: HashMap<String, usize>,
    fixups: HashMap<usize, (DataSize, String)>,
    input: NamedSource<String>,
}

impl Compiler {
    pub fn new(program: Vec<Statement>, input: NamedSource<String>) -> Self {
        let program_len = program.len();
        Self {
            program,
            bytecode: Bytecode::new(Some(4 * program_len)),
            labels: HashMap::with_capacity(4 * program_len),
            fixups: HashMap::with_capacity(4 * program_len),
            input,
        }
    }

    pub fn compile(&mut self) -> Result<&[u8]> {
        for stmt in std::mem::take(&mut self.program) {
            match stmt {
                Statement::Label(name, _) => {
                    self.labels.insert(name, self.bytecode.len());
                }
                Statement::Nop(_) => self.bytecode.push(Opcode::Nop),
                Statement::Mov(lhs, rhs, span) => self.compile_mov(lhs, rhs, span.into())?,
                Statement::Ldr(lhs, rhs, span) => {
                    self.compile_ldr_or_str(lhs, rhs, Opcode::Ldr, span.into())?
                }
                Statement::Str(lhs, rhs, span) => {
                    self.compile_ldr_or_str(lhs, rhs, Opcode::Str, span.into())?
                }
                Statement::Push(ds, expr, span) => self.compile_push(ds, expr, span.into())?,
                Statement::Pop(ds, expr, span) => self.compile_pop(ds, expr, span.into())?,
                Statement::Hlt(_) => self.bytecode.push(Opcode::Hlt),
                Statement::Db(exprs, span) => {
                    for expr in exprs {
                        match expr {
                            Expression::IntegerLiteral(integer) => {
                                self.bytecode.push(integer as u8);
                            }
                            Expression::StringLiteral(string) => {
                                self.bytecode.extend(string.bytes());
                            }
                            _ => {
                                return Err(Error::InvalidExpression {
                                    inst: "DB",
                                    src: self.input.clone(),
                                    span: span.into(),
                                })?;
                            }
                        }
                    }
                }
                other => {
                    let span = other.span().into();
                    return Err(Error::UnsupportedOperation {
                        inst: format!("{:?}", other),
                        src: self.input.clone(),
                        span,
                    })?;
                }
            }
        }

        for (offset, (size, label)) in self.fixups.drain() {
            let label_pos = self
                .labels
                .get(&label)
                .ok_or_else(|| Error::UndefinedLabel {
                    inst: "FIXUP",
                    label: label.clone(),
                    src: self.input.clone(),
                    span: SourceSpan::new(offset.into(), 0),
                })?;

            match size {
                DataSize::Byte => self.bytecode.write_u8_at(offset, *label_pos as u8),
                DataSize::Word => self.bytecode.write_u16_at(offset, *label_pos as u16),
                DataSize::DWord => self.bytecode.write_u32_at(offset, *label_pos as u32),
                DataSize::QWord => self.bytecode.write_u64_at(offset, *label_pos as u64),
                _ => {
                    return Err(Error::InvalidDataSize {
                        inst: "FIXUP",
                        src: self.input.clone(),
                        span: SourceSpan::new(offset.into(), 0),
                    })?;
                }
            }
        }

        Ok(&self.bytecode.storage)
    }

    fn compile_mov(&mut self, lhs: Expression, rhs: Expression, span: SourceSpan) -> Result<()> {
        const INST: &str = "MOV";

        match (&lhs, &rhs) {
            (Expression::Register(dest), Expression::Register(src)) => {
                self.bytecode.push(Opcode::MovRegReg);
                self.bytecode.push(*dest);
                self.bytecode.push(*src);
            }
            (Expression::Register(dest), Expression::IntegerLiteral(src)) => {
                self.bytecode.push(Opcode::MovRegImm);
                self.bytecode.push(*dest);
                match DataSize::from(*dest) {
                    DataSize::Byte => self.bytecode.push(*src as u8),
                    DataSize::Word => self.bytecode.extend((*src as u16).to_le_bytes()),
                    DataSize::DWord => self.bytecode.extend((*src as u32).to_le_bytes()),
                    DataSize::QWord => self.bytecode.extend((*src as u64).to_le_bytes()),
                    _ => {
                        return Err(Error::InvalidDataSize {
                            inst: INST,
                            src: self.input.clone(),
                            span,
                        })?;
                    }
                }
            }
            (Expression::Register(dest), Expression::FloatLiteral(src)) => {
                self.bytecode.push(Opcode::MovRegImm);
                self.bytecode.push(*dest);
                match DataSize::from(*dest) {
                    DataSize::Float => self.bytecode.extend((*src as f32).to_le_bytes()),
                    DataSize::Double => self.bytecode.extend(src.to_le_bytes()),
                    _ => {
                        return Err(Error::InvalidDataSize {
                            inst: INST,
                            src: self.input.clone(),
                            span,
                        })?;
                    }
                }
            }
            (Expression::Register(dest), Expression::Identifier(src)) => {
                self.bytecode.push(Opcode::MovRegImm);
                self.bytecode.push(*dest);
                let size = DataSize::from(*dest);
                self.fixups.insert(self.bytecode.len(), (size, src.clone()));
                match DataSize::from(*dest) {
                    DataSize::Byte => self.bytecode.push(0x00),
                    DataSize::Word => self.bytecode.extend((0x00_u16).to_le_bytes()),
                    DataSize::DWord => self.bytecode.extend((0x00_u32).to_le_bytes()),
                    DataSize::QWord => self.bytecode.extend((0x00_u64).to_le_bytes()),
                    _ => {
                        return Err(Error::InvalidDataSize {
                            inst: INST,
                            src: self.input.clone(),
                            span,
                        })?;
                    }
                }
            }
            (lhs, rhs) => {
                return Err(Error::InvalidOperands {
                    inst: INST,
                    details: format!("Unsupported operands: {:?} -> {:?}", lhs, rhs),
                    src: self.input.clone(),
                    span,
                })?;
            }
        }
        Ok(())
    }

    fn compile_ldr_or_str(
        &mut self,
        lhs: Expression,
        rhs: Expression,
        opcode: Opcode,
        span: SourceSpan,
    ) -> Result<()> {
        const INST: &str = "STR";

        match (&lhs, &rhs) {
            (Expression::Register(reg), Expression::Address(base_expr, offset_expr)) => {
                match (base_expr.as_ref(), offset_expr.as_ref()) {
                    (Expression::Register(base), Some(offset_box))
                        if matches!(offset_box.as_ref(), Expression::IntegerLiteral(_)) =>
                    {
                        let Expression::IntegerLiteral(offset) = offset_box.as_ref() else {
                            unreachable!()
                        };
                        self.bytecode.push(opcode);
                        self.bytecode.push(*reg);
                        self.bytecode.push(ADDRESSING_VARIANT_1);
                        self.bytecode.push(*base);
                        self.bytecode.extend(offset.to_le_bytes());
                    }
                    (Expression::IntegerLiteral(base), Some(offset_box))
                        if matches!(offset_box.as_ref(), Expression::IntegerLiteral(_)) =>
                    {
                        let Expression::IntegerLiteral(offset) = offset_box.as_ref() else {
                            unreachable!()
                        };
                        self.bytecode.push(opcode);
                        self.bytecode.push(*reg);
                        self.bytecode.push(ADDRESSING_VARIANT_2);
                        self.bytecode.extend(base.to_le_bytes());
                        self.bytecode.extend(offset.to_le_bytes());
                    }
                    (Expression::Identifier(base), Some(offset_box))
                        if matches!(offset_box.as_ref(), Expression::IntegerLiteral(_)) =>
                    {
                        let Expression::IntegerLiteral(offset) = offset_box.as_ref() else {
                            unreachable!()
                        };
                        self.bytecode.push(opcode);
                        self.bytecode.push(*reg);
                        self.bytecode.push(ADDRESSING_VARIANT_2);
                        self.bytecode.extend((0x00 as i64).to_le_bytes());
                        self.fixups
                            .insert(self.bytecode.len() - 8, (DataSize::QWord, base.clone()));
                        self.bytecode.extend(offset.to_le_bytes());
                    }
                    (Expression::Register(base), None) => {
                        self.bytecode.push(opcode);
                        self.bytecode.push(*reg);
                        self.bytecode.push(ADDRESSING_VARIANT_1);
                        self.bytecode.push(*base);
                        self.bytecode.extend((0x00 as u64).to_le_bytes());
                    }
                    (Expression::IntegerLiteral(base), None) => {
                        self.bytecode.push(opcode);
                        self.bytecode.push(*reg);
                        self.bytecode.push(ADDRESSING_VARIANT_2);
                        self.bytecode.extend(base.to_le_bytes());
                        self.bytecode.extend((0x00 as u64).to_le_bytes());
                    }
                    (Expression::Identifier(base), None) => {
                        self.bytecode.push(opcode);
                        self.bytecode.push(*reg);
                        self.bytecode.push(ADDRESSING_VARIANT_2);
                        self.bytecode.extend((0x00 as i64).to_le_bytes());
                        self.fixups
                            .insert(self.bytecode.len() - 8, (DataSize::QWord, base.clone()));
                        self.bytecode.extend((0x00 as u64).to_le_bytes());
                    }
                    _ => {
                        return Err(Error::InvalidOperands {
                            inst: INST,
                            details: format!(
                                "Unsupported addressing operands: {:?} -> {:?}",
                                base_expr, offset_expr
                            ),
                            src: self.input.clone(),
                            span,
                        })?;
                    }
                }
            }
            (lhs, rhs) => {
                return Err(Error::InvalidOperands {
                    inst: INST,
                    details: format!("Unsupported operands: {:?} -> {:?}", lhs, rhs),
                    src: self.input.clone(),
                    span,
                })?;
            }
        }
        Ok(())
    }

    fn compile_push(
        &mut self,
        ds: Option<Expression>,
        expr: Expression,
        span: SourceSpan,
    ) -> Result<()> {
        const INST: &str = "PUSH";

        match (ds, expr) {
            (None, Expression::Register(src)) => {
                self.bytecode.push(Opcode::PushReg);
                self.bytecode.push(DataSize::from(src));
                self.bytecode.push(src);
            }
            (Some(Expression::DataSize(size)), Expression::Register(src)) => {
                self.bytecode.push(Opcode::PushReg);
                self.bytecode.push(size);
                self.bytecode.push(src);
            }
            (None, Expression::Address(base_expr, offset_expr)) => {
                self.bytecode.push(Opcode::PushAddr);

                match (base_expr.deref(), offset_expr.as_deref()) {
                    (Expression::Register(base), Some(Expression::IntegerLiteral(offset))) => {
                        self.bytecode.push(ADDRESSING_VARIANT_1);
                        self.bytecode.push(*base);
                        self.bytecode.extend(offset.to_le_bytes());
                    }
                    (Expression::Register(base), None) => {
                        self.bytecode.push(ADDRESSING_VARIANT_1);
                        self.bytecode.push(*base);
                        self.bytecode.extend((0x00 as u64).to_le_bytes());
                    }
                    (
                        Expression::IntegerLiteral(base),
                        Some(Expression::IntegerLiteral(offset)),
                    ) => {
                        self.bytecode.push(ADDRESSING_VARIANT_2);
                        self.bytecode.extend(base.to_le_bytes());
                        self.bytecode.extend(offset.to_le_bytes());
                    }
                    (Expression::IntegerLiteral(base), None) => {
                        self.bytecode.push(ADDRESSING_VARIANT_2);
                        self.bytecode.extend(base.to_le_bytes());
                        self.bytecode.extend((0x00 as u64).to_le_bytes());
                    }
                    (Expression::Identifier(base), Some(Expression::IntegerLiteral(offset))) => {
                        self.bytecode.push(ADDRESSING_VARIANT_2);
                        self.bytecode.extend((0x00 as i64).to_le_bytes());
                        self.fixups
                            .insert(self.bytecode.len() - 8, (DataSize::QWord, base.clone()));
                        self.bytecode.extend(offset.to_le_bytes());
                    }
                    (Expression::Identifier(base), None) => {
                        self.bytecode.push(ADDRESSING_VARIANT_2);
                        self.bytecode.extend((0x00 as i64).to_le_bytes());
                        self.fixups
                            .insert(self.bytecode.len() - 8, (DataSize::QWord, base.clone()));
                        self.bytecode.extend((0x00 as u64).to_le_bytes());
                    }
                    _ => {
                        return Err(Error::InvalidOperands {
                            inst: INST,
                            details: format!(
                                "Unsupported addressing operands: {:?} -> {:?}",
                                base_expr, offset_expr
                            ),
                            src: self.input.clone(),
                            span,
                        })?;
                    }
                }
            }
            (Some(Expression::DataSize(size)), Expression::IntegerLiteral(src)) => {
                self.bytecode.push(Opcode::PushImm);
                self.bytecode.push(size);
                match size {
                    DataSize::Byte => self.bytecode.push(src as u8),
                    DataSize::Word => self.bytecode.extend((src as u16).to_le_bytes()),
                    DataSize::DWord => self.bytecode.extend((src as u32).to_le_bytes()),
                    DataSize::QWord => self.bytecode.extend((src as u64).to_le_bytes()),
                    DataSize::Float => self.bytecode.extend((src as f32).to_le_bytes()),
                    DataSize::Double => self.bytecode.extend((src as f64).to_le_bytes()),
                }
            }
            (Some(Expression::DataSize(size)), Expression::Identifier(src)) => {
                self.bytecode.push(Opcode::PushImm);
                self.bytecode.push(size);
                self.fixups.insert(self.bytecode.len(), (size, src));
                self.bytecode.extend((0x00 as u64).to_le_bytes());
            }
            (None, Expression::Identifier(src)) => {
                self.bytecode.push(Opcode::PushImm);
                self.bytecode.push(DataSize::QWord);
                self.fixups
                    .insert(self.bytecode.len(), (DataSize::QWord, src));
                self.bytecode.extend((0x00 as u64).to_le_bytes());
            }
            (Some(Expression::DataSize(size)), Expression::Address(base_expr, offset_expr)) => {
                self.bytecode.push(Opcode::PushAddr);
                self.bytecode.push(size);

                match (base_expr.deref(), offset_expr.as_deref()) {
                    (Expression::Register(base), Some(Expression::IntegerLiteral(offset))) => {
                        self.bytecode.push(ADDRESSING_VARIANT_1);
                        self.bytecode.push(*base);
                        self.bytecode.extend(offset.to_le_bytes());
                    }
                    (Expression::Register(base), None) => {
                        self.bytecode.push(ADDRESSING_VARIANT_1);
                        self.bytecode.push(*base);
                        self.bytecode.extend((0x00 as u64).to_le_bytes());
                    }
                    (
                        Expression::IntegerLiteral(base),
                        Some(Expression::IntegerLiteral(offset)),
                    ) => {
                        self.bytecode.push(ADDRESSING_VARIANT_2);
                        self.bytecode.extend(base.to_le_bytes());
                        self.bytecode.extend(offset.to_le_bytes());
                    }
                    (Expression::IntegerLiteral(base), None) => {
                        self.bytecode.push(ADDRESSING_VARIANT_2);
                        self.bytecode.extend(base.to_le_bytes());
                        self.bytecode.extend((0x00 as u64).to_le_bytes());
                    }
                    (Expression::Identifier(base), Some(Expression::IntegerLiteral(offset))) => {
                        self.bytecode.push(ADDRESSING_VARIANT_2);
                        self.bytecode.extend((0x00 as i64).to_le_bytes());
                        self.fixups
                            .insert(self.bytecode.len() - 8, (DataSize::QWord, base.clone()));
                        self.bytecode.extend(offset.to_le_bytes());
                    }
                    (Expression::Identifier(base), None) => {
                        self.bytecode.push(ADDRESSING_VARIANT_2);
                        self.bytecode.extend((0x00 as i64).to_le_bytes());
                        self.fixups
                            .insert(self.bytecode.len() - 8, (DataSize::QWord, base.clone()));
                        self.bytecode.extend((0x00 as u64).to_le_bytes());
                    }
                    _ => {
                        return Err(Error::InvalidOperands {
                            inst: INST,
                            details: format!(
                                "Unsupported addressing operands with size: {:?} -> {:?}",
                                base_expr, offset_expr
                            ),
                            src: self.input.clone(),
                            span,
                        })?;
                    }
                }
            }
            (ds, expr) => {
                return Err(Error::InvalidOperands {
                    inst: INST,
                    details: format!("Unsupported data size and operand: {:?} -> {:?}", ds, expr),
                    src: self.input.clone(),
                    span,
                })?;
            }
        }
        Ok(())
    }

    fn compile_pop(
        &mut self,
        ds: Option<Expression>,
        expr: Expression,
        span: SourceSpan,
    ) -> Result<()> {
        const INST: &str = "POP";

        match (ds, expr) {
            (None, Expression::Register(dest)) => {
                self.bytecode.push(Opcode::PopReg);
                self.bytecode.push(DataSize::from(dest));
                self.bytecode.push(dest);
            }
            (Some(Expression::DataSize(size)), Expression::Register(dest)) => {
                self.bytecode.push(Opcode::PopReg);
                self.bytecode.push(size);
                self.bytecode.push(dest);
            }
            (Some(Expression::DataSize(size)), Expression::Address(base_expr, offset_expr)) => {
                self.bytecode.push(Opcode::PopAddr);
                self.bytecode.push(size);

                match (base_expr.deref(), offset_expr.as_deref()) {
                    (Expression::Register(base), Some(Expression::IntegerLiteral(offset))) => {
                        self.bytecode.push(ADDRESSING_VARIANT_1);
                        self.bytecode.push(*base);
                        self.bytecode.extend(offset.to_le_bytes());
                    }
                    (Expression::Register(base), None) => {
                        self.bytecode.push(ADDRESSING_VARIANT_1);
                        self.bytecode.push(*base);
                        self.bytecode.extend((0x00 as u64).to_le_bytes());
                    }
                    (
                        Expression::IntegerLiteral(base),
                        Some(Expression::IntegerLiteral(offset)),
                    ) => {
                        self.bytecode.push(ADDRESSING_VARIANT_2);
                        self.bytecode.extend(base.to_le_bytes());
                        self.bytecode.extend(offset.to_le_bytes());
                    }
                    (Expression::IntegerLiteral(base), None) => {
                        self.bytecode.push(ADDRESSING_VARIANT_2);
                        self.bytecode.extend(base.to_le_bytes());
                        self.bytecode.extend((0x00 as u64).to_le_bytes());
                    }
                    (Expression::Identifier(base), Some(Expression::IntegerLiteral(offset))) => {
                        self.bytecode.push(ADDRESSING_VARIANT_2);
                        self.bytecode.extend((0x00 as i64).to_le_bytes());
                        self.fixups
                            .insert(self.bytecode.len() - 8, (DataSize::QWord, base.clone()));
                        self.bytecode.extend(offset.to_le_bytes());
                    }
                    (Expression::Identifier(base), None) => {
                        self.bytecode.push(ADDRESSING_VARIANT_2);
                        self.bytecode.extend((0x00 as i64).to_le_bytes());
                        self.fixups
                            .insert(self.bytecode.len() - 8, (DataSize::QWord, base.clone()));
                        self.bytecode.extend((0x00 as u64).to_le_bytes());
                    }
                    _ => {
                        return Err(Error::InvalidOperands {
                            inst: INST,
                            details: format!(
                                "Unsupported addressing operands: {:?} -> {:?}",
                                base_expr, offset_expr
                            ),
                            src: self.input.clone(),
                            span,
                        })?;
                    }
                }
            }
            (ds, expr) => {
                return Err(Error::InvalidOperands {
                    inst: INST,
                    details: format!("Unsupported data size and operand: {:?} -> {:?}", ds, expr),
                    src: self.input.clone(),
                    span,
                })?;
            }
        }
        Ok(())
    }
}
