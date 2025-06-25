use miette::Result;

use crate::{
    parser::ast::{DataSize, Immediate},
    vm::Error,
};

#[derive(Debug)]
pub struct Memory {
    pub(crate) storage: Vec<u8>,
}

impl Memory {
    pub fn new(size: usize) -> Self {
        Self {
            storage: vec![0; size],
        }
    }

    pub fn len(&self) -> usize {
        self.storage.len()
    }

    pub fn read(&self, addr: usize, size: DataSize) -> Result<Immediate> {
        match size {
            DataSize::Byte => {
                if addr + 1 > self.storage.len() {
                    return Err(Error::InstructionPointerOutOfBounds(addr).into());
                }
                Ok(Immediate::Byte(self.storage[addr]))
            }
            DataSize::Word => {
                if addr + 2 > self.storage.len() {
                    return Err(Error::InstructionPointerOutOfBounds(addr).into());
                }
                Ok(Immediate::Word(u16::from_le_bytes(
                    self.storage[addr..addr + 2].try_into().unwrap(),
                )))
            }
            DataSize::DWord => {
                if addr + 4 > self.storage.len() {
                    return Err(Error::InstructionPointerOutOfBounds(addr).into());
                }
                Ok(Immediate::DWord(u32::from_le_bytes(
                    self.storage[addr..addr + 4].try_into().unwrap(),
                )))
            }
            DataSize::QWord => {
                if addr + 8 > self.storage.len() {
                    return Err(Error::InstructionPointerOutOfBounds(addr).into());
                }
                Ok(Immediate::QWord(u64::from_le_bytes(
                    self.storage[addr..addr + 8].try_into().unwrap(),
                )))
            }
            DataSize::Float => {
                if addr + 4 > self.storage.len() {
                    return Err(Error::InstructionPointerOutOfBounds(addr).into());
                }
                Ok(Immediate::Float(f32::from_le_bytes(
                    self.storage[addr..addr + 4].try_into().unwrap(),
                )))
            }
            DataSize::Double => {
                if addr + 8 > self.storage.len() {
                    return Err(Error::InstructionPointerOutOfBounds(addr).into());
                }
                Ok(Immediate::Double(f64::from_le_bytes(
                    self.storage[addr..addr + 8].try_into().unwrap(),
                )))
            }
        }
    }

    pub fn write(&mut self, addr: usize, value: Immediate, size: DataSize) -> Result<()> {
        match size {
            DataSize::Byte => {
                if addr + 1 > self.storage.len() {
                    return Err(Error::InstructionPointerOutOfBounds(addr).into());
                }
                self.storage[addr] = value.as_u8()?;
            }
            DataSize::Word => {
                if addr + 2 > self.storage.len() {
                    return Err(Error::InstructionPointerOutOfBounds(addr).into());
                }
                self.storage[addr..addr + 2].copy_from_slice(&value.as_u16()?.to_le_bytes());
            }
            DataSize::DWord => {
                if addr + 4 > self.storage.len() {
                    return Err(Error::InstructionPointerOutOfBounds(addr).into());
                }
                self.storage[addr..addr + 4].copy_from_slice(&value.as_u32()?.to_le_bytes());
            }
            DataSize::QWord => {
                if addr + 8 > self.storage.len() {
                    return Err(Error::InstructionPointerOutOfBounds(addr).into());
                }
                self.storage[addr..addr + 8].copy_from_slice(&value.as_u64()?.to_le_bytes());
            }
            DataSize::Float => {
                if addr + 4 > self.storage.len() {
                    return Err(Error::InstructionPointerOutOfBounds(addr).into());
                }
                self.storage[addr..addr + 4].copy_from_slice(&value.as_f32()?.to_le_bytes());
            }
            DataSize::Double => {
                if addr + 8 > self.storage.len() {
                    return Err(Error::InstructionPointerOutOfBounds(addr).into());
                }
                self.storage[addr..addr + 8].copy_from_slice(&value.as_f64()?.to_le_bytes());
            }
        }
        Ok(())
    }
}
