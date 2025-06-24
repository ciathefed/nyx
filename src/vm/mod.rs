use anyhow::Result;

use crate::{
    compiler::{ADDRESSING_VARIANT_1, ADDRESSING_VARIANT_2, opcode::Opcode},
    parser::ast::{DataSize, Immediate},
    vm::{
        memory::Memory,
        register::{Register, Registers},
    },
};

pub mod memory;
pub mod register;

#[cfg(test)]
mod tests;

#[allow(dead_code)]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid opcode: {0}")]
    InvalidOpcode(u8),
    #[error("unhandled opcode: {0}")]
    UnhandledOpcode(u8),
    #[error("invalid register: {0}")]
    InvalidRegister(u8),
    #[error("invalid data size: {0}")]
    InvalidDataSize(u8),
    #[error("unknown addressing variant: {0}")]
    UnknownAddressingVariant(u8),
    #[error("instruction pointer out of bounds: {0}")]
    InstructionPointerOutOfBounds(usize),
    #[error("stack overflow")]
    StackOverflow,
    #[error("stack underflow")]
    StackUnderflow,
    #[error("unimplemented: {0}")]
    Unimplemented(&'static str),
}

pub struct VM {
    pub(crate) regs: Registers,
    pub(crate) mem: Memory,
    // pub(crate) stack: Stack,
    pub(crate) program: Vec<u8>,
    pub(crate) halted: bool,
}

impl VM {
    pub fn new(program: Vec<u8>, mem_size: usize) -> Self {
        let mut regs = Registers::new();
        regs.sp = mem_size;

        Self {
            regs,
            mem: Memory::new(mem_size),
            program,
            halted: false,
        }
    }

    #[allow(unreachable_patterns)]
    pub fn step(&mut self) -> Result<()> {
        if self.halted {
            return Ok(());
        }

        let byte = self.read_byte()?;
        let opcode = Opcode::try_from(byte).map_err(|_| Error::InvalidOpcode(byte))?;

        match opcode {
            Opcode::Nop => Ok(()),
            Opcode::MovRegReg => {
                let dest = self.read_register()?;
                let src = self.read_register()?;
                self.regs.set(dest, self.regs.get(src))
            }
            Opcode::MovRegImm => {
                let dest = self.read_register()?;
                let src = match DataSize::from(dest) {
                    DataSize::Byte => Immediate::Byte(self.read_byte()?),
                    DataSize::Word => Immediate::Word(self.read_word()?),
                    DataSize::DWord => Immediate::DWord(self.read_dword()?),
                    DataSize::QWord => Immediate::QWord(self.read_qword()?),
                    DataSize::Float => Immediate::Float(self.read_float()?),
                    DataSize::Double => Immediate::Double(self.read_double()?),
                };
                self.regs.set(dest, src)
            }
            Opcode::Ldr => {
                let dest = self.read_register()?;
                let variant = self.read_byte()?;
                let base = match variant {
                    ADDRESSING_VARIANT_1 => {
                        let src = self.read_register()?;
                        self.regs.get(src).as_u64()?
                    }
                    ADDRESSING_VARIANT_2 => self.read_qword()?,
                    _ => return Err(Error::UnknownAddressingVariant(variant).into()),
                };
                let offset = self.read_qword()?;
                let addr = (base + offset) as usize;
                let imm = self.mem.read(addr, DataSize::from(dest))?;
                self.regs.set(dest, imm)
            }
            Opcode::Str => {
                let src = self.read_register()?;
                let value = self.regs.get(src);
                let variant = self.read_byte()?;
                let base = match variant {
                    ADDRESSING_VARIANT_1 => {
                        let dest = self.read_register()?;
                        self.regs.get(dest).as_u64()?
                    }
                    ADDRESSING_VARIANT_2 => self.read_qword()?,
                    _ => return Err(Error::UnknownAddressingVariant(variant).into()),
                };
                let offset = self.read_qword()?;
                let addr = (base + offset) as usize;
                self.mem.write(addr, value, DataSize::from(src))
            }
            Opcode::PushReg => {
                let src = self.read_register()?;
                let size = self.read_data_size()?;
                let imm = match size {
                    DataSize::Byte => Immediate::Byte(self.regs.get(src).as_u8()?),
                    DataSize::Word => Immediate::Word(self.regs.get(src).as_u16()?),
                    DataSize::DWord => Immediate::DWord(self.regs.get(src).as_u32()?),
                    DataSize::QWord => Immediate::QWord(self.regs.get(src).as_u64()?),
                    DataSize::Float => Immediate::Float(self.regs.get(src).as_f32()?),
                    DataSize::Double => Immediate::Double(self.regs.get(src).as_f64()?),
                };
                self.push(imm)
            }
            Opcode::PushImm => {
                let size = self.read_data_size()?;
                let imm = match size {
                    DataSize::Byte => Immediate::Byte(self.read_byte()?),
                    DataSize::Word => Immediate::Word(self.read_word()?),
                    DataSize::DWord => Immediate::DWord(self.read_dword()?),
                    DataSize::QWord => Immediate::QWord(self.read_qword()?),
                    DataSize::Float => Immediate::Float(self.read_float()?),
                    DataSize::Double => Immediate::Double(self.read_double()?),
                };
                self.push(imm)
            }
            Opcode::PushAddr => {
                let size = self.read_data_size()?;
                let variant = self.read_byte()?;
                let base = match variant {
                    ADDRESSING_VARIANT_1 => {
                        let reg = self.read_register()?;
                        self.regs.get(reg).as_u64()?
                    }
                    ADDRESSING_VARIANT_2 => self.read_qword()?,
                    _ => return Err(Error::UnknownAddressingVariant(variant).into()),
                };
                let offset = self.read_qword()?;
                let addr = (base + offset) as usize;
                let value = self.mem.read(addr, size)?;
                self.push(value)
            }
            Opcode::PopReg => {
                let dest = self.read_register()?;
                let size = self.read_data_size()?;
                let value = self.pop(size)?;
                self.regs.set(dest, value)
            }
            Opcode::PopAddr => {
                let size = self.read_data_size()?;
                let variant = self.read_byte()?;
                let base = match variant {
                    ADDRESSING_VARIANT_1 => {
                        let reg = self.read_register()?;
                        self.regs.get(reg).as_u64()?
                    }
                    ADDRESSING_VARIANT_2 => self.read_qword()?,
                    _ => return Err(Error::UnknownAddressingVariant(variant).into()),
                };
                let offset = self.read_qword()?;
                let addr = (base + offset) as usize;
                let value = self.pop(size)?;
                self.mem.write(addr, value, size)
            }
            Opcode::Hlt => {
                self.halted = true;
                Ok(())
            }
            _ => Err(Error::UnhandledOpcode(byte).into()),
        }
    }

