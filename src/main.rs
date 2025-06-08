use anyhow::Result;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VMError {
    #[error("instruction pointer out of bounds: {0}")]
    InstructionPointerOutOfBounds(usize),
    #[error("stack overflow")]
    StackOverflow,
    #[error("stack underflow")]
    StackUnderflow,
    #[error("unimplemented: {0}")]
    Unimplemented(&'static str),
}

#[derive(Clone, Copy, Debug)]
pub enum Register {
    B0,
    W0,
    D0,
    Q0,
    FF0,
    DD0,
    IP,
    SP,
    BP,
}

#[derive(Debug)]
pub struct Registers {
    b0: u8,
    w0: u16,
    d0: u32,
    q0: u64,
    ff0: f32,
    dd0: f64,
    ip: usize,
    sp: usize,
    bp: usize,
}

impl Registers {
    pub fn new() -> Self {
        Self {
            b0: 0,
            w0: 0,
            d0: 0,
            q0: 0,
            ff0: 0.0,
            dd0: 0.0,
            ip: 0,
            sp: 0,
            bp: 0,
        }
    }

    pub fn get(&self, reg: Register) -> Immediate {
        match reg {
            Register::B0 => Immediate::Byte(self.b0),
            Register::W0 => Immediate::Word(self.w0),
            Register::D0 => Immediate::DWord(self.d0),
            Register::Q0 => Immediate::QWord(self.q0),
            Register::FF0 => Immediate::Float(self.ff0),
            Register::DD0 => Immediate::Double(self.dd0),
            Register::IP => Immediate::QWord(self.ip as u64),
            Register::SP => Immediate::QWord(self.sp as u64),
            Register::BP => Immediate::QWord(self.bp as u64),
        }
    }

    pub fn set(&mut self, reg: Register, imm: Immediate) -> Result<()> {
        match reg {
            Register::B0 => self.b0 = imm.as_u8()?,
            Register::W0 => self.w0 = imm.as_u16()?,
            Register::D0 => self.d0 = imm.as_u32()?,
            Register::Q0 => self.q0 = imm.as_u64()?,
            Register::FF0 => self.ff0 = imm.as_f32()?,
            Register::DD0 => self.dd0 = imm.as_f64()?,
            Register::IP => self.ip = imm.as_usize()?,
            Register::SP => self.sp = imm.as_usize()?,
            Register::BP => self.bp = imm.as_usize()?,
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub enum DataSize {
    Byte,
    Word,
    DWord,
    QWord,
    Float,
    Double,
}

#[derive(Clone, Copy, Debug)]
pub enum Immediate {
    Byte(u8),
    Word(u16),
    DWord(u32),
    QWord(u64),
    Float(f32),
    Double(f64),
}

impl Immediate {
    pub fn as_u8(self) -> Result<u8> {
        match self {
            Immediate::Byte(v) => Ok(v),
            Immediate::Word(v) => Ok(v as u8),
            Immediate::DWord(v) => Ok(v as u8),
            Immediate::QWord(v) => Ok(v as u8),
            Immediate::Float(v) => Ok(v as u8),
            Immediate::Double(v) => Ok(v as u8),
        }
    }

    pub fn as_u16(self) -> Result<u16> {
        match self {
            Immediate::Byte(v) => Ok(v as u16),
            Immediate::Word(v) => Ok(v),
            Immediate::DWord(v) => Ok(v as u16),
            Immediate::QWord(v) => Ok(v as u16),
            Immediate::Float(v) => Ok(v as u16),
            Immediate::Double(v) => Ok(v as u16),
        }
    }

    pub fn as_u32(self) -> Result<u32> {
        match self {
            Immediate::Byte(v) => Ok(v as u32),
            Immediate::Word(v) => Ok(v as u32),
            Immediate::DWord(v) => Ok(v),
            Immediate::QWord(v) => Ok(v as u32),
            Immediate::Float(v) => Ok(v as u32),
            Immediate::Double(v) => Ok(v as u32),
        }
    }

    pub fn as_u64(self) -> Result<u64> {
        match self {
            Immediate::Byte(v) => Ok(v as u64),
            Immediate::Word(v) => Ok(v as u64),
            Immediate::DWord(v) => Ok(v as u64),
            Immediate::QWord(v) => Ok(v),
            Immediate::Float(v) => Ok(v as u64),
            Immediate::Double(v) => Ok(v as u64),
        }
    }

    pub fn as_f32(self) -> Result<f32> {
        match self {
            Immediate::Byte(v) => Ok(v as f32),
            Immediate::Word(v) => Ok(v as f32),
            Immediate::DWord(v) => Ok(v as f32),
            Immediate::QWord(v) => Ok(v as f32),
            Immediate::Float(v) => Ok(v),
            Immediate::Double(v) => Ok(v as f32),
        }
    }

    pub fn as_f64(self) -> Result<f64> {
        match self {
            Immediate::Byte(v) => Ok(v as f64),
            Immediate::Word(v) => Ok(v as f64),
            Immediate::DWord(v) => Ok(v as f64),
            Immediate::QWord(v) => Ok(v as f64),
            Immediate::Float(v) => Ok(v as f64),
            Immediate::Double(v) => Ok(v),
        }
    }

    pub fn as_usize(self) -> Result<usize> {
        match self {
            Immediate::Byte(v) => Ok(v as usize),
            Immediate::Word(v) => Ok(v as usize),
            Immediate::DWord(v) => Ok(v as usize),
            Immediate::QWord(v) => Ok(v as usize),
            Immediate::Float(v) => Ok(v as usize),
            Immediate::Double(v) => Ok(v as usize),
        }
    }
}

#[derive(Debug)]
pub struct Memory {
    storage: Vec<u8>,
}

impl Memory {
    pub fn new(size: usize) -> Self {
        Self {
            storage: vec![0; size],
        }
    }

    pub fn read(&self, addr: usize, size: DataSize) -> Result<Immediate> {
        use DataSize::*;
        match size {
            Byte => {
                if addr + 1 > self.storage.len() {
                    return Err(VMError::InstructionPointerOutOfBounds(addr).into());
                }
                Ok(Immediate::Byte(self.storage[addr]))
            }
            Word => {
                if addr + 2 > self.storage.len() {
                    return Err(VMError::InstructionPointerOutOfBounds(addr).into());
                }
                Ok(Immediate::Word(u16::from_le_bytes(
                    self.storage[addr..addr + 2].try_into().unwrap(),
                )))
            }
            DWord => {
                if addr + 4 > self.storage.len() {
                    return Err(VMError::InstructionPointerOutOfBounds(addr).into());
                }
                Ok(Immediate::DWord(u32::from_le_bytes(
                    self.storage[addr..addr + 4].try_into().unwrap(),
                )))
            }
            QWord => {
                if addr + 8 > self.storage.len() {
                    return Err(VMError::InstructionPointerOutOfBounds(addr).into());
                }
                Ok(Immediate::QWord(u64::from_le_bytes(
                    self.storage[addr..addr + 8].try_into().unwrap(),
                )))
            }
            Float => {
                if addr + 4 > self.storage.len() {
                    return Err(VMError::InstructionPointerOutOfBounds(addr).into());
                }
                Ok(Immediate::Float(f32::from_le_bytes(
                    self.storage[addr..addr + 4].try_into().unwrap(),
                )))
            }
            Double => {
                if addr + 8 > self.storage.len() {
                    return Err(VMError::InstructionPointerOutOfBounds(addr).into());
                }
                Ok(Immediate::Double(f64::from_le_bytes(
                    self.storage[addr..addr + 8].try_into().unwrap(),
                )))
            }
        }
    }

    pub fn write(&mut self, addr: usize, value: Immediate, size: DataSize) -> Result<()> {
        use DataSize::*;
        match size {
            Byte => {
                if addr + 1 > self.storage.len() {
                    return Err(VMError::InstructionPointerOutOfBounds(addr).into());
                }
                self.storage[addr] = value.as_u8()?;
            }
            Word => {
                if addr + 2 > self.storage.len() {
                    return Err(VMError::InstructionPointerOutOfBounds(addr).into());
                }
                self.storage[addr..addr + 2].copy_from_slice(&value.as_u16()?.to_le_bytes());
            }
            DWord => {
                if addr + 4 > self.storage.len() {
                    return Err(VMError::InstructionPointerOutOfBounds(addr).into());
                }
                self.storage[addr..addr + 4].copy_from_slice(&value.as_u32()?.to_le_bytes());
            }
            QWord => {
                if addr + 8 > self.storage.len() {
                    return Err(VMError::InstructionPointerOutOfBounds(addr).into());
                }
                self.storage[addr..addr + 8].copy_from_slice(&value.as_u64()?.to_le_bytes());
            }
            Float => {
                if addr + 4 > self.storage.len() {
                    return Err(VMError::InstructionPointerOutOfBounds(addr).into());
                }
                self.storage[addr..addr + 4].copy_from_slice(&value.as_f32()?.to_le_bytes());
            }
            Double => {
                if addr + 8 > self.storage.len() {
                    return Err(VMError::InstructionPointerOutOfBounds(addr).into());
                }
                self.storage[addr..addr + 8].copy_from_slice(&value.as_f64()?.to_le_bytes());
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Stack {
    storage: Vec<Immediate>,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            storage: Vec::new(),
        }
    }

    pub fn push(&mut self, value: Immediate) -> Result<()> {
        if self.storage.len() as isize >= isize::MAX {
            return Err(VMError::StackOverflow.into());
        }
        self.storage.push(value);
        Ok(())
    }

    pub fn pop(&mut self) -> Result<Immediate> {
        if let Some(imm) = self.storage.pop() {
            Ok(imm)
        } else {
            Err(VMError::StackUnderflow.into())
        }
    }
}

pub enum Instruction {
    Hlt,
    MovRegReg(Register, Register),
    MovRegImm(Register, Immediate),
    Ldr(Register, usize, DataSize),
    Str(usize, Register, DataSize),
    PushImm(Immediate),
    PushReg(Register),
    PushAddr(usize, DataSize),
    PopReg(Register),
    PopAddr(usize, DataSize),
}

pub struct VM {
    regs: Registers,
    mem: Memory,
    stack: Stack,
    program: Vec<Instruction>,
    halted: bool,
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
            .ok_or(VMError::InstructionPointerOutOfBounds(ip))?;

        match instr {
            Instruction::Hlt => self.halted = true,
            // Instruction::Hlt => return Err(VMError::Unimplemented("hlt").into()),
            Instruction::MovRegReg(dst, src) => {
                let value = self.regs.get(*src);
                self.regs.set(*dst, value)?;
            }
            Instruction::MovRegImm(dst, imm) => {
                self.regs.set(*dst, *imm)?;
            }
            Instruction::Ldr(dst, addr, size) => {
                let value = self.mem.read(*addr, *size)?;
                self.regs.set(*dst, value)?;
            }
            Instruction::Str(addr, src, size) => {
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

fn main() -> Result<()> {
    let program = vec![
        Instruction::MovRegImm(Register::Q0, Immediate::QWord(1337)),
        Instruction::PushReg(Register::Q0),
        Instruction::PopReg(Register::D0),
        Instruction::Hlt,
    ];

    let mut vm = VM::new(program, 256);
    vm.run()?;

    println!("{:#?}", vm.regs);
    println!("Memory {:?}", vm.mem.storage);
    println!("Stack {:?}", vm.stack.storage);
    Ok(())
}
