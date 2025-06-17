use crate::{
    parser::ast::{DataSize, Immediate},
    vm::register::Register,
};

pub enum Instruction {
    Hlt,
    MovRegReg(Register, Register),
    MovRegImm(Register, Immediate),
    Ldr(DataSize, Register, usize),
    Str(DataSize, Register, usize),
    PushImm(Immediate),
    PushReg(Register),
    PushAddr(usize, DataSize),
    PopReg(Register),
    PopAddr(usize, DataSize),
}
