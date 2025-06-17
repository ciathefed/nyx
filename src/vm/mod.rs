use anyhow::Result;

use crate::{
    parser::ast::Instruction,
    vm::{memory::Memory, register::Registers, stack::Stack},
};

pub mod memory;
pub mod register;
pub mod stack;

#[derive(thiserror::Error, Debug)]
pub enum Error {
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
    pub(crate) program: Vec<Instruction>,
    pub(crate) halted: bool,
}

impl VM {
    pub fn new(program: Vec<Instruction>, mem_size: usize) -> Self {
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

        let ip = self.regs.ip;
        let instr = self
            .program
            .get(ip)
            .ok_or(Error::InstructionPointerOutOfBounds(ip))?;

        match instr {
            Instruction::Hlt => self.halted = true,
            Instruction::MovRegReg(dst, src) => {
                let value = self.regs.get(*src);
                self.regs.set(*dst, value)?;
            }
            Instruction::MovRegImm(dst, imm) => {
                self.regs.set(*dst, *imm)?;
            }
            Instruction::Ldr(size, dst, addr) => {
                let value = self.mem.read(*addr, *size)?;
                self.regs.set(*dst, value)?;
            }
            Instruction::Str(size, src, addr) => {
                let value = self.regs.get(*src);
                self.mem.write(*addr, value, *size)?;
            }
            Instruction::PushImm(imm) => {
                self.stack.push(*imm)?;
                self.regs.sp += 1;
            }
            Instruction::PushReg(reg) => {
                let val = self.regs.get(*reg);
                self.stack.push(val)?;
                self.regs.sp += 1;
            }
            Instruction::PushAddr(addr, size) => {
                let val = self.mem.read(*addr, *size)?;
                self.stack.push(val)?;
                self.regs.sp += 1;
            }
            Instruction::PopReg(dst) => {
                let val = self.stack.pop()?;
                self.regs.set(*dst, val)?;
                self.regs.sp = self.regs.sp.saturating_sub(1);
            }
            Instruction::PopAddr(addr, size) => {
                let val = self.stack.pop()?;
                self.mem.write(*addr, val, *size)?;
                self.regs.sp = self.regs.sp.saturating_sub(1);
            }
        }

        self.regs.ip += 1;
        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        while !self.halted {
            self.step()?;
        }
        Ok(())
    }
}
