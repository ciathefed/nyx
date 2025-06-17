use std::{collections::HashMap, fmt};

use anyhow::Result;

use crate::{
    compiler::{bytecode::Bytecode, opcode::Opcode},
    parser::ast::{DataSize, Expression, Statement},
};

pub mod bytecode;
pub mod opcode;

#[cfg(test)]
mod tests;

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
    UnsupportedOperation(&'static str), // instruction name
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
            bytecode: Bytecode::new(),
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
                Statement::Ldr(size, lhs, rhs) => self.compile_ldr(size, lhs, rhs)?,
                Statement::Str(size, lhs, rhs) => self.compile_str(size, lhs, rhs)?,
                Statement::Push(size, expr) => self.compile_push(size, expr)?,
                Statement::Pop(size, expr) => self.compile_pop(size, expr)?,
                Statement::Hlt => self.bytecode.push(Opcode::Hlt),
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

    fn compile_ldr(&mut self, size: Expression, lhs: Expression, rhs: Expression) -> Result<()> {
        const INST: &str = "LDR";
        // Implementation would follow same pattern as compile_mov
        todo!()
    }

    fn compile_str(&mut self, size: Expression, lhs: Expression, rhs: Expression) -> Result<()> {
        const INST: &str = "STR";
        // Implementation would follow same pattern as compile_mov
        todo!()
    }

    fn compile_push(&mut self, size: Expression, expr: Expression) -> Result<()> {
        const INST: &str = "PUSH";
        // Implementation would follow same pattern as compile_mov
        todo!()
    }

    fn compile_pop(&mut self, size: Expression, expr: Expression) -> Result<()> {
        const INST: &str = "POP";
        // Implementation would follow same pattern as compile_mov
        todo!()
    }
}
