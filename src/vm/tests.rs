use crate::{compiler::Compiler, lexer::Lexer, parser::Parser};

use super::*;

use anyhow::Result;
use pretty_assertions::assert_eq;

const TEST_MEM_SIZE: usize = 1024;

fn run(input: &str) -> Result<VM> {
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let mut compiler = Compiler::new(parser.parse()?);
    let mut vm = VM::new(Vec::from(compiler.compile()?), TEST_MEM_SIZE);
    vm.mem
        .write(0x00, Immediate::QWord(1337), DataSize::QWord)?;
    vm.run()?;
    Ok(vm)
}

#[test]
fn hlt() -> Result<()> {
    let input = r#"hlt"#;
    let vm = run(input)?;

    assert_eq!(vm.halted, true);
    Ok(())
}

#[test]
fn nop() -> Result<()> {
    let input = r#"
        nop
        hlt
    "#;
    let vm = run(input)?;

    assert_eq!(vm.halted, true);
    assert_eq!(vm.regs.ip, 2);
    Ok(())
}

#[test]
fn mov() -> Result<()> {
    let input = r#"
        mov q0, 1337
        mov d0, q0
        hlt
    "#;
    let vm = run(input)?;

    assert_eq!(vm.halted, true);
    assert_eq!(vm.regs.ip, 14);
    assert_eq!(vm.regs.get(Register::Q0), Immediate::QWord(1337));
    assert_eq!(vm.regs.get(Register::D0), Immediate::DWord(1337));
    Ok(())
}

#[test]
fn ldr() -> Result<()> {
    let input = r#"
        ldr q0, [d0, 0]
        hlt
    "#;
    let vm = run(input)?;

    assert_eq!(vm.halted, true);
    assert_eq!(vm.regs.ip, 13);
    assert_eq!(vm.regs.get(Register::Q0), Immediate::QWord(1337));
    Ok(())
}

#[test]
fn str() -> Result<()> {
    let input = r#"
        mov q0, 7331
        str q0, [d0]
        hlt
    "#;
    let vm = run(input)?;

    assert_eq!(vm.halted, true);
    assert_eq!(vm.regs.ip, 23);
    assert_eq!(vm.mem.read(0x00, DataSize::QWord)?, Immediate::QWord(7331));
    Ok(())
}

#[test]
fn push() -> Result<()> {
    let input = r#"
        mov q0, 1337
        push q0
        hlt
    "#;
    let vm = run(input)?;

    assert!(vm.halted);
    assert_eq!(vm.regs.sp, vm.mem.storage.len() - 8);
    let value = vm.mem.read(vm.regs.sp as usize, DataSize::QWord)?;
    assert_eq!(value, Immediate::QWord(1337));
    Ok(())
}

#[test]
fn pop() -> Result<()> {
    let input = r#"
        push QWORD 1337
        pop q0
        hlt
    "#;
    let vm = run(input)?;

    assert!(vm.halted);
    assert_eq!(vm.regs.sp, vm.mem.storage.len());
    assert_eq!(vm.regs.get(Register::Q0), Immediate::QWord(1337));
    Ok(())
}
