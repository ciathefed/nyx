use anyhow::Result;

use crate::{
    compiler::{ADDRESSING_VARIANT_1, ADDRESSING_VARIANT_2, opcode::Opcode},
    parser::ast::{DataSize, Immediate},
    vm::{
        memory::Memory,
        register::{Register, Registers},
        stack::Stack,
    },
};

pub mod memory;
pub mod register;
pub mod stack;

#[cfg(test)]
mod tests;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid opcode: {0}")]
    InvalidOpcode(u8),
    #[error("unhandled opcode: {0}")]
    UnhandledOpcode(u8),
    #[error("invalid register: {0}")]
    InvalidRegister(u8),
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
    pub(crate) stack: Stack,
    pub(crate) program: Vec<u8>,
    pub(crate) halted: bool,
}

impl VM {
    pub fn new(program: Vec<u8>, mem_size: usize) -> Self {
        Self {
            regs: Registers::new(),
            mem: Memory::new(mem_size),
            stack: Stack::new(),
            program,
            halted: false,
        }
    }

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
                let imm = match DataSize::from(dest) {
                    DataSize::Byte => Immediate::Byte(self.read_byte()?),
                    DataSize::Word => Immediate::Word(self.read_word()?),
                    DataSize::DWord => Immediate::DWord(self.read_dword()?),
                    DataSize::QWord => Immediate::QWord(self.read_qword()?),
                    DataSize::Float => Immediate::Float(self.read_float()?),
                    DataSize::Double => Immediate::Double(self.read_double()?),
                };
                self.regs.set(dest, imm)
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
            Opcode::Hlt => {
                self.halted = true;
                Ok(())
            }
            _ => Err(Error::UnhandledOpcode(byte).into()),
        }

        // match instr {
        //     Instruction::Hlt => self.halted = true,
        //     Instruction::MovRegReg(dst, src) => {
        //         let value = self.regs.get(*src);
        //         self.regs.set(*dst, value)?;
        //     }
        //     Instruction::MovRegImm(dst, imm) => {
        //         self.regs.set(*dst, *imm)?;
        //     }
        //     Instruction::Ldr(size, dst, addr) => {
        //         let value = self.mem.read(*addr, *size)?;
        //         self.regs.set(*dst, value)?;
        //     }
        //     Instruction::Str(size, src, addr) => {
        //         let value = self.regs.get(*src);
        //         self.mem.write(*addr, value, *size)?;
        //     }
        //     Instruction::PushImm(imm) => {
        //         self.stack.push(*imm)?;
        //         self.regs.sp += 1;
        //     }
        //     Instruction::PushReg(reg) => {
        //         let val = self.regs.get(*reg);
        //         self.stack.push(val)?;
        //         self.regs.sp += 1;
        //     }
        //     Instruction::PushAddr(addr, size) => {
        //         let val = self.mem.read(*addr, *size)?;
        //         self.stack.push(val)?;
        //         self.regs.sp += 1;
        //     }
        //     Instruction::PopReg(dst) => {
        //         let val = self.stack.pop()?;
        //         self.regs.set(*dst, val)?;
        //         self.regs.sp = self.regs.sp.saturating_sub(1);
        //     }
        //     Instruction::PopAddr(addr, size) => {
        //         let val = self.stack.pop()?;
        //         self.mem.write(*addr, val, *size)?;
        //         self.regs.sp = self.regs.sp.saturating_sub(1);
        //     }
        // }

        // Ok(())
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
}
