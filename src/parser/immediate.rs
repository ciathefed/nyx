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
    #[rustfmt::skip]
    fn from(value: Register) -> Self {
        match value {
            Register::B0  | Register::B1  | Register::B2  | Register::B3  |
            Register::B4  | Register::B5  | Register::B6  | Register::B7  |
            Register::B8  | Register::B9  | Register::B10 | Register::B11 |
            Register::B12 | Register::B13 | Register::B14 | Register::B15
                => Self::Byte,

            Register::W0  | Register::W1  | Register::W2  | Register::W3  |
            Register::W4  | Register::W5  | Register::W6  | Register::W7  |
            Register::W8  | Register::W9  | Register::W10 | Register::W11 |
            Register::W12 | Register::W13 | Register::W14 | Register::W15
                => Self::Word,

            Register::D0  | Register::D1  | Register::D2  | Register::D3  |
            Register::D4  | Register::D5  | Register::D6  | Register::D7  |
            Register::D8  | Register::D9  | Register::D10 | Register::D11 |
            Register::D12 | Register::D13 | Register::D14 | Register::D15
                => Self::DWord,

            Register::Q0  | Register::Q1  | Register::Q2  | Register::Q3  |
            Register::Q4  | Register::Q5  | Register::Q6  | Register::Q7  |
            Register::Q8  | Register::Q9  | Register::Q10 | Register::Q11 |
            Register::Q12 | Register::Q13 | Register::Q14 | Register::Q15 |
            Register::IP  | Register::SP  | Register::BP
                => Self::QWord,

            Register::FF0  | Register::FF1  | Register::FF2  | Register::FF3  |
            Register::FF4  | Register::FF5  | Register::FF6  | Register::FF7  |
            Register::FF8  | Register::FF9  | Register::FF10 | Register::FF11 |
            Register::FF12 | Register::FF13 | Register::FF14 | Register::FF15
                => Self::Float,

            Register::DD0  | Register::DD1  | Register::DD2  | Register::DD3  |
            Register::DD4  | Register::DD5  | Register::DD6  | Register::DD7  |
            Register::DD8  | Register::DD9  | Register::DD10 | Register::DD11 |
            Register::DD12 | Register::DD13 | Register::DD14 | Register::DD15
                => Self::Double,
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
