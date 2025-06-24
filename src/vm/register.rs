use miette::Result;

use crate::parser::ast::Immediate;

const GPR0: usize = 0;
const GPR1: usize = 1;

const FPR0: usize = 0;
const FPR1: usize = 1;
const IP_REG: usize = 0;
const SP_REG: usize = 1;
const BP_REG: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Register {
    B0,
    W0,
    D0,
    Q0,

    B1,
    W1,
    D1,
    Q1,

    FF0,
    DD0,

    IP,
    SP,
    BP,
}

impl Register {
    fn physical_info(self) -> (PhysicalRegisterType, usize, RegisterView) {
        match self {
            Register::B0 => (
                PhysicalRegisterType::GeneralPurpose,
                GPR0,
                RegisterView::Byte,
            ),
            Register::W0 => (
                PhysicalRegisterType::GeneralPurpose,
                GPR0,
                RegisterView::Word,
            ),
            Register::D0 => (
                PhysicalRegisterType::GeneralPurpose,
                GPR0,
                RegisterView::DWord,
            ),
            Register::Q0 => (
                PhysicalRegisterType::GeneralPurpose,
                GPR0,
                RegisterView::QWord,
            ),
            Register::B1 => (
                PhysicalRegisterType::GeneralPurpose,
                GPR1,
                RegisterView::Byte,
            ),
            Register::W1 => (
                PhysicalRegisterType::GeneralPurpose,
                GPR1,
                RegisterView::Word,
            ),
            Register::D1 => (
                PhysicalRegisterType::GeneralPurpose,
                GPR1,
                RegisterView::DWord,
            ),
            Register::Q1 => (
                PhysicalRegisterType::GeneralPurpose,
                GPR1,
                RegisterView::QWord,
            ),
            Register::FF0 => (
                PhysicalRegisterType::FloatingPoint,
                FPR0,
                RegisterView::Float,
            ),
            Register::DD0 => (
                PhysicalRegisterType::FloatingPoint,
                FPR0,
                RegisterView::Double,
            ),
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

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        match value {
            "b0" => Ok(Register::B0),
            "w0" => Ok(Register::W0),
            "d0" => Ok(Register::D0),
            "q0" => Ok(Register::Q0),
            "b1" => Ok(Register::B1),
            "w1" => Ok(Register::W1),
            "d1" => Ok(Register::D1),
            "q1" => Ok(Register::Q1),
            "ff0" => Ok(Register::FF0),
            "dd0" => Ok(Register::DD0),
            "ip" => Ok(Register::IP),
            "sp" => Ok(Register::SP),
            "bp" => Ok(Register::BP),
            _ => Err(()),
        }
    }
}

impl TryFrom<u8> for Register {
    type Error = ();

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Register::B0),
            0x01 => Ok(Register::W0),
            0x02 => Ok(Register::D0),
            0x03 => Ok(Register::Q0),
            0x04 => Ok(Register::B1),
            0x05 => Ok(Register::W1),
            0x06 => Ok(Register::D1),
            0x07 => Ok(Register::Q1),
            0x08 => Ok(Register::FF0),
            0x09 => Ok(Register::DD0),
            0x0A => Ok(Register::IP),
            0x0B => Ok(Register::SP),
            0x0C => Ok(Register::BP),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub struct Registers {
    gpr: [u64; 2],
    fpr: [u64; 1],
    special: [usize; 3],
}

impl Registers {
    pub fn new() -> Self {
        Self {
            gpr: [0; 2],
            fpr: [0; 1],
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
                    RegisterView::Float => {
                        // Interpret lower 32 bits as f32
                        let f32_bits = bits as u32;
                        Immediate::Float(f32::from_bits(f32_bits))
                    }
                    RegisterView::Double => {
                        // Interpret full 64 bits as f64
                        Immediate::Double(f64::from_bits(bits))
                    }
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
                    let f32_bits = new_value.to_bits() as u64;
                    self.fpr[index] = (self.fpr[index] & 0xFFFFFFFF00000000) | f32_bits;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overlapping_gpr_registers() {
        let mut regs = Registers::new();

        // Set Q0 to a known value
        regs.set(Register::Q0, Immediate::QWord(0x123456789ABCDEF0))
            .unwrap();

        // Check that smaller views show the correct bits
        assert_eq!(regs.get(Register::B0), Immediate::Byte(0xF0));
        assert_eq!(regs.get(Register::W0), Immediate::Word(0xDEF0));
        assert_eq!(regs.get(Register::D0), Immediate::DWord(0x9ABCDEF0));
        assert_eq!(regs.get(Register::Q0), Immediate::QWord(0x123456789ABCDEF0));
    }

    #[test]
    fn test_multiple_gpr_independence() {
        let mut regs = Registers::new();

        // Set Q0 and Q1 to different values
        regs.set(Register::Q0, Immediate::QWord(0x1111111111111111))
            .unwrap();
        regs.set(Register::Q1, Immediate::QWord(0x2222222222222222))
            .unwrap();

        // Check that they don't interfere with each other
        assert_eq!(regs.get(Register::Q0), Immediate::QWord(0x1111111111111111));
        assert_eq!(regs.get(Register::Q1), Immediate::QWord(0x2222222222222222));
        assert_eq!(regs.get(Register::D0), Immediate::DWord(0x11111111));
        assert_eq!(regs.get(Register::D1), Immediate::DWord(0x22222222));
    }

    #[test]
    fn test_byte_register_update() {
        let mut regs = Registers::new();

        // Set Q0 to a known value
        regs.set(Register::Q0, Immediate::QWord(0x123456789ABCDEF0))
            .unwrap();

        // Update B0 (lower 8 bits)
        regs.set(Register::B0, Immediate::Byte(0x42)).unwrap();

        // Check that only the lower 8 bits changed
        assert_eq!(regs.get(Register::B0), Immediate::Byte(0x42));
        assert_eq!(regs.get(Register::W0), Immediate::Word(0xDE42));
        assert_eq!(regs.get(Register::D0), Immediate::DWord(0x9ABCDE42));
        assert_eq!(regs.get(Register::Q0), Immediate::QWord(0x123456789ABCDE42));
    }

    #[test]
    fn test_word_register_update() {
        let mut regs = Registers::new();

        // Set Q0 to a known value
        regs.set(Register::Q0, Immediate::QWord(0x123456789ABCDEF0))
            .unwrap();

        // Update W0 (lower 16 bits)
        regs.set(Register::W0, Immediate::Word(0x1234)).unwrap();

        // Check that only the lower 16 bits changed
        assert_eq!(regs.get(Register::B0), Immediate::Byte(0x34));
        assert_eq!(regs.get(Register::W0), Immediate::Word(0x1234));
        assert_eq!(regs.get(Register::D0), Immediate::DWord(0x9ABC1234));
        assert_eq!(regs.get(Register::Q0), Immediate::QWord(0x123456789ABC1234));
    }

    #[test]
    fn test_dword_register_update_zeros_upper() {
        let mut regs = Registers::new();

        // Set Q0 to a known value
        regs.set(Register::Q0, Immediate::QWord(0x123456789ABCDEF0))
            .unwrap();

        // Update D0 (lower 32 bits) - should zero upper 32 bits in x86_64 style
        regs.set(Register::D0, Immediate::DWord(0x12345678))
            .unwrap();

        // Check that upper 32 bits are zeroed
        assert_eq!(regs.get(Register::B0), Immediate::Byte(0x78));
        assert_eq!(regs.get(Register::W0), Immediate::Word(0x5678));
        assert_eq!(regs.get(Register::D0), Immediate::DWord(0x12345678));
        assert_eq!(regs.get(Register::Q0), Immediate::QWord(0x12345678)); // Upper 32 bits zeroed
    }

    #[test]
    fn test_floating_point_registers() {
        let mut regs = Registers::new();

        // Set DD0 to a known double value
        regs.set(Register::DD0, Immediate::Double(123.456)).unwrap();

        // Check that we can read it back
        match regs.get(Register::DD0) {
            Immediate::Double(val) => assert!((val - 123.456).abs() < f64::EPSILON),
            _ => panic!("Expected Double"),
        }

        // Set FF0 to a float value (should preserve upper bits of the physical register)
        regs.set(Register::FF0, Immediate::Float(42.0)).unwrap();

        // Check that FF0 reads back correctly
        match regs.get(Register::FF0) {
            Immediate::Float(val) => assert!((val - 42.0).abs() < f32::EPSILON),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_register_independence_for_vm_test() {
        let mut regs = Registers::new();

        // This test demonstrates that we can now use D1 for addresses
        // while Q0 holds data, and they won't interfere
        regs.set(Register::D1, Immediate::DWord(512)).unwrap();
        regs.set(Register::Q0, Immediate::QWord(7331)).unwrap();

        // D1 should still be 512 even after setting Q0
        assert_eq!(regs.get(Register::D1), Immediate::DWord(512));
        assert_eq!(regs.get(Register::Q0), Immediate::QWord(7331));
    }
}
