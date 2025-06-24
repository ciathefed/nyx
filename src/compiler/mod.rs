use std::{collections::HashMap, ops::Deref};

use anyhow::Result;

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
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid register in {0}")]
    InvalidRegister(&'static str), // instruction name
    #[error("Invalid data size in {0}")]
    InvalidDataSize(&'static str), // instruction name
    #[error("Invalid operands in {0}: {1}")]
    InvalidOperands(&'static str, String), // (instruction, details)
    #[error("Undefined label in {0}: {1}")]
    UndefinedLabel(&'static str, String), // (instruction, label)
    #[error("Invalid immediate in {0}")]
    InvalidImmediate(&'static str), // instruction name
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String), // instruction name
    #[error("Fixup failed in {0} for label: {1}")]
    FixupFailure(&'static str, String), // (instruction, label)
    #[error("Invalid expression in {0}")]
    InvalidExpression(&'static str), // instruction name
}

pub struct Compiler {
    program: Vec<Statement>,
    bytecode: Bytecode,
    labels: HashMap<String, usize>,
    fixups: HashMap<usize, (DataSize, String)>,
}

impl Compiler {
    pub fn new(program: Vec<Statement>) -> Self {
        let program_len = program.len();
        Self {
            program,
            bytecode: Bytecode::new(Some(4 * program_len)),
            labels: HashMap::with_capacity(4 * program_len),
            fixups: HashMap::with_capacity(4 * program_len),
        }
    }

    pub fn compile(&mut self) -> Result<&[u8]> {
        for stmt in std::mem::take(&mut self.program) {
            match stmt {
                Statement::Label(name) => {
                    self.labels.insert(name, self.bytecode.len());
                }
                Statement::Nop => self.bytecode.push(Opcode::Nop),
                Statement::Mov(lhs, rhs) => self.compile_mov(lhs, rhs)?,
                Statement::Ldr(lhs, rhs) => self.compile_ldr_or_str(lhs, rhs, Opcode::Ldr)?,
                Statement::Str(lhs, rhs) => self.compile_ldr_or_str(lhs, rhs, Opcode::Str)?,
                Statement::Push(ds, expr) => self.compile_push(ds, expr)?,
                Statement::Pop(ds, expr) => self.compile_pop(ds, expr)?,
                Statement::Hlt => self.bytecode.push(Opcode::Hlt),
                other => return Err(Error::UnsupportedOperation(format!("{:?}", other)).into()),
            }
        }

        for (offset, (size, label)) in self.fixups.drain() {
            let label_pos = self
                .labels
                .get(&label)
                .ok_or_else(|| Error::UndefinedLabel("FIXUP", label.clone()))?;

            match size {
                DataSize::Byte => self.bytecode.write_u8_at(offset, *label_pos as u8),
                DataSize::Word => self.bytecode.write_u16_at(offset, *label_pos as u16),
                DataSize::DWord => self.bytecode.write_u32_at(offset, *label_pos as u32),
                DataSize::QWord => self.bytecode.write_u64_at(offset, *label_pos as u64),
                _ => return Err(Error::InvalidDataSize("FIXUP").into()),
            }
        }

        Ok(&self.bytecode.storage)
    }

    fn compile_mov(&mut self, lhs: Expression, rhs: Expression) -> Result<()> {
        const INST: &str = "MOV";

        match (lhs, rhs) {
            (Expression::Register(dest), Expression::Register(src)) => {
                self.bytecode.push(Opcode::MovRegReg);
                self.bytecode.push(dest);
                self.bytecode.push(src);
            }
            (Expression::Register(dest), Expression::IntegerLiteral(src)) => {
                self.bytecode.push(Opcode::MovRegImm);
                self.bytecode.push(dest);
                match DataSize::from(dest) {
                    DataSize::Byte => self.bytecode.push(src as u8),
                    DataSize::Word => self.bytecode.extend((src as u16).to_le_bytes()),
                    DataSize::DWord => self.bytecode.extend((src as u32).to_le_bytes()),
                    DataSize::QWord => self.bytecode.extend((src as u64).to_le_bytes()),
                    _ => return Err(Error::InvalidDataSize(INST).into()),
                }
            }
            (Expression::Register(dest), Expression::FloatLiteral(src)) => {
                self.bytecode.push(Opcode::MovRegImm);
                self.bytecode.push(dest);
                match DataSize::from(dest) {
                    DataSize::Float => self.bytecode.extend((src as f32).to_le_bytes()),
                    DataSize::Double => self.bytecode.extend(src.to_le_bytes()),
                    _ => return Err(Error::InvalidDataSize(INST).into()),
                }
            }
            (Expression::Register(dest), Expression::Identifier(src)) => {
                self.bytecode.push(Opcode::MovRegImm);
                self.bytecode.push(dest);
                let size = DataSize::from(dest);
                self.fixups.insert(self.bytecode.len(), (size, src));
                match DataSize::from(dest) {
                    DataSize::Byte => self.bytecode.push(0x00),
                    DataSize::Word => self.bytecode.extend((0x00 as u16).to_le_bytes()),
                    DataSize::DWord => self.bytecode.extend((0x00 as u32).to_le_bytes()),
                    DataSize::QWord => self.bytecode.extend((0x00 as u64).to_le_bytes()),
                    _ => return Err(Error::InvalidDataSize(INST).into()),
                }
            }
            (lhs, rhs) => {
                return Err(Error::InvalidOperands(
                    INST,
                    format!("unsupported operands: {:?} -> {:?}", lhs, rhs),
                )
                .into());
            }
        }
        Ok(())
    }

    fn compile_ldr_or_str(
        &mut self,
        lhs: Expression,
        rhs: Expression,
        opcode: Opcode,
    ) -> Result<()> {
        const INST: &str = "STR";

        match (lhs, rhs) {
            (Expression::Register(reg), Expression::Address(base_expr, offset_expr)) => {
                match (base_expr.deref(), offset_expr.as_deref()) {
                    (Expression::Register(base), Some(Expression::IntegerLiteral(offset))) => {
                        self.bytecode.push(opcode);
                        self.bytecode.push(reg);
                        self.bytecode.push(ADDRESSING_VARIANT_1);
                        self.bytecode.push(*base);
                        self.bytecode.extend(offset.to_le_bytes());
                    }
                    (
                        Expression::IntegerLiteral(base),
                        Some(Expression::IntegerLiteral(offset)),
                    ) => {
                        self.bytecode.push(opcode);
                        self.bytecode.push(reg);
                        self.bytecode.push(ADDRESSING_VARIANT_2);
                        self.bytecode.extend(base.to_le_bytes());
                        self.bytecode.extend(offset.to_le_bytes());
                    }
                    (Expression::Identifier(base), Some(Expression::IntegerLiteral(offset))) => {
                        self.bytecode.push(opcode);
                        self.bytecode.push(reg);
                        self.bytecode.push(ADDRESSING_VARIANT_2);
                        self.bytecode.extend((0x00 as i64).to_le_bytes());
                        self.fixups
                            .insert(self.bytecode.len(), (DataSize::QWord, base.clone()));
                        self.bytecode.extend(offset.to_le_bytes());
                    }
                    (Expression::Register(base), None) => {
                        self.bytecode.push(opcode);
                        self.bytecode.push(reg);
                        self.bytecode.push(ADDRESSING_VARIANT_1);
                        self.bytecode.push(*base);
                        self.bytecode.extend((0x00 as u64).to_le_bytes());
                    }
                    (Expression::IntegerLiteral(base), None) => {
                        self.bytecode.push(opcode);
                        self.bytecode.push(reg);
                        self.bytecode.push(ADDRESSING_VARIANT_1);
                        self.bytecode.extend(base.to_le_bytes());
                        self.bytecode.extend((0x00 as u64).to_le_bytes());
                    }
                    (Expression::Identifier(base), None) => {
                        self.bytecode.push(opcode);
                        self.bytecode.push(reg);
                        self.bytecode.push(ADDRESSING_VARIANT_2);
                        self.bytecode.extend((0x00 as i64).to_le_bytes());
                        self.fixups
                            .insert(self.bytecode.len(), (DataSize::QWord, base.clone()));
                        self.bytecode.extend((0x00 as u64).to_le_bytes());
                    }
                    _ => {
                        return Err(Error::InvalidOperands(
                            INST,
                            format!(
                                "unsupported addressing operands: {:?} -> {:?}",
                                base_expr, offset_expr
                            ),
                        )
                        .into());
                    }
                }
            }
            (lhs, rhs) => {
                return Err(Error::InvalidOperands(
                    INST,
                    format!("unsupported operands: {:?} -> {:?}", lhs, rhs),
                )
                .into());
            }
        }
        Ok(())
    }

    fn compile_push(&mut self, ds: Option<Expression>, expr: Expression) -> Result<()> {
        const INST: &str = "PUSH";

        match (ds, expr) {
            (None, Expression::Register(src)) => {
                self.bytecode.push(Opcode::PushReg);
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
                        return Err(Error::InvalidOperands(
                            INST,
                            format!(
                                "unsupported addressing operands: {:?} -> {:?}",
                                base_expr, offset_expr
                            ),
                        )
                        .into());
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
                self.bytecode.push(size); // first byte is the data size

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
                        return Err(Error::InvalidOperands(
                            INST,
                            format!(
                                "unsupported addressing operands with size: {:?} -> {:?}",
                                base_expr, offset_expr
                            ),
                        )
                        .into());
                    }
                }
            }
            (ds, expr) => {
                return Err(Error::InvalidOperands(
                    INST,
                    format!("unsupported data size and operand: {:?} -> {:?}", ds, expr),
                )
                .into());
            }
        }
        Ok(())
    }

    fn compile_pop(&mut self, ds: Option<Expression>, expr: Expression) -> Result<()> {
        const INST: &str = "POP";

        match (ds, expr) {
            (None, Expression::Register(dest)) => {
                self.bytecode.push(Opcode::PopReg);
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
                        return Err(Error::InvalidOperands(
                            INST,
                            format!(
                                "unsupported addressing operands: {:?} -> {:?}",
                                base_expr, offset_expr
                            ),
                        )
                        .into());
                    }
                }
            }
            (ds, expr) => {
                return Err(Error::InvalidOperands(
                    INST,
                    format!("unsupported data size and operand: {:?} -> {:?}", ds, expr),
                )
                .into());
            }
        }
        Ok(())
    }
}
