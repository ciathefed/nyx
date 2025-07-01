use miette::Result;

use crate::parser::ast::Immediate;

const GPR0: usize = 0x00;
const GPR1: usize = 0x01;
const GPR2: usize = 0x02;
const GPR3: usize = 0x03;
const GPR4: usize = 0x04;
const GPR5: usize = 0x05;
const GPR6: usize = 0x06;
const GPR7: usize = 0x07;
const GPR8: usize = 0x08;
const GPR9: usize = 0x09;
const GPR10: usize = 0x0A;
const GPR11: usize = 0x0B;
const GPR12: usize = 0x0C;
const GPR13: usize = 0x0D;
const GPR14: usize = 0x0E;
const GPR15: usize = 0x0F;

const FPR0: usize = 0x00;
const FPR1: usize = 0x01;
const FPR2: usize = 0x02;
const FPR3: usize = 0x03;
const FPR4: usize = 0x04;
const FPR5: usize = 0x05;
const FPR6: usize = 0x06;
const FPR7: usize = 0x07;
const FPR8: usize = 0x08;
const FPR9: usize = 0x09;
const FPR10: usize = 0x0A;
const FPR11: usize = 0x0B;
const FPR12: usize = 0x0C;
const FPR13: usize = 0x0D;
const FPR14: usize = 0x0E;
const FPR15: usize = 0x0F;

const DPR0: usize = 0x10;
const DPR1: usize = 0x11;
const DPR2: usize = 0x12;
const DPR3: usize = 0x13;
const DPR4: usize = 0x14;
const DPR5: usize = 0x15;
const DPR6: usize = 0x16;
const DPR7: usize = 0x17;
const DPR8: usize = 0x18;
const DPR9: usize = 0x19;
const DPR10: usize = 0x1A;
const DPR11: usize = 0x1B;
const DPR12: usize = 0x1C;
const DPR13: usize = 0x1D;
const DPR14: usize = 0x1E;
const DPR15: usize = 0x1F;

const IP_REG: usize = 0;
const SP_REG: usize = 1;
const BP_REG: usize = 2;

#[rustfmt::skip]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Register {
    B0, W0, D0, Q0, FF0, DD0,
    B1, W1, D1, Q1, FF1, DD1,
    B2, W2, D2, Q2, FF2, DD2,
    B3, W3, D3, Q3, FF3, DD3,
    B4, W4, D4, Q4, FF4, DD4,
    B5, W5, D5, Q5, FF5, DD5,
    B6, W6, D6, Q6, FF6, DD6,
    B7, W7, D7, Q7, FF7, DD7,
    B8, W8, D8, Q8, FF8, DD8,
    B9, W9, D9, Q9, FF9, DD9,
    B10, W10, D10, Q10, FF10, DD10,
    B11, W11, D11, Q11, FF11, DD11,
    B12, W12, D12, Q12, FF12, DD12,
    B13, W13, D13, Q13, FF13, DD13,
    B14, W14, D14, Q14, FF14, DD14,
    B15, W15, D15, Q15, FF15, DD15,

    IP,
    SP,
    BP,
}

