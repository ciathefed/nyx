use miette::Result;

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

impl DataSize {
    pub fn size_in_bytes(&self) -> usize {
        match self {
            DataSize::Byte => 1,
            DataSize::Word => 2,
            DataSize::DWord => 4,
            DataSize::QWord => 8,
            DataSize::Float => 4,
            DataSize::Double => 8,
        }
    }
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
            Register::B1 => Self::Byte,
            Register::W1 => Self::Word,
            Register::D1 => Self::DWord,
            Register::Q1 => Self::QWord,
            Register::FF1 => Self::Float,
            Register::DD1 => Self::Double,
            Register::B2 => Self::Byte,
            Register::W2 => Self::Word,
            Register::D2 => Self::DWord,
            Register::Q2 => Self::QWord,
            Register::FF2 => Self::Float,
            Register::DD2 => Self::Double,
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

impl TryFrom<u8> for DataSize {
    type Error = ();

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::Byte),
            0x01 => Ok(Self::Word),
            0x02 => Ok(Self::DWord),
            0x03 => Ok(Self::QWord),
            0x04 => Ok(Self::Float),
            0x05 => Ok(Self::Double),
            _ => Err(()),
        }
    }
}

impl Into<u8> for DataSize {
    fn into(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
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

    pub fn size(&self) -> DataSize {
        match self {
            Immediate::Byte(_) => DataSize::Byte,
            Immediate::Word(_) => DataSize::Word,
            Immediate::DWord(_) => DataSize::DWord,
            Immediate::QWord(_) => DataSize::QWord,
            Immediate::Float(_) => DataSize::Float,
            Immediate::Double(_) => DataSize::Double,
        }
    }
}
