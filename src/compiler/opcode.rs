use std::fmt;

#[derive(Debug)]
#[repr(u8)]
pub enum Opcode {
    Nop,
    MovRegReg,
    MovRegImm,
    Ldr,
    Str,
    PushImm,
    PushReg,
    PushAddr,
    PopReg,
    PopAddr,
    Hlt,
}

impl Into<u8> for Opcode {
    fn into(self) -> u8 {
        self as u8
    }
}

impl TryFrom<u8> for Opcode {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::Nop),
            0x01 => Ok(Self::MovRegReg),
            0x02 => Ok(Self::MovRegImm),
            0x03 => Ok(Self::Ldr),
            0x04 => Ok(Self::Str),
            0x05 => Ok(Self::PushImm),
            0x06 => Ok(Self::PushReg),
            0x07 => Ok(Self::PushAddr),
            0x08 => Ok(Self::PopReg),
            0x09 => Ok(Self::PopAddr),
            0x0A => Ok(Self::Hlt),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Opcode::Nop => write!(f, "nop"),
            Opcode::MovRegReg => write!(f, "mov"),
            Opcode::MovRegImm => write!(f, "mov"),
            Opcode::Ldr => write!(f, "ldr"),
            Opcode::Str => write!(f, "str"),
            Opcode::PushImm => write!(f, "push"),
            Opcode::PushReg => write!(f, "push"),
            Opcode::PushAddr => write!(f, "push"),
            Opcode::PopReg => write!(f, "pop"),
            Opcode::PopAddr => write!(f, "pop"),
            Opcode::Hlt => write!(f, "hlt"),
        }
    }
}
