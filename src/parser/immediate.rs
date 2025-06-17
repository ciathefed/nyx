use anyhow::Result;

use crate::vm::register::Register;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataSize {
    Byte,
    Word,
    DWord,
    QWord,
    Float,
    Double,
}

impl From<Register> for DataSize {
    fn from(value: Register) -> Self {
        match value {
            Register::B0 => Self::Byte,
            Register::W0 => Self::Word,
            Register::D0 => Self::DWord,
            Register::Q0 => Self::QWord,
            Register::FF0 => Self::Float,
            Register::DD0 => Self::Double,
            Register::IP => Self::QWord,
            Register::SP => Self::QWord,
            Register::BP => Self::QWord,
        }
    }
}

impl TryFrom<&str> for DataSize {
    type Error = ();

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "byte" => Ok(Self::Byte),
            "word" => Ok(Self::Word),
            "dword" => Ok(Self::DWord),
            "qword" => Ok(Self::QWord),
            "float" => Ok(Self::Float),
            "double" => Ok(Self::Double),
            _ => Err(()),
        }
    }
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
