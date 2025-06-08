use anyhow::Result;

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
