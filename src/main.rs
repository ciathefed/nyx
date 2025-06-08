use anyhow::Result;

use crate::{
    immediate::Immediate,
    instruction::Instruction,
    vm::{VM, register::Register},
};

mod immediate;
mod instruction;
mod vm;

fn main() -> Result<()> {
    let program = vec![
        Instruction::MovRegImm(Register::Q0, Immediate::QWord(1337)),
        Instruction::PushReg(Register::Q0),
        Instruction::PopReg(Register::D0),
        Instruction::Hlt,
    ];

    let mut vm = VM::new(program, 256);
    vm.run()?;

    println!("{:#?}", vm.regs);
    println!("Memory {:?}", vm.mem.storage);
    println!("Stack {:?}", vm.stack.storage);
    Ok(())
}
