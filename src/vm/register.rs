use anyhow::Result;

use crate::immediate::Immediate;

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
    pub(crate) b0: u8,
    pub(crate) w0: u16,
    pub(crate) d0: u32,
    pub(crate) q0: u64,
    pub(crate) ff0: f32,
    pub(crate) dd0: f64,
    pub(crate) ip: usize,
    pub(crate) sp: usize,
    pub(crate) bp: usize,
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
