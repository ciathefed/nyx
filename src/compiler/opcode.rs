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
    AddRegRegReg,
    AddRegRegImm,
    SubRegRegReg,
    SubRegRegImm,
    MulRegRegReg,
    MulRegRegImm,
    DivRegRegReg,
    DivRegRegImm,
    AndRegRegReg,
    AndRegRegImm,
    OrRegRegReg,
    OrRegRegImm,
    XorRegRegReg,
    XorRegRegImm,
    ShlRegRegReg,
    ShlRegRegImm,
    ShrRegRegReg,
    ShrRegRegImm,
    CmpRegImm,
    CmpRegReg,
    Syscall,
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
            0x0A => Ok(Self::AddRegRegReg),
            0x0B => Ok(Self::AddRegRegImm),
            0x0C => Ok(Self::SubRegRegReg),
            0x0D => Ok(Self::SubRegRegImm),
            0x0E => Ok(Self::MulRegRegReg),
            0x0F => Ok(Self::MulRegRegImm),
            0x10 => Ok(Self::DivRegRegReg),
            0x11 => Ok(Self::DivRegRegImm),
            0x12 => Ok(Self::AndRegRegReg),
            0x13 => Ok(Self::AndRegRegImm),
            0x14 => Ok(Self::OrRegRegReg),
            0x15 => Ok(Self::OrRegRegImm),
            0x16 => Ok(Self::XorRegRegReg),
            0x17 => Ok(Self::XorRegRegImm),
            0x18 => Ok(Self::ShlRegRegReg),
            0x19 => Ok(Self::ShlRegRegImm),
            0x1A => Ok(Self::ShrRegRegReg),
            0x1B => Ok(Self::ShrRegRegImm),
            0x1C => Ok(Self::CmpRegImm),
            0x1D => Ok(Self::CmpRegReg),
            0x1E => Ok(Self::Syscall),
            0x1F => Ok(Self::Hlt),
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
            Opcode::AddRegRegReg => write!(f, "add"),
            Opcode::AddRegRegImm => write!(f, "add"),
            Opcode::SubRegRegReg => write!(f, "sub"),
            Opcode::SubRegRegImm => write!(f, "sub"),
            Opcode::MulRegRegReg => write!(f, "mul"),
            Opcode::MulRegRegImm => write!(f, "mul"),
            Opcode::DivRegRegReg => write!(f, "div"),
            Opcode::DivRegRegImm => write!(f, "div"),
            Opcode::AndRegRegReg => write!(f, "and"),
            Opcode::AndRegRegImm => write!(f, "and"),
            Opcode::OrRegRegReg => write!(f, "or"),
            Opcode::OrRegRegImm => write!(f, "or"),
            Opcode::XorRegRegReg => write!(f, "xor"),
            Opcode::XorRegRegImm => write!(f, "xor"),
            Opcode::ShlRegRegReg => write!(f, "shl"),
            Opcode::ShlRegRegImm => write!(f, "shl"),
            Opcode::ShrRegRegReg => write!(f, "shr"),
            Opcode::ShrRegRegImm => write!(f, "shr"),
            Opcode::CmpRegImm => write!(f, "cmp"),
            Opcode::CmpRegReg => write!(f, "cmp"),
            Opcode::Syscall => write!(f, "syscall"),
            Opcode::Hlt => write!(f, "hlt"),
        }
    }
}
