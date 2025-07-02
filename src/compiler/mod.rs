use std::{collections::HashMap, ops::Deref};

use miette::{Diagnostic, NamedSource, Result, SourceSpan};

use crate::{
    compiler::{bytecode::Bytecode, opcode::Opcode},
    parser::ast::{DataSize, Expression, SectionType, Statement},
};

pub mod bytecode;
pub mod opcode;

use bytecode::Section;

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
    labels: HashMap<String, (Section, usize)>,
    fixups: HashMap<(Section, usize), (DataSize, String)>,
    current_section: Section,
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
            current_section: Section::Text,
            input,
        }
    }

    pub fn compile(&mut self) -> Result<Vec<u8>> {
        for stmt in std::mem::take(&mut self.program) {
            match stmt {
                Statement::Section(section_type, _) => {
                    self.current_section = match section_type {
                        SectionType::Text => Section::Text,
                        SectionType::Data => Section::Data,
                    };
                }
                Statement::Label(name, _) => {
                    let offset = self.bytecode.len(self.current_section);
                    self.labels.insert(name, (self.current_section, offset));
                }
                Statement::Nop(_) => self.bytecode.push(self.current_section, Opcode::Nop),
                Statement::Mov(lhs, rhs, span) => self.compile_mov(lhs, rhs, span.into())?,
                Statement::Ldr(lhs, rhs, span) => {
                    self.compile_ldr_or_str(lhs, rhs, Opcode::Ldr, span.into())?
                }
                Statement::Str(lhs, rhs, span) => {
                    self.compile_ldr_or_str(lhs, rhs, Opcode::Str, span.into())?
                }
                Statement::Push(ds, expr, span) => self.compile_push(ds, expr, span.into())?,
                Statement::Pop(ds, expr, span) => self.compile_pop(ds, expr, span.into())?,
                Statement::Add(dest, lhs, rhs, span) => {
                    self.compile_arithmetic(dest, lhs, rhs, span.into(), "ADD")?
                }
                Statement::Sub(dest, lhs, rhs, span) => {
                    self.compile_arithmetic(dest, lhs, rhs, span.into(), "SUB")?
                }
                Statement::Mul(dest, lhs, rhs, span) => {
                    self.compile_arithmetic(dest, lhs, rhs, span.into(), "MUL")?
                }
                Statement::Div(dest, lhs, rhs, span) => {
                    self.compile_arithmetic(dest, lhs, rhs, span.into(), "DIV")?
                }
                Statement::And(dest, lhs, rhs, span) => {
                    self.compile_bitwise(dest, lhs, rhs, span.into(), "AND")?
                }
                Statement::Or(dest, lhs, rhs, span) => {
                    self.compile_bitwise(dest, lhs, rhs, span.into(), "OR")?
                }
                Statement::Xor(dest, lhs, rhs, span) => {
                    self.compile_bitwise(dest, lhs, rhs, span.into(), "XOR")?
                }
                Statement::Shl(dest, lhs, rhs, span) => {
                    self.compile_bitwise(dest, lhs, rhs, span.into(), "SHL")?
                }
                Statement::Shr(dest, lhs, rhs, span) => {
                    self.compile_bitwise(dest, lhs, rhs, span.into(), "SHR")?
                }
                Statement::Cmp(lhs, rhs, span) => self.compile_cmp(lhs, rhs, span.into())?,
                Statement::Jmp(expr, span) => self.compile_jump(expr, span.into(), "JMP")?,
                Statement::Jeq(expr, span) => self.compile_jump(expr, span.into(), "JEQ")?,
                Statement::Jne(expr, span) => self.compile_jump(expr, span.into(), "JNE")?,
                Statement::Jlt(expr, span) => self.compile_jump(expr, span.into(), "JLT")?,
                Statement::Jgt(expr, span) => self.compile_jump(expr, span.into(), "JGT")?,
                Statement::Jle(expr, span) => self.compile_jump(expr, span.into(), "JLE")?,
                Statement::Jge(expr, span) => self.compile_jump(expr, span.into(), "JGE")?,
                Statement::Call(expr, span) => self.compile_call(expr, span.into())?,
                Statement::Ret(_) => self.bytecode.push(self.current_section, Opcode::Ret),
                Statement::Inc(expr, span) => {
                    self.compile_inc_or_dec(expr, Opcode::Inc, span.into())?
                }
                Statement::Dec(expr, span) => {
                    self.compile_inc_or_dec(expr, Opcode::Dec, span.into())?
                }
                Statement::Syscall(_) => self.bytecode.push(self.current_section, Opcode::Syscall),
                Statement::Hlt(_) => self.bytecode.push(self.current_section, Opcode::Hlt),
                Statement::Db(exprs, span) => {
                    for expr in exprs {
                        match expr {
                            Expression::IntegerLiteral(integer) => {
                                self.bytecode.push(self.current_section, integer as u8);
                            }
                            Expression::StringLiteral(string) => {
                                self.bytecode.extend(self.current_section, string.bytes());
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
                Statement::Resb(expr, span) => match expr {
                    Expression::IntegerLiteral(integer) => {
                        self.bytecode
                            .extend(self.current_section, vec![0x00; integer as usize]);
                    }
                    _ => {
                        return Err(Error::InvalidExpression {
                            inst: "RESB",
                            src: self.input.clone(),
                            span: span.into(),
                        })?;
                    }
                },
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

        for ((fixup_section, offset), (size, label)) in self.fixups.drain() {
            let (label_section, label_pos) =
                self.labels
                    .get(&label)
                    .ok_or_else(|| Error::UndefinedLabel {
                        inst: "FIXUP",
                        label: label.clone(),
                        src: self.input.clone(),
                        span: SourceSpan::new(offset.into(), 0),
                    })?;

            let absolute_pos = match label_section {
                Section::Text => *label_pos,
                Section::Data => self.bytecode.len(Section::Text) + *label_pos,
            };

            match size {
                DataSize::Byte => {
                    self.bytecode
                        .write_u8_at(fixup_section, offset, absolute_pos as u8)
                }
                DataSize::Word => {
                    self.bytecode
                        .write_u16_at(fixup_section, offset, absolute_pos as u16)
                }
                DataSize::DWord => {
                    self.bytecode
                        .write_u32_at(fixup_section, offset, absolute_pos as u32)
                }
                DataSize::QWord => {
                    self.bytecode
                        .write_u64_at(fixup_section, offset, absolute_pos as u64)
                }
                _ => {
                    return Err(Error::InvalidDataSize {
                        inst: "FIXUP",
                        src: self.input.clone(),
                        span: SourceSpan::new(offset.into(), 0),
                    })?;
                }
            }
        }

        Ok(self.bytecode.finalize())
    }

    fn compile_mov(&mut self, lhs: Expression, rhs: Expression, span: SourceSpan) -> Result<()> {
        const INST: &str = "MOV";

        match (&lhs, &rhs) {
            (Expression::Register(dest), Expression::Register(src)) => {
                self.bytecode.push(self.current_section, Opcode::MovRegReg);
                self.bytecode.push(self.current_section, *dest);
                self.bytecode.push(self.current_section, *src);
            }
            (Expression::Register(dest), Expression::IntegerLiteral(src)) => {
                self.bytecode.push(self.current_section, Opcode::MovRegImm);
                self.bytecode.push(self.current_section, *dest);
                match DataSize::from(*dest) {
                    DataSize::Byte => self.bytecode.push(self.current_section, *src as u8),
                    DataSize::Word => self
                        .bytecode
                        .extend(self.current_section, (*src as u16).to_le_bytes()),
                    DataSize::DWord => self
                        .bytecode
                        .extend(self.current_section, (*src as u32).to_le_bytes()),
                    DataSize::QWord => self
                        .bytecode
                        .extend(self.current_section, (*src as u64).to_le_bytes()),
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
                self.bytecode.push(self.current_section, Opcode::MovRegImm);
                self.bytecode.push(self.current_section, *dest);
                match DataSize::from(*dest) {
                    DataSize::Float => self
                        .bytecode
                        .extend(self.current_section, (*src as f32).to_le_bytes()),
                    DataSize::Double => self
                        .bytecode
                        .extend(self.current_section, src.to_le_bytes()),
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
                self.bytecode.push(self.current_section, Opcode::MovRegImm);
                self.bytecode.push(self.current_section, *dest);
                let size = DataSize::from(*dest);
                let offset = self.bytecode.len(self.current_section);
                self.fixups
                    .insert((self.current_section, offset), (size, src.clone()));
                match DataSize::from(*dest) {
                    DataSize::Byte => self.bytecode.push(self.current_section, 0x00),
                    DataSize::Word => self
                        .bytecode
                        .extend(self.current_section, (0x00_u16).to_le_bytes()),
                    DataSize::DWord => self
                        .bytecode
                        .extend(self.current_section, (0x00_u32).to_le_bytes()),
                    DataSize::QWord => self
                        .bytecode
                        .extend(self.current_section, (0x00_u64).to_le_bytes()),
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
        let inst = match opcode {
            Opcode::Ldr => "LDR",
            Opcode::Str => "STR",
            _ => unreachable!(),
        };

        match (&lhs, &rhs) {
            (Expression::Register(reg), Expression::Address(base_expr, offset_expr)) => {
                match (base_expr.as_ref(), offset_expr.as_ref()) {
                    (Expression::Register(base), Some(offset_box))
                        if matches!(offset_box.as_ref(), Expression::IntegerLiteral(_)) =>
                    {
                        let Expression::IntegerLiteral(offset) = offset_box.as_ref() else {
                            unreachable!()
                        };
                        self.bytecode.push(self.current_section, opcode);
                        self.bytecode.push(self.current_section, *reg);
                        self.bytecode
                            .push(self.current_section, ADDRESSING_VARIANT_1);
                        self.bytecode.push(self.current_section, *base);
                        self.bytecode
                            .extend(self.current_section, offset.to_le_bytes());
                    }
                    (Expression::IntegerLiteral(base), Some(offset_box))
                        if matches!(offset_box.as_ref(), Expression::IntegerLiteral(_)) =>
                    {
                        let Expression::IntegerLiteral(offset) = offset_box.as_ref() else {
                            unreachable!()
                        };
                        self.bytecode.push(self.current_section, opcode);
                        self.bytecode.push(self.current_section, *reg);
                        self.bytecode
                            .push(self.current_section, ADDRESSING_VARIANT_2);
                        self.bytecode
                            .extend(self.current_section, base.to_le_bytes());
                        self.bytecode
                            .extend(self.current_section, offset.to_le_bytes());
                    }
                    (Expression::Identifier(base), Some(offset_box))
                        if matches!(offset_box.as_ref(), Expression::IntegerLiteral(_)) =>
                    {
                        let Expression::IntegerLiteral(offset) = offset_box.as_ref() else {
                            unreachable!()
                        };
                        self.bytecode.push(self.current_section, opcode);
                        self.bytecode.push(self.current_section, *reg);
                        self.bytecode
                            .push(self.current_section, ADDRESSING_VARIANT_2);
                        let fixup_offset = self.bytecode.len(self.current_section);
                        self.fixups.insert(
                            (self.current_section, fixup_offset),
                            (DataSize::QWord, base.clone()),
                        );
                        self.bytecode
                            .extend(self.current_section, (0_u64).to_le_bytes());
                        self.bytecode
                            .extend(self.current_section, offset.to_le_bytes());
                    }
                    (Expression::Register(base), None) => {
                        self.bytecode.push(self.current_section, opcode);
                        self.bytecode.push(self.current_section, *reg);
                        self.bytecode
                            .push(self.current_section, ADDRESSING_VARIANT_1);
                        self.bytecode.push(self.current_section, *base);
                        self.bytecode
                            .extend(self.current_section, (0_u64).to_le_bytes());
                    }
                    (Expression::IntegerLiteral(base), None) => {
                        self.bytecode.push(self.current_section, opcode);
                        self.bytecode.push(self.current_section, *reg);
                        self.bytecode
                            .push(self.current_section, ADDRESSING_VARIANT_2);
                        self.bytecode
                            .extend(self.current_section, base.to_le_bytes());
                        self.bytecode
                            .extend(self.current_section, (0_u64).to_le_bytes());
                    }
                    (Expression::Identifier(base), None) => {
                        self.bytecode.push(self.current_section, opcode);
                        self.bytecode.push(self.current_section, *reg);
                        self.bytecode
                            .push(self.current_section, ADDRESSING_VARIANT_2);
                        let offset = self.bytecode.len(self.current_section);
                        self.bytecode
                            .extend(self.current_section, (0_u64).to_le_bytes());
                        self.fixups.insert(
                            (self.current_section, offset),
                            (DataSize::QWord, base.clone()),
                        );
                        self.bytecode
                            .extend(self.current_section, (0_u64).to_le_bytes());
                    }
                    _ => {
                        return Err(Error::InvalidOperands {
                            inst,
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
                    inst,
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
                self.bytecode.push(self.current_section, Opcode::PushReg);
                self.bytecode
                    .push(self.current_section, DataSize::from(src));
                self.bytecode.push(self.current_section, src);
            }
            (Some(Expression::DataSize(size)), Expression::Register(src)) => {
                self.bytecode.push(self.current_section, Opcode::PushReg);
                self.bytecode.push(self.current_section, size);
                self.bytecode.push(self.current_section, src);
            }
            (None, Expression::Address(base_expr, offset_expr)) => {
                self.bytecode.push(self.current_section, Opcode::PushAddr);

                match (base_expr.deref(), offset_expr.as_deref()) {
                    (Expression::Register(base), Some(Expression::IntegerLiteral(offset))) => {
                        self.bytecode
                            .push(self.current_section, ADDRESSING_VARIANT_1);
                        self.bytecode.push(self.current_section, *base);
                        self.bytecode
                            .extend(self.current_section, offset.to_le_bytes());
                    }
                    (Expression::Register(base), None) => {
                        self.bytecode
                            .push(self.current_section, ADDRESSING_VARIANT_1);
                        self.bytecode.push(self.current_section, *base);
                        self.bytecode
                            .extend(self.current_section, (0_u64).to_le_bytes());
                    }
                    (
                        Expression::IntegerLiteral(base),
                        Some(Expression::IntegerLiteral(offset)),
                    ) => {
                        self.bytecode
                            .push(self.current_section, ADDRESSING_VARIANT_2);
                        self.bytecode
                            .extend(self.current_section, base.to_le_bytes());
                        self.bytecode
                            .extend(self.current_section, offset.to_le_bytes());
                    }
                    (Expression::IntegerLiteral(base), None) => {
                        self.bytecode
                            .push(self.current_section, ADDRESSING_VARIANT_2);
                        self.bytecode
                            .extend(self.current_section, base.to_le_bytes());
                        self.bytecode
                            .extend(self.current_section, (0_u64).to_le_bytes());
                    }
                    (Expression::Identifier(base), Some(Expression::IntegerLiteral(offset))) => {
                        self.bytecode
                            .push(self.current_section, ADDRESSING_VARIANT_2);
                        let fixup_offset = self.bytecode.len(self.current_section);
                        self.bytecode
                            .extend(self.current_section, (0_u64).to_le_bytes());
                        self.fixups.insert(
                            (self.current_section, fixup_offset),
                            (DataSize::QWord, base.clone()),
                        );
                        self.bytecode
                            .extend(self.current_section, offset.to_le_bytes());
                    }
                    (Expression::Identifier(base), None) => {
                        self.bytecode
                            .push(self.current_section, ADDRESSING_VARIANT_2);
                        let fixup_offset = self.bytecode.len(self.current_section);
                        self.bytecode
                            .extend(self.current_section, (0_u64).to_le_bytes());
                        self.fixups.insert(
                            (self.current_section, fixup_offset),
                            (DataSize::QWord, base.clone()),
                        );
                        self.bytecode
                            .extend(self.current_section, (0_u64).to_le_bytes());
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
                self.bytecode.push(self.current_section, Opcode::PushImm);
                self.bytecode.push(self.current_section, size);
                match size {
                    DataSize::Byte => self.bytecode.push(self.current_section, src as u8),
                    DataSize::Word => self
                        .bytecode
                        .extend(self.current_section, (src as u16).to_le_bytes()),
                    DataSize::DWord => self
                        .bytecode
                        .extend(self.current_section, (src as u32).to_le_bytes()),
                    DataSize::QWord => self
                        .bytecode
                        .extend(self.current_section, (src as u64).to_le_bytes()),
                    DataSize::Float => self
                        .bytecode
                        .extend(self.current_section, (src as f32).to_le_bytes()),
                    DataSize::Double => self
                        .bytecode
                        .extend(self.current_section, (src as f64).to_le_bytes()),
                }
            }
            (Some(Expression::DataSize(size)), Expression::FloatLiteral(src)) => {
                self.bytecode.push(self.current_section, Opcode::PushImm);
                self.bytecode.push(self.current_section, size);
                match size {
                    DataSize::Float => self
                        .bytecode
                        .extend(self.current_section, (src as f32).to_le_bytes()),
                    DataSize::Double => self
                        .bytecode
                        .extend(self.current_section, (src as f64).to_le_bytes()),
                    _ => {
                        return Err(Error::InvalidDataSize {
                            inst: INST,
                            src: self.input.clone(),
                            span,
                        })?;
                    }
                }
            }
            (Some(Expression::DataSize(size)), Expression::Identifier(src)) => {
                self.bytecode.push(self.current_section, Opcode::PushImm);
                self.bytecode.push(self.current_section, size);
                let offset = self.bytecode.len(self.current_section);
                self.fixups
                    .insert((self.current_section, offset), (size, src));
                self.bytecode
                    .extend(self.current_section, (0_u64).to_le_bytes());
            }
            (None, Expression::Identifier(src)) => {
                self.bytecode.push(self.current_section, Opcode::PushImm);
                self.bytecode.push(self.current_section, DataSize::QWord);
                let offset = self.bytecode.len(self.current_section);
                self.fixups
                    .insert((self.current_section, offset), (DataSize::QWord, src));
                self.bytecode
                    .extend(self.current_section, (0_u64).to_le_bytes());
            }
            (Some(Expression::DataSize(size)), Expression::Address(base_expr, offset_expr)) => {
                self.bytecode.push(self.current_section, Opcode::PushAddr);
                self.bytecode.push(self.current_section, size);

                match (base_expr.deref(), offset_expr.as_deref()) {
                    (Expression::Register(base), Some(Expression::IntegerLiteral(offset))) => {
                        self.bytecode
                            .push(self.current_section, ADDRESSING_VARIANT_1);
                        self.bytecode.push(self.current_section, *base);
                        self.bytecode
                            .extend(self.current_section, offset.to_le_bytes());
                    }
                    (Expression::Register(base), None) => {
                        self.bytecode
                            .push(self.current_section, ADDRESSING_VARIANT_1);
                        self.bytecode.push(self.current_section, *base);
                        self.bytecode
                            .extend(self.current_section, (0_u64).to_le_bytes());
                    }
                    (
                        Expression::IntegerLiteral(base),
                        Some(Expression::IntegerLiteral(offset)),
                    ) => {
                        self.bytecode
                            .push(self.current_section, ADDRESSING_VARIANT_2);
                        self.bytecode
                            .extend(self.current_section, base.to_le_bytes());
                        self.bytecode
                            .extend(self.current_section, offset.to_le_bytes());
                    }
                    (Expression::IntegerLiteral(base), None) => {
                        self.bytecode
                            .push(self.current_section, ADDRESSING_VARIANT_2);
                        self.bytecode
                            .extend(self.current_section, base.to_le_bytes());
                        self.bytecode
                            .extend(self.current_section, (0_u64).to_le_bytes());
                    }
                    (Expression::Identifier(base), Some(Expression::IntegerLiteral(offset))) => {
                        self.bytecode
                            .push(self.current_section, ADDRESSING_VARIANT_2);
                        let fixup_offset = self.bytecode.len(self.current_section);
                        self.bytecode
                            .extend(self.current_section, (0_u64).to_le_bytes());
                        self.fixups.insert(
                            (self.current_section, fixup_offset),
                            (DataSize::QWord, base.clone()),
                        );
                        self.bytecode
                            .extend(self.current_section, offset.to_le_bytes());
                    }
                    (Expression::Identifier(base), None) => {
                        self.bytecode
                            .push(self.current_section, ADDRESSING_VARIANT_2);
                        let fixup_offset = self.bytecode.len(self.current_section);
                        self.bytecode
                            .extend(self.current_section, (0_u64).to_le_bytes());
                        self.fixups.insert(
                            (self.current_section, fixup_offset),
                            (DataSize::QWord, base.clone()),
                        );
                        self.bytecode
                            .extend(self.current_section, (0_u64).to_le_bytes());
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
                self.bytecode.push(self.current_section, Opcode::PopReg);
                self.bytecode
                    .push(self.current_section, DataSize::from(dest));
                self.bytecode.push(self.current_section, dest);
            }
            (Some(Expression::DataSize(size)), Expression::Register(dest)) => {
                self.bytecode.push(self.current_section, Opcode::PopReg);
                self.bytecode.push(self.current_section, size);
                self.bytecode.push(self.current_section, dest);
            }
            (Some(Expression::DataSize(size)), Expression::Address(base_expr, offset_expr)) => {
                self.bytecode.push(self.current_section, Opcode::PopAddr);
                self.bytecode.push(self.current_section, size);

                match (base_expr.deref(), offset_expr.as_deref()) {
                    (Expression::Register(base), Some(Expression::IntegerLiteral(offset))) => {
                        self.bytecode
                            .push(self.current_section, ADDRESSING_VARIANT_1);
                        self.bytecode.push(self.current_section, *base);
                        self.bytecode
                            .extend(self.current_section, offset.to_le_bytes());
                    }
                    (Expression::Register(base), None) => {
                        self.bytecode
                            .push(self.current_section, ADDRESSING_VARIANT_1);
                        self.bytecode.push(self.current_section, *base);
                        self.bytecode
                            .extend(self.current_section, (0_u64).to_le_bytes());
                    }
                    (
                        Expression::IntegerLiteral(base),
                        Some(Expression::IntegerLiteral(offset)),
                    ) => {
                        self.bytecode
                            .push(self.current_section, ADDRESSING_VARIANT_2);
                        self.bytecode
                            .extend(self.current_section, base.to_le_bytes());
                        self.bytecode
                            .extend(self.current_section, offset.to_le_bytes());
                    }
                    (Expression::IntegerLiteral(base), None) => {
                        self.bytecode
                            .push(self.current_section, ADDRESSING_VARIANT_2);
                        self.bytecode
                            .extend(self.current_section, base.to_le_bytes());
                        self.bytecode
                            .extend(self.current_section, (0_u64).to_le_bytes());
                    }
                    (Expression::Identifier(base), Some(Expression::IntegerLiteral(offset))) => {
                        self.bytecode
                            .push(self.current_section, ADDRESSING_VARIANT_2);
                        let fixup_offset = self.bytecode.len(self.current_section);
                        self.bytecode
                            .extend(self.current_section, (0_u64).to_le_bytes());
                        self.fixups.insert(
                            (self.current_section, fixup_offset),
                            (DataSize::QWord, base.clone()),
                        );
                        self.bytecode
                            .extend(self.current_section, offset.to_le_bytes());
                    }
                    (Expression::Identifier(base), None) => {
                        self.bytecode
                            .push(self.current_section, ADDRESSING_VARIANT_2);
                        let fixup_offset = self.bytecode.len(self.current_section);
                        self.bytecode
                            .extend(self.current_section, (0_u64).to_le_bytes());
                        self.fixups.insert(
                            (self.current_section, fixup_offset),
                            (DataSize::QWord, base.clone()),
                        );
                        self.bytecode
                            .extend(self.current_section, (0_u64).to_le_bytes());
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

    fn compile_arithmetic(
        &mut self,
        dest: Expression,
        lhs: Expression,
        rhs: Expression,
        span: SourceSpan,
        op: &'static str,
    ) -> Result<()> {
        let dest_reg = match dest {
            Expression::Register(reg) => reg,
            _ => {
                return Err(Error::InvalidOperands {
                    inst: op,
                    details: "Destination must be a register".to_string(),
                    src: self.input.clone(),
                    span,
                })?;
            }
        };

        match (&lhs, &rhs) {
            (Expression::Register(lhs_reg), Expression::Register(rhs_reg)) => {
                let opcode = match op {
                    "ADD" => Opcode::AddRegRegReg,
                    "SUB" => Opcode::SubRegRegReg,
                    "MUL" => Opcode::MulRegRegReg,
                    "DIV" => Opcode::DivRegRegReg,
                    _ => unreachable!(),
                };
                self.bytecode.push(self.current_section, opcode);
                self.bytecode.push(self.current_section, dest_reg);
                self.bytecode.push(self.current_section, *lhs_reg);
                self.bytecode.push(self.current_section, *rhs_reg);
            }
            (Expression::Register(lhs_reg), Expression::IntegerLiteral(rhs_val)) => {
                let opcode = match op {
                    "ADD" => Opcode::AddRegRegImm,
                    "SUB" => Opcode::SubRegRegImm,
                    "MUL" => Opcode::MulRegRegImm,
                    "DIV" => Opcode::DivRegRegImm,
                    _ => unreachable!(),
                };
                self.bytecode.push(self.current_section, opcode);
                self.bytecode.push(self.current_section, dest_reg);
                self.bytecode.push(self.current_section, *lhs_reg);
                match DataSize::from(dest_reg) {
                    DataSize::Byte => self.bytecode.push(self.current_section, *rhs_val as u8),
                    DataSize::Word => self
                        .bytecode
                        .extend(self.current_section, (*rhs_val as u16).to_le_bytes()),
                    DataSize::DWord => self
                        .bytecode
                        .extend(self.current_section, (*rhs_val as u32).to_le_bytes()),
                    DataSize::QWord => self
                        .bytecode
                        .extend(self.current_section, (*rhs_val as u64).to_le_bytes()),
                    DataSize::Float => self
                        .bytecode
                        .extend(self.current_section, (*rhs_val as f32).to_le_bytes()),
                    DataSize::Double => self
                        .bytecode
                        .extend(self.current_section, (*rhs_val as f64).to_le_bytes()),
                }
            }
            (Expression::Register(lhs_reg), Expression::FloatLiteral(rhs_val)) => {
                let opcode = match op {
                    "ADD" => Opcode::AddRegRegImm,
                    "SUB" => Opcode::SubRegRegImm,
                    "MUL" => Opcode::MulRegRegImm,
                    "DIV" => Opcode::DivRegRegImm,
                    _ => unreachable!(),
                };
                self.bytecode.push(self.current_section, opcode);
                self.bytecode.push(self.current_section, dest_reg);
                self.bytecode.push(self.current_section, *lhs_reg);
                match DataSize::from(dest_reg) {
                    DataSize::Float => self
                        .bytecode
                        .extend(self.current_section, (*rhs_val as f32).to_le_bytes()),
                    DataSize::Double => self
                        .bytecode
                        .extend(self.current_section, rhs_val.to_le_bytes()),
                    _ => {
                        return Err(Error::InvalidDataSize {
                            inst: op,
                            src: self.input.clone(),
                            span,
                        })?;
                    }
                }
            }
            _ => {
                return Err(Error::InvalidOperands {
                    inst: op,
                    details: format!("Unsupported operand combination: {:?}, {:?}", lhs, rhs),
                    src: self.input.clone(),
                    span,
                })?;
            }
        }
        Ok(())
    }

    fn compile_bitwise(
        &mut self,
        dest: Expression,
        lhs: Expression,
        rhs: Expression,
        span: SourceSpan,
        op: &'static str,
    ) -> Result<()> {
        let dest_reg = match dest {
            Expression::Register(reg) => reg,
            _ => {
                return Err(Error::InvalidOperands {
                    inst: op,
                    details: "Destination must be a register".to_string(),
                    src: self.input.clone(),
                    span,
                })?;
            }
        };

        match DataSize::from(dest_reg) {
            DataSize::Float | DataSize::Double => {
                return Err(Error::InvalidOperands {
                    inst: op,
                    details: "Bitwise operations not supported on floating-point registers"
                        .to_string(),
                    src: self.input.clone(),
                    span,
                })?;
            }
            _ => {}
        }

        match (&lhs, &rhs) {
            (Expression::Register(lhs_reg), Expression::Register(rhs_reg)) => {
                match (DataSize::from(*lhs_reg), DataSize::from(*rhs_reg)) {
                    (DataSize::Float, _)
                    | (DataSize::Double, _)
                    | (_, DataSize::Float)
                    | (_, DataSize::Double) => {
                        return Err(Error::InvalidOperands {
                            inst: op,
                            details: "Bitwise operations not supported on floating-point registers"
                                .to_string(),
                            src: self.input.clone(),
                            span,
                        })?;
                    }
                    _ => {}
                }

                let opcode = match op {
                    "AND" => Opcode::AndRegRegReg,
                    "OR" => Opcode::OrRegRegReg,
                    "XOR" => Opcode::XorRegRegReg,
                    "SHL" => Opcode::ShlRegRegReg,
                    "SHR" => Opcode::ShrRegRegReg,
                    _ => unreachable!(),
                };
                self.bytecode.push(self.current_section, opcode);
                self.bytecode.push(self.current_section, dest_reg);
                self.bytecode.push(self.current_section, *lhs_reg);
                self.bytecode.push(self.current_section, *rhs_reg);
            }
            (Expression::Register(lhs_reg), Expression::IntegerLiteral(rhs_val)) => {
                match DataSize::from(*lhs_reg) {
                    DataSize::Float | DataSize::Double => {
                        return Err(Error::InvalidOperands {
                            inst: op,
                            details: "Bitwise operations not supported on floating-point registers"
                                .to_string(),
                            src: self.input.clone(),
                            span,
                        })?;
                    }
                    _ => {}
                }

                let opcode = match op {
                    "AND" => Opcode::AndRegRegImm,
                    "OR" => Opcode::OrRegRegImm,
                    "XOR" => Opcode::XorRegRegImm,
                    "SHL" => Opcode::ShlRegRegImm,
                    "SHR" => Opcode::ShrRegRegImm,
                    _ => unreachable!(),
                };
                self.bytecode.push(self.current_section, opcode);
                self.bytecode.push(self.current_section, dest_reg);
                self.bytecode.push(self.current_section, *lhs_reg);
                match DataSize::from(dest_reg) {
                    DataSize::Byte => self.bytecode.push(self.current_section, *rhs_val as u8),
                    DataSize::Word => self
                        .bytecode
                        .extend(self.current_section, (*rhs_val as u16).to_le_bytes()),
                    DataSize::DWord => self
                        .bytecode
                        .extend(self.current_section, (*rhs_val as u32).to_le_bytes()),
                    DataSize::QWord => self
                        .bytecode
                        .extend(self.current_section, (*rhs_val as u64).to_le_bytes()),
                    _ => {
                        return Err(Error::InvalidDataSize {
                            inst: op,
                            src: self.input.clone(),
                            span,
                        })?;
                    }
                }
            }
            (Expression::Register(_), Expression::FloatLiteral(_)) => {
                return Err(Error::InvalidOperands {
                    inst: op,
                    details: "Bitwise operations not supported with floating-point operands"
                        .to_string(),
                    src: self.input.clone(),
                    span,
                })?;
            }
            _ => {
                return Err(Error::InvalidOperands {
                    inst: op,
                    details: format!("Unsupported operand combination: {:?}, {:?}", lhs, rhs),
                    src: self.input.clone(),
                    span,
                })?;
            }
        }
        Ok(())
    }

    fn compile_cmp(&mut self, lhs: Expression, rhs: Expression, span: SourceSpan) -> Result<()> {
        const INST: &str = "CMP";

        match (&lhs, &rhs) {
            (Expression::Register(lhs_reg), Expression::IntegerLiteral(rhs_imm)) => {
                self.bytecode.push(self.current_section, Opcode::CmpRegImm);
                self.bytecode.push(self.current_section, *lhs_reg);
                match DataSize::from(*lhs_reg) {
                    DataSize::Byte => self.bytecode.push(self.current_section, *rhs_imm as u8),
                    DataSize::Word => self
                        .bytecode
                        .extend(self.current_section, (*rhs_imm as u16).to_le_bytes()),
                    DataSize::DWord => self
                        .bytecode
                        .extend(self.current_section, (*rhs_imm as u32).to_le_bytes()),
                    DataSize::QWord => self
                        .bytecode
                        .extend(self.current_section, (*rhs_imm as u64).to_le_bytes()),
                    DataSize::Float => self
                        .bytecode
                        .extend(self.current_section, (*rhs_imm as f32).to_le_bytes()),
                    DataSize::Double => self
                        .bytecode
                        .extend(self.current_section, (*rhs_imm as f64).to_le_bytes()),
                }
            }
            (Expression::Register(lhs_reg), Expression::FloatLiteral(rhs_imm)) => {
                self.bytecode.push(self.current_section, Opcode::CmpRegImm);
                self.bytecode.push(self.current_section, *lhs_reg);
                match DataSize::from(*lhs_reg) {
                    DataSize::Float => self
                        .bytecode
                        .extend(self.current_section, (*rhs_imm as f32).to_le_bytes()),
                    DataSize::Double => self
                        .bytecode
                        .extend(self.current_section, rhs_imm.to_le_bytes()),
                    _ => {
                        return Err(Error::InvalidDataSize {
                            inst: INST,
                            src: self.input.clone(),
                            span,
                        })?;
                    }
                }
            }
            (Expression::Register(lhs_reg), Expression::Register(rhs_reg)) => {
                self.bytecode.push(self.current_section, Opcode::CmpRegReg);
                self.bytecode.push(self.current_section, *lhs_reg);
                self.bytecode.push(self.current_section, *rhs_reg);
            }
            _ => {
                return Err(Error::InvalidOperands {
                    inst: INST,
                    details: format!("Unsupported operand combination: {:?}, {:?}", lhs, rhs),
                    src: self.input.clone(),
                    span,
                })?;
            }
        }
        Ok(())
    }

    fn compile_jump(&mut self, expr: Expression, span: SourceSpan, op: &'static str) -> Result<()> {
        match expr {
            Expression::IntegerLiteral(src) => {
                let opcode = match op {
                    "JMP" => Opcode::JmpImm,
                    "JEQ" => Opcode::JeqImm,
                    "JNE" => Opcode::JneImm,
                    "JLT" => Opcode::JltImm,
                    "JGT" => Opcode::JgtImm,
                    "JLE" => Opcode::JleImm,
                    "JGE" => Opcode::JgeImm,
                    _ => unreachable!(),
                };

                self.bytecode.push(self.current_section, opcode);
                self.bytecode
                    .extend(self.current_section, (src as u64).to_le_bytes());
            }
            Expression::Register(src) => {
                let opcode = match op {
                    "JMP" => Opcode::JmpReg,
                    "JEQ" => Opcode::JeqReg,
                    "JNE" => Opcode::JneReg,
                    "JLT" => Opcode::JltReg,
                    "JGT" => Opcode::JgtReg,
                    "JLE" => Opcode::JleReg,
                    "JGE" => Opcode::JgeReg,
                    _ => unreachable!(),
                };

                self.bytecode.push(self.current_section, opcode);
                self.bytecode.push(self.current_section, src);
            }
            Expression::Identifier(src) => {
                let opcode = match op {
                    "JMP" => Opcode::JmpImm,
                    "JEQ" => Opcode::JeqImm,
                    "JNE" => Opcode::JneImm,
                    "JLT" => Opcode::JltImm,
                    "JGT" => Opcode::JgtImm,
                    "JLE" => Opcode::JleImm,
                    "JGE" => Opcode::JgeImm,
                    _ => unreachable!(),
                };

                self.bytecode.push(self.current_section, opcode);
                let offset = self.bytecode.len(self.current_section);
                self.bytecode
                    .extend(self.current_section, (0_u64).to_le_bytes());
                self.fixups.insert(
                    (self.current_section, offset),
                    (DataSize::QWord, src.clone()),
                );
            }
            _ => {
                return Err(Error::InvalidOperands {
                    inst: op,
                    details: format!("Unsupported operand: {:?}", expr),
                    src: self.input.clone(),
                    span,
                })?;
            }
        }
        Ok(())
    }

    fn compile_call(&mut self, expr: Expression, span: SourceSpan) -> Result<()> {
        const INST: &'static str = "CALL";

        match expr {
            Expression::IntegerLiteral(src) => {
                self.bytecode.push(self.current_section, Opcode::CallImm);
                self.bytecode
                    .extend(self.current_section, (src as u64).to_le_bytes());
            }
            Expression::Register(src) => {
                self.bytecode.push(self.current_section, Opcode::CallReg);
                self.bytecode.push(self.current_section, src);
            }
            Expression::Identifier(src) => {
                self.bytecode.push(self.current_section, Opcode::CallImm);
                let offset = self.bytecode.len(self.current_section);
                self.bytecode
                    .extend(self.current_section, (0_u64).to_le_bytes());
                self.fixups.insert(
                    (self.current_section, offset),
                    (DataSize::QWord, src.clone()),
                );
            }
            _ => {
                return Err(Error::InvalidOperands {
                    inst: INST,
                    details: format!("Unsupported operand: {:?}", expr),
                    src: self.input.clone(),
                    span,
                })?;
            }
        }
        Ok(())
    }

    fn compile_inc_or_dec(
        &mut self,
        expr: Expression,
        opcode: Opcode,
        span: SourceSpan,
    ) -> Result<()> {
        let inst = match opcode {
            Opcode::Inc => "INC",
            Opcode::Dec => "DEC",
            _ => unreachable!(),
        };

        match expr {
            Expression::Register(src) => {
                self.bytecode.push(self.current_section, opcode);
                self.bytecode.push(self.current_section, src);
            }
            _ => {
                return Err(Error::InvalidOperands {
                    inst,
                    details: format!("Unsupported operand: {:?}", expr),
                    src: self.input.clone(),
                    span,
                })?;
            }
        }
        Ok(())
    }
}
