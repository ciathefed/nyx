use crate::{compiler::Compiler, lexer::Lexer, parser::Parser};

use super::*;

use miette::{NamedSource, Result};
use pretty_assertions::assert_eq;

const TEST_MEM_SIZE: usize = 1024;

fn run(input: &str, data_addr: usize, data_value: Option<Immediate>) -> Result<VM> {
    let named_source = NamedSource::new("vm_tests.nyx", input.to_string());
    let lexer = Lexer::new(named_source.clone());
    let mut parser = Parser::new(lexer);
    let mut compiler = Compiler::new(parser.parse()?, named_source);
    let program_bytes = Vec::from(compiler.compile()?);

    let mut vm = VM::new(program_bytes.clone(), TEST_MEM_SIZE);
    if let Some(data_value) = data_value {
        let size = data_value.size();
        vm.mem.write(data_addr, data_value, size)?;
    }
    vm.run()?;
    Ok(vm)
}

#[test]
fn hlt() -> Result<()> {
    let input = r#"hlt"#;
    let vm = run(input, 0, None)?;

    assert_eq!(vm.halted, true);
    Ok(())
}

#[test]
fn nop() -> Result<()> {
    let input = r#"
        nop
        hlt
    "#;
    let vm = run(input, 0, None)?;

    assert_eq!(vm.halted, true);
    assert_eq!(vm.regs.ip(), 2);
    Ok(())
}

#[test]
fn mov() -> Result<()> {
    let input = r#"
        mov q0, 1337
        mov d0, q0
        hlt
    "#;
    let vm = run(input, 0, None)?;

    assert_eq!(vm.halted, true);
    assert_eq!(vm.regs.ip(), 14);
    assert_eq!(vm.regs.get(Register::Q0), Immediate::QWord(1337));
    assert_eq!(vm.regs.get(Register::D0), Immediate::DWord(1337));
    Ok(())
}

#[test]
fn ldr() -> Result<()> {
    let data_addr = 512;
    let input = format!(
        r#"
            mov d0, {data_addr}
            ldr q0, [d0, 0]
            hlt
        "#
    );
    let vm = run(&input, data_addr, Some(Immediate::QWord(1337)))?;

    assert_eq!(vm.halted, true);
    assert_eq!(vm.regs.ip(), 19);
    assert_eq!(vm.regs.get(Register::Q0), Immediate::QWord(1337));
    Ok(())
}

#[test]
fn str() -> Result<()> {
    let data_addr = 512;
    let input = format!(
        r#"
            mov d1, {data_addr}
            mov q0, 7331
            str q0, [d1]
            hlt
        "#
    );
    let vm = run(&input, data_addr, None)?;

    assert_eq!(vm.halted, true);
    assert_eq!(vm.regs.ip(), 29);
    assert_eq!(
        vm.mem.read(data_addr, DataSize::QWord)?,
        Immediate::QWord(7331)
    );
    Ok(())
}

#[test]
fn push() -> Result<()> {
    let input = r#"
        mov q0, 1337
        push DWORD q0
        hlt
    "#;
    let vm = run(input, 0, None)?;

    assert!(vm.halted);
    assert_eq!(vm.regs.sp(), vm.mem.storage.len() - 4);
    let value = vm.mem.read(vm.regs.sp(), DataSize::DWord)?;
    assert_eq!(value, Immediate::DWord(1337));
    Ok(())
}

#[test]
fn pop() -> Result<()> {
    let input = r#"
        push QWORD 1337
        pop QWORD d0
        hlt
    "#;
    let vm = run(input, 0, None)?;

    assert!(vm.halted);
    assert_eq!(vm.regs.sp(), vm.mem.storage.len());
    assert_eq!(vm.regs.get(Register::D0), Immediate::DWord(1337));
    Ok(())
}