impl Register {
    #[rustfmt::skip]
    fn physical_info(self) -> (PhysicalRegisterType, usize, RegisterView) {
        match self {
            Register::B0 => (PhysicalRegisterType::GeneralPurpose, GPR0, RegisterView::Byte),
            Register::W0 => (PhysicalRegisterType::GeneralPurpose, GPR0, RegisterView::Word),
            Register::D0 => (PhysicalRegisterType::GeneralPurpose, GPR0, RegisterView::DWord),
            Register::Q0 => (PhysicalRegisterType::GeneralPurpose, GPR0, RegisterView::QWord),
            Register::FF0 => (PhysicalRegisterType::FloatingPoint, FPR0, RegisterView::Float),
            Register::DD0 => (PhysicalRegisterType::FloatingPoint, DPR0, RegisterView::Double),

            Register::B1 => (PhysicalRegisterType::GeneralPurpose, GPR1, RegisterView::Byte),
            Register::W1 => (PhysicalRegisterType::GeneralPurpose, GPR1, RegisterView::Word),
            Register::D1 => (PhysicalRegisterType::GeneralPurpose, GPR1, RegisterView::DWord),
            Register::Q1 => (PhysicalRegisterType::GeneralPurpose, GPR1, RegisterView::QWord),
            Register::FF1 => (PhysicalRegisterType::FloatingPoint, FPR1, RegisterView::Float),
            Register::DD1 => (PhysicalRegisterType::FloatingPoint, DPR1, RegisterView::Double),

            Register::B2 => (PhysicalRegisterType::GeneralPurpose, GPR2, RegisterView::Byte),
            Register::W2 => (PhysicalRegisterType::GeneralPurpose, GPR2, RegisterView::Word),
            Register::D2 => (PhysicalRegisterType::GeneralPurpose, GPR2, RegisterView::DWord),
            Register::Q2 => (PhysicalRegisterType::GeneralPurpose, GPR2, RegisterView::QWord),
            Register::FF2 => (PhysicalRegisterType::FloatingPoint, FPR2, RegisterView::Float),
            Register::DD2 => (PhysicalRegisterType::FloatingPoint, DPR2, RegisterView::Double),

            Register::B3 => (PhysicalRegisterType::GeneralPurpose, GPR3, RegisterView::Byte),
            Register::W3 => (PhysicalRegisterType::GeneralPurpose, GPR3, RegisterView::Word),
            Register::D3 => (PhysicalRegisterType::GeneralPurpose, GPR3, RegisterView::DWord),
            Register::Q3 => (PhysicalRegisterType::GeneralPurpose, GPR3, RegisterView::QWord),
            Register::FF3 => (PhysicalRegisterType::FloatingPoint, FPR3, RegisterView::Float),
            Register::DD3 => (PhysicalRegisterType::FloatingPoint, DPR3, RegisterView::Double),

            Register::B4 => (PhysicalRegisterType::GeneralPurpose, GPR4, RegisterView::Byte),
            Register::W4 => (PhysicalRegisterType::GeneralPurpose, GPR4, RegisterView::Word),
            Register::D4 => (PhysicalRegisterType::GeneralPurpose, GPR4, RegisterView::DWord),
            Register::Q4 => (PhysicalRegisterType::GeneralPurpose, GPR4, RegisterView::QWord),
            Register::FF4 => (PhysicalRegisterType::FloatingPoint, FPR4, RegisterView::Float),
            Register::DD4 => (PhysicalRegisterType::FloatingPoint, DPR4, RegisterView::Double),

            Register::B5 => (PhysicalRegisterType::GeneralPurpose, GPR5, RegisterView::Byte),
            Register::W5 => (PhysicalRegisterType::GeneralPurpose, GPR5, RegisterView::Word),
            Register::D5 => (PhysicalRegisterType::GeneralPurpose, GPR5, RegisterView::DWord),
            Register::Q5 => (PhysicalRegisterType::GeneralPurpose, GPR5, RegisterView::QWord),
            Register::FF5 => (PhysicalRegisterType::FloatingPoint, FPR5, RegisterView::Float),
            Register::DD5 => (PhysicalRegisterType::FloatingPoint, DPR5, RegisterView::Double),

            Register::B6 => (PhysicalRegisterType::GeneralPurpose, GPR6, RegisterView::Byte),
            Register::W6 => (PhysicalRegisterType::GeneralPurpose, GPR6, RegisterView::Word),
            Register::D6 => (PhysicalRegisterType::GeneralPurpose, GPR6, RegisterView::DWord),
            Register::Q6 => (PhysicalRegisterType::GeneralPurpose, GPR6, RegisterView::QWord),
            Register::FF6 => (PhysicalRegisterType::FloatingPoint, FPR6, RegisterView::Float),
            Register::DD6 => (PhysicalRegisterType::FloatingPoint, DPR6, RegisterView::Double),

            Register::B7 => (PhysicalRegisterType::GeneralPurpose, GPR7, RegisterView::Byte),
            Register::W7 => (PhysicalRegisterType::GeneralPurpose, GPR7, RegisterView::Word),
            Register::D7 => (PhysicalRegisterType::GeneralPurpose, GPR7, RegisterView::DWord),
            Register::Q7 => (PhysicalRegisterType::GeneralPurpose, GPR7, RegisterView::QWord),
            Register::FF7 => (PhysicalRegisterType::FloatingPoint, FPR7, RegisterView::Float),
            Register::DD7 => (PhysicalRegisterType::FloatingPoint, DPR7, RegisterView::Double),

            Register::B8 => (PhysicalRegisterType::GeneralPurpose, GPR8, RegisterView::Byte),
            Register::W8 => (PhysicalRegisterType::GeneralPurpose, GPR8, RegisterView::Word),
            Register::D8 => (PhysicalRegisterType::GeneralPurpose, GPR8, RegisterView::DWord),
            Register::Q8 => (PhysicalRegisterType::GeneralPurpose, GPR8, RegisterView::QWord),
            Register::FF8 => (PhysicalRegisterType::FloatingPoint, FPR8, RegisterView::Float),
            Register::DD8 => (PhysicalRegisterType::FloatingPoint, DPR8, RegisterView::Double),

            Register::B9 => (PhysicalRegisterType::GeneralPurpose, GPR9, RegisterView::Byte),
            Register::W9 => (PhysicalRegisterType::GeneralPurpose, GPR9, RegisterView::Word),
            Register::D9 => (PhysicalRegisterType::GeneralPurpose, GPR9, RegisterView::DWord),
            Register::Q9 => (PhysicalRegisterType::GeneralPurpose, GPR9, RegisterView::QWord),
            Register::FF9 => (PhysicalRegisterType::FloatingPoint, FPR9, RegisterView::Float),
            Register::DD9 => (PhysicalRegisterType::FloatingPoint, DPR9, RegisterView::Double),

            Register::B10 => (PhysicalRegisterType::GeneralPurpose, GPR10, RegisterView::Byte),
            Register::W10 => (PhysicalRegisterType::GeneralPurpose, GPR10, RegisterView::Word),
            Register::D10 => (PhysicalRegisterType::GeneralPurpose, GPR10, RegisterView::DWord),
            Register::Q10 => (PhysicalRegisterType::GeneralPurpose, GPR10, RegisterView::QWord),
            Register::FF10 => (PhysicalRegisterType::FloatingPoint, FPR10, RegisterView::Float),
            Register::DD10 => (PhysicalRegisterType::FloatingPoint, DPR10, RegisterView::Double),

            Register::B11 => (PhysicalRegisterType::GeneralPurpose, GPR11, RegisterView::Byte),
            Register::W11 => (PhysicalRegisterType::GeneralPurpose, GPR11, RegisterView::Word),
            Register::D11 => (PhysicalRegisterType::GeneralPurpose, GPR11, RegisterView::DWord),
            Register::Q11 => (PhysicalRegisterType::GeneralPurpose, GPR11, RegisterView::QWord),
            Register::FF11 => (PhysicalRegisterType::FloatingPoint, FPR11, RegisterView::Float),
            Register::DD11 => (PhysicalRegisterType::FloatingPoint, DPR11, RegisterView::Double),

            Register::B12 => (PhysicalRegisterType::GeneralPurpose, GPR12, RegisterView::Byte),
            Register::W12 => (PhysicalRegisterType::GeneralPurpose, GPR12, RegisterView::Word),
            Register::D12 => (PhysicalRegisterType::GeneralPurpose, GPR12, RegisterView::DWord),
            Register::Q12 => (PhysicalRegisterType::GeneralPurpose, GPR12, RegisterView::QWord),
            Register::FF12 => (PhysicalRegisterType::FloatingPoint, FPR12, RegisterView::Float),
            Register::DD12 => (PhysicalRegisterType::FloatingPoint, DPR12, RegisterView::Double),

            Register::B13 => (PhysicalRegisterType::GeneralPurpose, GPR13, RegisterView::Byte),
            Register::W13 => (PhysicalRegisterType::GeneralPurpose, GPR13, RegisterView::Word),
            Register::D13 => (PhysicalRegisterType::GeneralPurpose, GPR13, RegisterView::DWord),
            Register::Q13 => (PhysicalRegisterType::GeneralPurpose, GPR13, RegisterView::QWord),
            Register::FF13 => (PhysicalRegisterType::FloatingPoint, FPR13, RegisterView::Float),
            Register::DD13 => (PhysicalRegisterType::FloatingPoint, DPR13, RegisterView::Double),

            Register::B14 => (PhysicalRegisterType::GeneralPurpose, GPR14, RegisterView::Byte),
            Register::W14 => (PhysicalRegisterType::GeneralPurpose, GPR14, RegisterView::Word),
            Register::D14 => (PhysicalRegisterType::GeneralPurpose, GPR14, RegisterView::DWord),
            Register::Q14 => (PhysicalRegisterType::GeneralPurpose, GPR14, RegisterView::QWord),
            Register::FF14 => (PhysicalRegisterType::FloatingPoint, FPR14, RegisterView::Float),
            Register::DD14 => (PhysicalRegisterType::FloatingPoint, DPR14, RegisterView::Double),

            Register::B15 => (PhysicalRegisterType::GeneralPurpose, GPR15, RegisterView::Byte),
            Register::W15 => (PhysicalRegisterType::GeneralPurpose, GPR15, RegisterView::Word),
            Register::D15 => (PhysicalRegisterType::GeneralPurpose, GPR15, RegisterView::DWord),
            Register::Q15 => (PhysicalRegisterType::GeneralPurpose, GPR15, RegisterView::QWord),
            Register::FF15 => (PhysicalRegisterType::FloatingPoint, FPR15, RegisterView::Float),
            Register::DD15 => (PhysicalRegisterType::FloatingPoint, DPR15, RegisterView::Double),

            Register::IP => (PhysicalRegisterType::Special, IP_REG, RegisterView::QWord),
            Register::SP => (PhysicalRegisterType::Special, SP_REG, RegisterView::QWord),
            Register::BP => (PhysicalRegisterType::Special, BP_REG, RegisterView::QWord),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PhysicalRegisterType {
    GeneralPurpose,
    FloatingPoint,
    Special,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RegisterView {
    Byte,
    Word,
    DWord,
    QWord,
    Float,
    Double,
}

impl Into<u8> for Register {
    fn into(self) -> u8 {
        self as u8
    }
}

impl TryFrom<&str> for Register {
    type Error = ();

    #[rustfmt::skip]
    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        match value {
            "b0" => Ok(Register::B0),    "w0" => Ok(Register::W0),    "d0" => Ok(Register::D0),    "q0" => Ok(Register::Q0),    "ff0" => Ok(Register::FF0),    "dd0" => Ok(Register::DD0),
            "b1" => Ok(Register::B1),    "w1" => Ok(Register::W1),    "d1" => Ok(Register::D1),    "q1" => Ok(Register::Q1),    "ff1" => Ok(Register::FF1),    "dd1" => Ok(Register::DD1),
            "b2" => Ok(Register::B2),    "w2" => Ok(Register::W2),    "d2" => Ok(Register::D2),    "q2" => Ok(Register::Q2),    "ff2" => Ok(Register::FF2),    "dd2" => Ok(Register::DD2),
            "b3" => Ok(Register::B3),    "w3" => Ok(Register::W3),    "d3" => Ok(Register::D3),    "q3" => Ok(Register::Q3),    "ff3" => Ok(Register::FF3),    "dd3" => Ok(Register::DD3),
            "b4" => Ok(Register::B4),    "w4" => Ok(Register::W4),    "d4" => Ok(Register::D4),    "q4" => Ok(Register::Q4),    "ff4" => Ok(Register::FF4),    "dd4" => Ok(Register::DD4),
            "b5" => Ok(Register::B5),    "w5" => Ok(Register::W5),    "d5" => Ok(Register::D5),    "q5" => Ok(Register::Q5),    "ff5" => Ok(Register::FF5),    "dd5" => Ok(Register::DD5),
            "b6" => Ok(Register::B6),    "w6" => Ok(Register::W6),    "d6" => Ok(Register::D6),    "q6" => Ok(Register::Q6),    "ff6" => Ok(Register::FF6),    "dd6" => Ok(Register::DD6),
            "b7" => Ok(Register::B7),    "w7" => Ok(Register::W7),    "d7" => Ok(Register::D7),    "q7" => Ok(Register::Q7),    "ff7" => Ok(Register::FF7),    "dd7" => Ok(Register::DD7),
            "b8" => Ok(Register::B8),    "w8" => Ok(Register::W8),    "d8" => Ok(Register::D8),    "q8" => Ok(Register::Q8),    "ff8" => Ok(Register::FF8),    "dd8" => Ok(Register::DD8),
            "b9" => Ok(Register::B9),    "w9" => Ok(Register::W9),    "d9" => Ok(Register::D9),    "q9" => Ok(Register::Q9),    "ff9" => Ok(Register::FF9),    "dd9" => Ok(Register::DD9),
            "b10" => Ok(Register::B10),  "w10" => Ok(Register::W10),  "d10" => Ok(Register::D10),  "q10" => Ok(Register::Q10),  "ff10" => Ok(Register::FF10),  "dd10" => Ok(Register::DD10),
            "b11" => Ok(Register::B11),  "w11" => Ok(Register::W11),  "d11" => Ok(Register::D11),  "q11" => Ok(Register::Q11),  "ff11" => Ok(Register::FF11),  "dd11" => Ok(Register::DD11),
            "b12" => Ok(Register::B12),  "w12" => Ok(Register::W12),  "d12" => Ok(Register::D12),  "q12" => Ok(Register::Q12),  "ff12" => Ok(Register::FF12),  "dd12" => Ok(Register::DD12),
            "b13" => Ok(Register::B13),  "w13" => Ok(Register::W13),  "d13" => Ok(Register::D13),  "q13" => Ok(Register::Q13),  "ff13" => Ok(Register::FF13),  "dd13" => Ok(Register::DD13),
            "b14" => Ok(Register::B14),  "w14" => Ok(Register::W14),  "d14" => Ok(Register::D14),  "q14" => Ok(Register::Q14),  "ff14" => Ok(Register::FF14),  "dd14" => Ok(Register::DD14),
            "b15" => Ok(Register::B15),  "w15" => Ok(Register::W15),  "d15" => Ok(Register::D15),  "q15" => Ok(Register::Q15),  "ff15" => Ok(Register::FF15),  "dd15" => Ok(Register::DD15),

            "ip" => Ok(Register::IP),
            "sp" => Ok(Register::SP),
            "bp" => Ok(Register::BP),
            _ => Err(()),
        }
    }
}

impl TryFrom<u8> for Register {
    type Error = ();

    #[rustfmt::skip]
    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Register::B0),   0x01 => Ok(Register::W0),   0x02 => Ok(Register::D0),   0x03 => Ok(Register::Q0),   0x04 => Ok(Register::FF0),   0x05 => Ok(Register::DD0),
            0x06 => Ok(Register::B1),   0x07 => Ok(Register::W1),   0x08 => Ok(Register::D1),   0x09 => Ok(Register::Q1),   0x0A => Ok(Register::FF1),   0x0B => Ok(Register::DD1),
            0x0C => Ok(Register::B2),   0x0D => Ok(Register::W2),   0x0E => Ok(Register::D2),   0x0F => Ok(Register::Q2),   0x10 => Ok(Register::FF2),   0x11 => Ok(Register::DD2),
            0x12 => Ok(Register::B3),   0x13 => Ok(Register::W3),   0x14 => Ok(Register::D3),   0x15 => Ok(Register::Q3),   0x16 => Ok(Register::FF3),   0x17 => Ok(Register::DD3),
            0x18 => Ok(Register::B4),   0x19 => Ok(Register::W4),   0x1A => Ok(Register::D4),   0x1B => Ok(Register::Q4),   0x1C => Ok(Register::FF4),   0x1D => Ok(Register::DD4),
            0x1E => Ok(Register::B5),   0x1F => Ok(Register::W5),   0x20 => Ok(Register::D5),   0x21 => Ok(Register::Q5),   0x22 => Ok(Register::FF5),   0x23 => Ok(Register::DD5),
            0x24 => Ok(Register::B6),   0x25 => Ok(Register::W6),   0x26 => Ok(Register::D6),   0x27 => Ok(Register::Q6),   0x28 => Ok(Register::FF6),   0x29 => Ok(Register::DD6),
            0x2A => Ok(Register::B7),   0x2B => Ok(Register::W7),   0x2C => Ok(Register::D7),   0x2D => Ok(Register::Q7),   0x2E => Ok(Register::FF7),   0x2F => Ok(Register::DD7),
            0x30 => Ok(Register::B8),   0x31 => Ok(Register::W8),   0x32 => Ok(Register::D8),   0x33 => Ok(Register::Q8),   0x34 => Ok(Register::FF8),   0x35 => Ok(Register::DD8),
            0x36 => Ok(Register::B9),   0x37 => Ok(Register::W9),   0x38 => Ok(Register::D9),   0x39 => Ok(Register::Q9),   0x3A => Ok(Register::FF9),   0x3B => Ok(Register::DD9),
            0x3C => Ok(Register::B10),  0x3D => Ok(Register::W10),  0x3E => Ok(Register::D10),  0x3F => Ok(Register::Q10),  0x40 => Ok(Register::FF10),  0x41 => Ok(Register::DD10),
            0x42 => Ok(Register::B11),  0x43 => Ok(Register::W11),  0x44 => Ok(Register::D11),  0x45 => Ok(Register::Q11),  0x46 => Ok(Register::FF11),  0x47 => Ok(Register::DD11),
            0x48 => Ok(Register::B12),  0x49 => Ok(Register::W12),  0x4A => Ok(Register::D12),  0x4B => Ok(Register::Q12),  0x4C => Ok(Register::FF12),  0x4D => Ok(Register::DD12),
            0x4E => Ok(Register::B13),  0x4F => Ok(Register::W13),  0x50 => Ok(Register::D13),  0x51 => Ok(Register::Q13),  0x52 => Ok(Register::FF13),  0x53 => Ok(Register::DD13),
            0x54 => Ok(Register::B14),  0x55 => Ok(Register::W14),  0x56 => Ok(Register::D14),  0x57 => Ok(Register::Q14),  0x58 => Ok(Register::FF14),  0x59 => Ok(Register::DD14),
            0x5A => Ok(Register::B15),  0x5B => Ok(Register::W15),  0x5C => Ok(Register::D15),  0x5D => Ok(Register::Q15),  0x5E => Ok(Register::FF15),  0x5F => Ok(Register::DD15),

            0x60 => Ok(Register::IP),
            0x61 => Ok(Register::SP),
            0x62 => Ok(Register::BP),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub struct Registers {
    gpr: [u64; 16],
    fpr: [u64; 32],
    special: [usize; 3],
}

#[allow(dead_code)]
impl Registers {
    pub fn new() -> Self {
        Self {
            gpr: [0; 16],
            fpr: [0; 32],
            special: [0; 3],
        }
    }

    pub fn get(&self, reg: Register) -> Immediate {
        let (reg_type, index, view) = reg.physical_info();

        match reg_type {
            PhysicalRegisterType::GeneralPurpose => {
                let value = self.gpr[index];
                match view {
                    RegisterView::Byte => Immediate::Byte(value as u8),
                    RegisterView::Word => Immediate::Word(value as u16),
                    RegisterView::DWord => Immediate::DWord(value as u32),
                    RegisterView::QWord => Immediate::QWord(value),
                    _ => unreachable!("Invalid view for general-purpose register"),
                }
            }
            PhysicalRegisterType::FloatingPoint => {
                let bits = self.fpr[index];
                match view {
                    RegisterView::Float => Immediate::Float(f32::from_bits(bits as u32)),
                    RegisterView::Double => Immediate::Double(f64::from_bits(bits)),
                    _ => unreachable!("Invalid view for floating-point register"),
                }
            }
            PhysicalRegisterType::Special => {
                let value = self.special[index] as u64;
                match view {
                    RegisterView::QWord => Immediate::QWord(value),
                    _ => unreachable!("Invalid view for special register"),
                }
            }
        }
    }

    pub fn set(&mut self, reg: Register, imm: Immediate) -> Result<()> {
        let (reg_type, index, view) = reg.physical_info();

        match reg_type {
            PhysicalRegisterType::GeneralPurpose => match view {
                RegisterView::Byte => {
                    let new_value = imm.as_u8()?;
                    self.gpr[index] = (self.gpr[index] & 0xFFFFFFFFFFFFFF00) | (new_value as u64);
                }
                RegisterView::Word => {
                    let new_value = imm.as_u16()?;
                    self.gpr[index] = (self.gpr[index] & 0xFFFFFFFFFFFF0000) | (new_value as u64);
                }
                RegisterView::DWord => {
                    let new_value = imm.as_u32()?;
                    self.gpr[index] = new_value as u64;
                }
                RegisterView::QWord => {
                    self.gpr[index] = imm.as_u64()?;
                }
                _ => unreachable!("Invalid view for general-purpose register"),
            },
            PhysicalRegisterType::FloatingPoint => match view {
                RegisterView::Float => {
                    let new_value = imm.as_f32()?;
                    self.fpr[index] = new_value.to_bits() as u64;
                }
                RegisterView::Double => {
                    let new_value = imm.as_f64()?;
                    self.fpr[index] = new_value.to_bits();
                }
                _ => unreachable!("Invalid view for floating-point register"),
            },
            PhysicalRegisterType::Special => match view {
                RegisterView::QWord => {
                    self.special[index] = imm.as_usize()?;
                }
                _ => unreachable!("Invalid view for special register"),
            },
        }
        Ok(())
    }

    pub fn ip(&self) -> usize {
        self.special[IP_REG]
    }

    pub fn set_ip(&mut self, value: usize) {
        self.special[IP_REG] = value;
    }

    pub fn sp(&self) -> usize {
        self.special[SP_REG]
    }

    pub fn set_sp(&mut self, value: usize) {
        self.special[SP_REG] = value;
    }

    pub fn bp(&self) -> usize {
        self.special[BP_REG]
    }

    pub fn set_bp(&mut self, value: usize) {
        self.special[BP_REG] = value;
    }
}
