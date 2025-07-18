use miette::{Diagnostic, Result};

use crate::{
    compiler::{ADDRESSING_VARIANT_1, ADDRESSING_VARIANT_2, opcode::Opcode},
    parser::ast::{DataSize, Immediate},
    vm::{
        flags::Flags,
        memory::Memory,
        register::{Register, Registers},
        syscall::{Syscalls, collect_syscalls},
    },
};

pub mod flags;
pub mod memory;
pub mod register;
pub mod syscall;

#[macro_use]
mod macros;

#[cfg(test)]
mod tests;

#[allow(dead_code)]
#[derive(Debug, thiserror::Error, Diagnostic)]
enum Error {
    #[diagnostic(code(vm::invalid_opcode))]
    #[error("invalid opcode: {0}")]
    InvalidOpcode(u8),

    #[diagnostic(code(vm::unknown_opcode))]
    #[error("unhandled opcode: {0}")]
    UnhandledOpcode(u8),

    #[diagnostic(code(vm::invalid_register))]
    #[error("invalid register: {0}")]
    InvalidRegister(u8),

    #[diagnostic(code(vm::invalid_data_size))]
    #[error("invalid data size: {0}")]
    InvalidDataSize(u8),

    #[diagnostic(code(vm::unknown_addressing_variant))]
    #[error("unknown addressing variant: {0}")]
    UnknownAddressingVariant(u8),

    #[diagnostic(code(vm::instruction_pointer_out_of_bounds))]
    #[error("instruction pointer out of bounds: {0}")]
    InstructionPointerOutOfBounds(usize),

    #[diagnostic(code(vm::stack_overflow))]
    #[error("stack overflow")]
    StackOverflow,

    #[diagnostic(code(vm::stack_underflow))]
    #[error("stack underflow")]
    StackUnderflow,

    #[diagnostic(code(vm::unknown_syscall))]
    #[error("unknown syscall: {0}")]
    UnknownSyscall(usize),

    #[diagnostic(code(vm::io_error))]
    #[error("I/O error")]
    IoError(#[from] std::io::Error),

    #[diagnostic(code(vm::program_too_small))]
    #[error("program too small: expected at least 8 bytes for entry point, got {0} bytes")]
    ProgramTooSmall(usize),

    #[diagnostic(code(vm::invalid_entry_point))]
    #[error(
        "invalid entry point: 0x{entry_point:x} is outside program bounds (program size: {program_size} bytes)"
    )]
    InvalidEntryPoint {
        entry_point: u64,
        program_size: usize,
    },

    #[diagnostic(code(vm::program_too_large))]
    #[error(
        "program too large: {program_size} bytes exceeds available memory ({memory_size} bytes)"
    )]
    ProgramTooLarge {
        program_size: usize,
        memory_size: usize,
    },

    #[diagnostic(code(vm::unimplemented))]
    #[error("unimplemented: {0}")]
    Unimplemented(&'static str),
}

pub struct VM {
    pub(crate) regs: Registers,
    pub(crate) mem: Memory,
    pub(crate) flags: Flags,
    pub(crate) syscalls: Syscalls,
    pub(crate) halted: bool,
}