    pub fn run(&mut self) -> Result<()> {
        while !self.halted {
            self.step()?;
        }
        Ok(())
    }

    fn read_byte(&mut self) -> Result<u8> {
        let ip = self.regs.ip;
        let byte = *self
            .program
            .get(ip)
            .ok_or(Error::InstructionPointerOutOfBounds(ip))?;
        self.regs.ip += 1;
        Ok(byte)
    }

    fn read_word(&mut self) -> Result<u16> {
        let ip = self.regs.ip;
        let bytes = self
            .program
            .get(ip..ip + 2)
            .ok_or(Error::InstructionPointerOutOfBounds(ip))?;
        self.regs.ip += 2;
        Ok(u16::from_le_bytes(bytes.try_into().unwrap()))
    }

    fn read_dword(&mut self) -> Result<u32> {
        let ip = self.regs.ip;
        let bytes = self
            .program
            .get(ip..ip + 4)
            .ok_or(Error::InstructionPointerOutOfBounds(ip))?;
        self.regs.ip += 4;
        Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
    }

    fn read_qword(&mut self) -> Result<u64> {
        let ip = self.regs.ip;
        let bytes = self
            .program
            .get(ip..ip + 8)
            .ok_or(Error::InstructionPointerOutOfBounds(ip))?;
        self.regs.ip += 8;
        Ok(u64::from_le_bytes(bytes.try_into().unwrap()))
    }

    fn read_float(&mut self) -> Result<f32> {
        let bits = self.read_dword()?;
        Ok(f32::from_bits(bits))
    }

    fn read_double(&mut self) -> Result<f64> {
        let bits = self.read_qword()?;
        Ok(f64::from_bits(bits))
    }

    fn read_register(&mut self) -> Result<Register> {
        let byte = self.read_byte()?;
        Ok(Register::try_from(byte).map_err(|_| Error::InvalidRegister(byte))?)
    }

    fn read_data_size(&mut self) -> Result<DataSize> {
        let byte = self.read_byte()?;
        Ok(DataSize::try_from(byte).map_err(|_| Error::InvalidDataSize(byte))?)
    }

    fn push(&mut self, value: Immediate) -> Result<()> {
        let size = value.size();
        let size_bytes = size.size_in_bytes();

        if self.regs.sp < size_bytes {
            return Err(Error::StackOverflow.into());
        }

        self.regs.sp -= size_bytes;
        self.mem.write(self.regs.sp as usize, value, size)
    }

    fn pop(&mut self, size: DataSize) -> Result<Immediate> {
        if (self.regs.sp as usize) + size.size_in_bytes() > self.mem.storage.len() {
            return Err(Error::StackUnderflow.into());
        }

        let value = self.mem.read(self.regs.sp as usize, size)?;
        self.regs.sp += size.size_in_bytes();
        Ok(value)
    }
}
