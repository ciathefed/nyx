macro_rules! binary_arithmetic_op {
    ($self:expr, $int_op:ident, $float_op:tt) => {{
        let dest = $self.read_register()?;
        let lhs = $self.read_register()?;
        let rhs = $self.read_register()?;
        let lhs_val = $self.regs.get(lhs);
        let rhs_val = $self.regs.get(rhs);
        let result = match DataSize::from(dest) {
            DataSize::Byte => {
                Immediate::Byte(lhs_val.as_u8()?.$int_op(rhs_val.as_u8()?))
            }
            DataSize::Word => {
                Immediate::Word(lhs_val.as_u16()?.$int_op(rhs_val.as_u16()?))
            }
            DataSize::DWord => {
                Immediate::DWord(lhs_val.as_u32()?.$int_op(rhs_val.as_u32()?))
            }
            DataSize::QWord => {
                Immediate::QWord(lhs_val.as_u64()?.$int_op(rhs_val.as_u64()?))
            }
            DataSize::Float => Immediate::Float(lhs_val.as_f32()? $float_op rhs_val.as_f32()?),
            DataSize::Double => Immediate::Double(lhs_val.as_f64()? $float_op rhs_val.as_f64()?),
        };
        $self.regs.set(dest, result)
    }};
}

macro_rules! binary_arithmetic_op_imm {
    ($self:expr, $int_op:ident, $float_op:tt) => {{
        let dest = $self.read_register()?;
        let lhs = $self.read_register()?;
        let lhs_val = $self.regs.get(lhs);
        let rhs_val = match DataSize::from(dest) {
            DataSize::Byte => Immediate::Byte($self.read_byte()?),
            DataSize::Word => Immediate::Word($self.read_word()?),
            DataSize::DWord => Immediate::DWord($self.read_dword()?),
            DataSize::QWord => Immediate::QWord($self.read_qword()?),
            DataSize::Float => Immediate::Float($self.read_float()?),
            DataSize::Double => Immediate::Double($self.read_double()?),
        };
        let result = match DataSize::from(dest) {
            DataSize::Byte => {
                Immediate::Byte(lhs_val.as_u8()?.$int_op(rhs_val.as_u8()?))
            }
            DataSize::Word => {
                Immediate::Word(lhs_val.as_u16()?.$int_op(rhs_val.as_u16()?))
            }
            DataSize::DWord => {
                Immediate::DWord(lhs_val.as_u32()?.$int_op(rhs_val.as_u32()?))
            }
            DataSize::QWord => {
                Immediate::QWord(lhs_val.as_u64()?.$int_op(rhs_val.as_u64()?))
            }
            DataSize::Float => Immediate::Float(lhs_val.as_f32()? $float_op rhs_val.as_f32()?),
            DataSize::Double => Immediate::Double(lhs_val.as_f64()? $float_op rhs_val.as_f64()?),
        };
        $self.regs.set(dest, result)
    }};
}

macro_rules! binary_bitwise_op {
    ($self:expr, $op:tt) => {{
        let dest = $self.read_register()?;
        let lhs = $self.read_register()?;
        let rhs = $self.read_register()?;
        let lhs_val = $self.regs.get(lhs);
        let rhs_val = $self.regs.get(rhs);
        let result = match DataSize::from(dest) {
            DataSize::Byte => Immediate::Byte(lhs_val.as_u8()? $op rhs_val.as_u8()?),
            DataSize::Word => Immediate::Word(lhs_val.as_u16()? $op rhs_val.as_u16()?),
            DataSize::DWord => Immediate::DWord(lhs_val.as_u32()? $op rhs_val.as_u32()?),
            DataSize::QWord => Immediate::QWord(lhs_val.as_u64()? $op rhs_val.as_u64()?),
            _ => return Err(Error::InvalidDataSize(DataSize::from(dest) as u8).into()),
        };
        $self.regs.set(dest, result)
    }};
}

macro_rules! binary_bitwise_op_imm {
    ($self:expr, $op:tt) => {{
        let dest = $self.read_register()?;
        let lhs = $self.read_register()?;
        let lhs_val = $self.regs.get(lhs);
        let rhs_val = match DataSize::from(dest) {
            DataSize::Byte => Immediate::Byte($self.read_byte()?),
            DataSize::Word => Immediate::Word($self.read_word()?),
            DataSize::DWord => Immediate::DWord($self.read_dword()?),
            DataSize::QWord => Immediate::QWord($self.read_qword()?),
            _ => return Err(Error::InvalidDataSize(DataSize::from(dest) as u8).into()),
        };
        let result = match DataSize::from(dest) {
            DataSize::Byte => Immediate::Byte(lhs_val.as_u8()? $op rhs_val.as_u8()?),
            DataSize::Word => Immediate::Word(lhs_val.as_u16()? $op rhs_val.as_u16()?),
            DataSize::DWord => Immediate::DWord(lhs_val.as_u32()? $op rhs_val.as_u32()?),
            DataSize::QWord => Immediate::QWord(lhs_val.as_u64()? $op rhs_val.as_u64()?),
            _ => return Err(Error::InvalidDataSize(DataSize::from(dest) as u8).into()),
        };
        $self.regs.set(dest, result)
    }};
}

macro_rules! shift_op {
    ($self:expr, $op:tt) => {{
        let dest = $self.read_register()?;
        let lhs = $self.read_register()?;
        let rhs = $self.read_register()?;
        let lhs_val = $self.regs.get(lhs);
        let rhs_val = $self.regs.get(rhs);
        let result = match DataSize::from(dest) {
            DataSize::Byte => Immediate::Byte(lhs_val.as_u8()? $op (rhs_val.as_u8()? & 7)),
            DataSize::Word => {
                Immediate::Word(lhs_val.as_u16()? $op (rhs_val.as_u16()? & 15))
            }
            DataSize::DWord => {
                Immediate::DWord(lhs_val.as_u32()? $op (rhs_val.as_u32()? & 31))
            }
            DataSize::QWord => {
                Immediate::QWord(lhs_val.as_u64()? $op (rhs_val.as_u64()? & 63))
            }
            _ => return Err(Error::InvalidDataSize(DataSize::from(dest) as u8).into()),
        };
        $self.regs.set(dest, result)
    }};
}

macro_rules! shift_op_imm {
    ($self:expr, $op:tt) => {{
        let dest = $self.read_register()?;
        let lhs = $self.read_register()?;
        let lhs_val = $self.regs.get(lhs);
        let rhs_val = match DataSize::from(dest) {
            DataSize::Byte => Immediate::Byte($self.read_byte()?),
            DataSize::Word => Immediate::Word($self.read_word()?),
            DataSize::DWord => Immediate::DWord($self.read_dword()?),
            DataSize::QWord => Immediate::QWord($self.read_qword()?),
            _ => return Err(Error::InvalidDataSize(DataSize::from(dest) as u8).into()),
        };
        let result = match DataSize::from(dest) {
            DataSize::Byte => Immediate::Byte(lhs_val.as_u8()? $op (rhs_val.as_u8()? & 7)),
            DataSize::Word => {
                Immediate::Word(lhs_val.as_u16()? $op (rhs_val.as_u16()? & 15))
            }
            DataSize::DWord => {
                Immediate::DWord(lhs_val.as_u32()? $op (rhs_val.as_u32()? & 31))
            }
            DataSize::QWord => {
                Immediate::QWord(lhs_val.as_u64()? $op (rhs_val.as_u64()? & 63))
            }
            _ => return Err(Error::InvalidDataSize(DataSize::from(dest) as u8).into()),
        };
        $self.regs.set(dest, result)
    }};
}
