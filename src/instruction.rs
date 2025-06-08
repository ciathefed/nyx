use crate::{
    immediate::{DataSize, Immediate},
    vm::register::Register,
};

pub enum Instruction {
    Hlt,
    MovRegReg(Register, Register),
    MovRegImm(Register, Immediate),
    Ldr(Register, usize, DataSize),
    Str(usize, Register, DataSize),
    PushImm(Immediate),
    PushReg(Register),
    PushAddr(usize, DataSize),
    PopReg(Register),
    PopAddr(usize, DataSize),
}