impl VM {
    pub fn new(program: Vec<u8>, mem_size: usize) -> Result<Self> {
        if program.len() < 8 {
            return Err(Error::ProgramTooSmall(program.len()))?;
        }

        let entry_point = u64::from_le_bytes([
            program[0], program[1], program[2], program[3], program[4], program[5], program[6],
            program[7],
        ]);

        let program_data = &program[8..];

        if entry_point as usize >= program_data.len() {
            return Err(Error::InvalidEntryPoint {
                entry_point,
                program_size: program_data.len(),
            })?;
        }

        if program_data.len() > mem_size {
            return Err(Error::ProgramTooLarge {
                program_size: program_data.len(),
                memory_size: mem_size,
            })?;
        }

        let mut regs = Registers::new();
        regs.set_sp(mem_size);
        regs.set_bp(0);
        regs.set_ip(entry_point as usize);

        let mut mem = Memory::new(mem_size);

        mem.storage[..program_data.len()].copy_from_slice(program_data);

        Ok(Self {
            regs,
            mem,
            flags: Flags::new(),
            syscalls: collect_syscalls(),
            halted: false,
        })
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
                let size = self.read_data_size()?;
                let src = self.read_register()?;
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
                let size = self.read_data_size()?;
                let dest = self.read_register()?;
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
            Opcode::AddRegRegReg => binary_arithmetic_op!(self, wrapping_add, +),
            Opcode::AddRegRegImm => binary_arithmetic_op_imm!(self, wrapping_add, +),
            Opcode::SubRegRegReg => binary_arithmetic_op!(self, wrapping_sub, -),
            Opcode::SubRegRegImm => binary_arithmetic_op_imm!(self, wrapping_sub, -),
            Opcode::MulRegRegReg => binary_arithmetic_op!(self, wrapping_mul, *),
            Opcode::MulRegRegImm => binary_arithmetic_op_imm!(self, wrapping_mul, *),
            Opcode::DivRegRegReg => binary_arithmetic_op!(self, wrapping_div, /),
            Opcode::DivRegRegImm => binary_arithmetic_op_imm!(self, wrapping_div, /),
            Opcode::AndRegRegReg => binary_bitwise_op!(self, &),
            Opcode::AndRegRegImm => binary_bitwise_op_imm!(self, &),
            Opcode::OrRegRegReg => binary_bitwise_op!(self, |),
            Opcode::OrRegRegImm => binary_bitwise_op_imm!(self, |),
            Opcode::XorRegRegReg => binary_bitwise_op!(self, ^),
            Opcode::XorRegRegImm => binary_bitwise_op_imm!(self, ^),
            Opcode::ShlRegRegReg => shift_op!(self, <<),
            Opcode::ShlRegRegImm => shift_op_imm!(self, <<),
            Opcode::ShrRegRegReg => shift_op!(self, >>),
            Opcode::ShrRegRegImm => shift_op_imm!(self, >>),
            Opcode::CmpRegImm => {
                let reg = self.read_register()?;
                let lhs = self.regs.get(reg);
                let rhs = match DataSize::from(reg) {
                    DataSize::Byte => Immediate::Byte(self.read_byte()?),
                    DataSize::Word => Immediate::Word(self.read_word()?),
                    DataSize::DWord => Immediate::DWord(self.read_dword()?),
                    DataSize::QWord => Immediate::QWord(self.read_qword()?),
                    DataSize::Float => Immediate::Float(self.read_float()?),
                    DataSize::Double => Immediate::Double(self.read_double()?),
                };
                self.flags.eq = lhs == rhs;
                self.flags.lt = lhs < rhs;
                Ok(())
            }
            Opcode::CmpRegReg => {
                let reg = self.read_register()?;
                let lhs = self.regs.get(reg);
                let reg = self.read_register()?;
                let rhs = self.regs.get(reg);
                self.flags.eq = lhs == rhs;
                self.flags.lt = lhs < rhs;
                Ok(())
            }
            Opcode::JmpImm => {
                let addr = self.read_qword()?;
                self.regs.set_ip(addr as usize);
                Ok(())
            }
            Opcode::JmpReg => {
                let reg = self.read_register()?;
                let addr = self.regs.get(reg).as_usize()?;
                self.regs.set_ip(addr);
                Ok(())
            }
            Opcode::JeqImm => {
                let addr = self.read_qword()?;
                if self.flags.eq {
                    self.regs.set_ip(addr as usize);
                }
                Ok(())
            }
            Opcode::JeqReg => {
                let reg = self.read_register()?;
                let addr = self.regs.get(reg).as_usize()?;
                if self.flags.eq {
                    self.regs.set_ip(addr);
                }
                Ok(())
            }
            Opcode::JneImm => {
                let addr = self.read_qword()?;
                if !self.flags.eq {
                    self.regs.set_ip(addr as usize);
                }
                Ok(())
            }
            Opcode::JneReg => {
                let reg = self.read_register()?;
                let addr = self.regs.get(reg).as_usize()?;
                if !self.flags.eq {
                    self.regs.set_ip(addr);
                }
                Ok(())
            }
            Opcode::JltImm => {
                let addr = self.read_qword()?;
                if self.flags.lt {
                    self.regs.set_ip(addr as usize);
                }
                Ok(())
            }
            Opcode::JltReg => {
                let reg = self.read_register()?;
                let addr = self.regs.get(reg).as_usize()?;
                if self.flags.lt {
                    self.regs.set_ip(addr);
                }
                Ok(())
            }
            Opcode::JgtImm => {
                let addr = self.read_qword()?;
                if !self.flags.lt {
                    self.regs.set_ip(addr as usize);
                }
                Ok(())
            }
            Opcode::JgtReg => {
                let reg = self.read_register()?;
                let addr = self.regs.get(reg).as_usize()?;
                if !self.flags.lt {
                    self.regs.set_ip(addr);
                }
                Ok(())
            }
            Opcode::JleImm => {
                let addr = self.read_qword()?;
                if self.flags.lt || self.flags.eq {
                    self.regs.set_ip(addr as usize);
                }
                Ok(())
            }
            Opcode::JleReg => {
                let reg = self.read_register()?;
                let addr = self.regs.get(reg).as_usize()?;
                if self.flags.lt || self.flags.eq {
                    self.regs.set_ip(addr);
                }
                Ok(())
            }
            Opcode::JgeImm => {
                let addr = self.read_qword()?;
                if !self.flags.lt || self.flags.eq {
                    self.regs.set_ip(addr as usize);
                }
                Ok(())
            }
            Opcode::JgeReg => {
                let reg = self.read_register()?;
                let addr = self.regs.get(reg).as_usize()?;
                if !self.flags.lt || self.flags.eq {
                    self.regs.set_ip(addr);
                }
                Ok(())
            }
            Opcode::CallImm => {
                let addr = self.read_qword()?;
                self.push(Immediate::QWord(self.regs.ip() as u64))?;
                self.regs.set_ip(addr as usize);
                Ok(())
            }
            Opcode::CallReg => {
                let reg = self.read_register()?;
                let addr = self.regs.get(reg).as_usize()?;
                self.push(Immediate::QWord(self.regs.ip() as u64))?;
                self.regs.set_ip(addr as usize);
                Ok(())
            }
            Opcode::Inc => {
                let reg = self.read_register()?;
                let value = self.regs.get(reg);
                let new_value = match value {
                    Immediate::Byte(imm) => Immediate::Byte(imm.wrapping_add(1)),
                    Immediate::Word(imm) => Immediate::Word(imm.wrapping_add(1)),
                    Immediate::DWord(imm) => Immediate::DWord(imm.wrapping_add(1)),
                    Immediate::QWord(imm) => Immediate::QWord(imm.wrapping_add(1)),
                    Immediate::Float(imm) => Immediate::Float(imm + 1.0),
                    Immediate::Double(imm) => Immediate::Double(imm + 1.0),
                };
                self.regs.set(reg, new_value)?;
                Ok(())
            }
            Opcode::Dec => {
                let reg = self.read_register()?;
                let value = self.regs.get(reg);
                let new_value = match value {
                    Immediate::Byte(imm) => Immediate::Byte(imm.wrapping_sub(1)),
                    Immediate::Word(imm) => Immediate::Word(imm.wrapping_sub(1)),
                    Immediate::DWord(imm) => Immediate::DWord(imm.wrapping_sub(1)),
                    Immediate::QWord(imm) => Immediate::QWord(imm.wrapping_sub(1)),
                    Immediate::Float(imm) => Immediate::Float(imm - 1.0),
                    Immediate::Double(imm) => Immediate::Double(imm - 1.0),
                };
                self.regs.set(reg, new_value)?;
                Ok(())
            }
            Opcode::Ret => {
                let addr = self.pop(DataSize::QWord)?.as_usize()?;
                self.regs.set_ip(addr);
                Ok(())
            }
            Opcode::Syscall => {
                let index = self.regs.get(Register::Q15).as_usize()?;
                let syscall = if let Some(syscall) = self.syscalls.get(&index) {
                    syscall
                } else {
                    return Err(Error::UnknownSyscall(index))?;
                };
                syscall(self)?;
                Ok(())
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

    #[inline]
    fn read_byte(&mut self) -> Result<u8> {
        let ip = self.regs.ip();
        if ip >= self.mem.storage.len() {
            return Err(Error::InstructionPointerOutOfBounds(ip).into());
        }
        let byte = unsafe { *self.mem.storage.get_unchecked(ip) };
        self.regs.set_ip(ip + 1);
        Ok(byte)
    }

    #[inline]
    fn read_word(&mut self) -> Result<u16> {
        let ip = self.regs.ip();
        if ip + 2 > self.mem.storage.len() {
            return Err(Error::InstructionPointerOutOfBounds(ip).into());
        }
        let bytes = unsafe {
            [
                *self.mem.storage.get_unchecked(ip),
                *self.mem.storage.get_unchecked(ip + 1),
            ]
        };
        self.regs.set_ip(ip + 2);
        Ok(u16::from_le_bytes(bytes))
    }

    #[inline]
    fn read_dword(&mut self) -> Result<u32> {
        let ip = self.regs.ip();
        if ip + 4 > self.mem.storage.len() {
            return Err(Error::InstructionPointerOutOfBounds(ip).into());
        }
        let bytes = unsafe {
            [
                *self.mem.storage.get_unchecked(ip),
                *self.mem.storage.get_unchecked(ip + 1),
                *self.mem.storage.get_unchecked(ip + 2),
                *self.mem.storage.get_unchecked(ip + 3),
            ]
        };
        self.regs.set_ip(ip + 4);
        Ok(u32::from_le_bytes(bytes))
    }

    #[inline]
    fn read_qword(&mut self) -> Result<u64> {
        let ip = self.regs.ip();
        if ip + 8 > self.mem.storage.len() {
            return Err(Error::InstructionPointerOutOfBounds(ip).into());
        }
        let bytes = unsafe {
            [
                *self.mem.storage.get_unchecked(ip),
                *self.mem.storage.get_unchecked(ip + 1),
                *self.mem.storage.get_unchecked(ip + 2),
                *self.mem.storage.get_unchecked(ip + 3),
                *self.mem.storage.get_unchecked(ip + 4),
                *self.mem.storage.get_unchecked(ip + 5),
                *self.mem.storage.get_unchecked(ip + 6),
                *self.mem.storage.get_unchecked(ip + 7),
            ]
        };
        self.regs.set_ip(ip + 8);
        Ok(u64::from_le_bytes(bytes))
    }

    #[inline]
    fn read_float(&mut self) -> Result<f32> {
        let bits = self.read_dword()?;
        Ok(f32::from_bits(bits))
    }

    #[inline]
    fn read_double(&mut self) -> Result<f64> {
        let bits = self.read_qword()?;
        Ok(f64::from_bits(bits))
    }

    #[inline]
    fn read_register(&mut self) -> Result<Register> {
        let byte = self.read_byte()?;
        Register::try_from(byte).map_err(|_| Error::InvalidRegister(byte).into())
    }

    #[inline]
    fn read_data_size(&mut self) -> Result<DataSize> {
        let byte = self.read_byte()?;
        DataSize::try_from(byte).map_err(|_| Error::InvalidDataSize(byte).into())
    }

    fn push(&mut self, value: Immediate) -> Result<()> {
        let size = value.size();
        let size_bytes = size.size_in_bytes();
        let current_sp = self.regs.sp();

        if current_sp < size_bytes {
            return Err(Error::StackOverflow.into());
        }

        let new_sp = current_sp - size_bytes;
        self.regs.set_sp(new_sp);
        self.mem.write(new_sp, value, size)
    }

    fn pop(&mut self, size: DataSize) -> Result<Immediate> {
        let current_sp = self.regs.sp();
        if current_sp + size.size_in_bytes() > self.mem.storage.len() {
            return Err(Error::StackUnderflow.into());
        }

        let value = self.mem.read(current_sp, size)?;
        self.regs.set_sp(current_sp + size.size_in_bytes());
        Ok(value)
    }
}
