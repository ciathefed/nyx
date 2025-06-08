use anyhow::Result;

use crate::{
    immediate::{DataSize, Immediate},
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

    pub fn read(&self, addr: usize, size: DataSize) -> Result<Immediate> {
        use DataSize::*;
        match size {
            Byte => {
                if addr + 1 > self.storage.len() {
                    return Err(Error::InstructionPointerOutOfBounds(addr).into());
                }
                Ok(Immediate::Byte(self.storage[addr]))
            }
            Word => {
                if addr + 2 > self.storage.len() {
                    return Err(Error::InstructionPointerOutOfBounds(addr).into());
                }
                Ok(Immediate::Word(u16::from_le_bytes(
                    self.storage[addr..addr + 2].try_into().unwrap(),
                )))
            }
            DWord => {
                if addr + 4 > self.storage.len() {
                    return Err(Error::InstructionPointerOutOfBounds(addr).into());
                }
                Ok(Immediate::DWord(u32::from_le_bytes(
                    self.storage[addr..addr + 4].try_into().unwrap(),
                )))
            }
            QWord => {
                if addr + 8 > self.storage.len() {
                    return Err(Error::InstructionPointerOutOfBounds(addr).into());
                }
                Ok(Immediate::QWord(u64::from_le_bytes(
                    self.storage[addr..addr + 8].try_into().unwrap(),
                )))
            }
            Float => {
                if addr + 4 > self.storage.len() {
                    return Err(Error::InstructionPointerOutOfBounds(addr).into());
                }
                Ok(Immediate::Float(f32::from_le_bytes(
                    self.storage[addr..addr + 4].try_into().unwrap(),
                )))
            }
            Double => {
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
        use DataSize::*;
        match size {
            Byte => {
                if addr + 1 > self.storage.len() {
                    return Err(Error::InstructionPointerOutOfBounds(addr).into());
                }
                self.storage[addr] = value.as_u8()?;
            }
            Word => {
                if addr + 2 > self.storage.len() {
                    return Err(Error::InstructionPointerOutOfBounds(addr).into());
                }
                self.storage[addr..addr + 2].copy_from_slice(&value.as_u16()?.to_le_bytes());
            }
            DWord => {
                if addr + 4 > self.storage.len() {
                    return Err(Error::InstructionPointerOutOfBounds(addr).into());
                }
                self.storage[addr..addr + 4].copy_from_slice(&value.as_u32()?.to_le_bytes());
            }
            QWord => {
                if addr + 8 > self.storage.len() {
                    return Err(Error::InstructionPointerOutOfBounds(addr).into());
                }
                self.storage[addr..addr + 8].copy_from_slice(&value.as_u64()?.to_le_bytes());
            }
            Float => {
                if addr + 4 > self.storage.len() {
                    return Err(Error::InstructionPointerOutOfBounds(addr).into());
                }
                self.storage[addr..addr + 4].copy_from_slice(&value.as_f32()?.to_le_bytes());
            }
            Double => {
                if addr + 8 > self.storage.len() {
                    return Err(Error::InstructionPointerOutOfBounds(addr).into());
                }
                self.storage[addr..addr + 8].copy_from_slice(&value.as_f64()?.to_le_bytes());
            }
        }
        Ok(())
    }
}
