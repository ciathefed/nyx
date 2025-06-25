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

#[test]
fn overlapping_gpr_registers() {
    let mut regs = Registers::new();

    regs.set(Register::Q0, Immediate::QWord(0x123456789ABCDEF0))
        .unwrap();

    assert_eq!(regs.get(Register::B0), Immediate::Byte(0xF0));
    assert_eq!(regs.get(Register::W0), Immediate::Word(0xDEF0));
    assert_eq!(regs.get(Register::D0), Immediate::DWord(0x9ABCDEF0));
    assert_eq!(regs.get(Register::Q0), Immediate::QWord(0x123456789ABCDEF0));
}

#[test]
fn multiple_gpr_independence() {
    let mut regs = Registers::new();

    regs.set(Register::Q0, Immediate::QWord(0x1111111111111111))
        .unwrap();
    regs.set(Register::Q1, Immediate::QWord(0x2222222222222222))
        .unwrap();

    assert_eq!(regs.get(Register::Q0), Immediate::QWord(0x1111111111111111));
    assert_eq!(regs.get(Register::Q1), Immediate::QWord(0x2222222222222222));
    assert_eq!(regs.get(Register::D0), Immediate::DWord(0x11111111));
    assert_eq!(regs.get(Register::D1), Immediate::DWord(0x22222222));
}

#[test]
fn byte_register_update() {
    let mut regs = Registers::new();

    regs.set(Register::Q0, Immediate::QWord(0x123456789ABCDEF0))
        .unwrap();

    regs.set(Register::B0, Immediate::Byte(0x42)).unwrap();

    assert_eq!(regs.get(Register::B0), Immediate::Byte(0x42));
    assert_eq!(regs.get(Register::W0), Immediate::Word(0xDE42));
    assert_eq!(regs.get(Register::D0), Immediate::DWord(0x9ABCDE42));
    assert_eq!(regs.get(Register::Q0), Immediate::QWord(0x123456789ABCDE42));
}

#[test]
fn word_register_update() {
    let mut regs = Registers::new();

    regs.set(Register::Q0, Immediate::QWord(0x123456789ABCDEF0))
        .unwrap();

    regs.set(Register::W0, Immediate::Word(0x1234)).unwrap();

    assert_eq!(regs.get(Register::B0), Immediate::Byte(0x34));
    assert_eq!(regs.get(Register::W0), Immediate::Word(0x1234));
    assert_eq!(regs.get(Register::D0), Immediate::DWord(0x9ABC1234));
    assert_eq!(regs.get(Register::Q0), Immediate::QWord(0x123456789ABC1234));
}

#[test]
fn dword_register_update_zeros_upper() {
    let mut regs = Registers::new();

    regs.set(Register::Q0, Immediate::QWord(0x123456789ABCDEF0))
        .unwrap();

    regs.set(Register::D0, Immediate::DWord(0x12345678))
        .unwrap();

    assert_eq!(regs.get(Register::B0), Immediate::Byte(0x78));
    assert_eq!(regs.get(Register::W0), Immediate::Word(0x5678));
    assert_eq!(regs.get(Register::D0), Immediate::DWord(0x12345678));
    assert_eq!(regs.get(Register::Q0), Immediate::QWord(0x12345678));
}

#[test]
fn floating_point_registers() {
    let mut regs = Registers::new();

    regs.set(Register::DD0, Immediate::Double(123.456)).unwrap();

    match regs.get(Register::DD0) {
        Immediate::Double(val) => assert!((val - 123.456).abs() < f64::EPSILON),
        _ => panic!("Expected Double"),
    }

    regs.set(Register::FF0, Immediate::Float(42.0)).unwrap();

    match regs.get(Register::FF0) {
        Immediate::Float(val) => assert!((val - 42.0).abs() < f32::EPSILON),
        _ => panic!("Expected Float"),
    }
}

#[test]
fn register_independence_for_vm_test() {
    let mut regs = Registers::new();

    regs.set(Register::D1, Immediate::DWord(512)).unwrap();
    regs.set(Register::Q0, Immediate::QWord(7331)).unwrap();

    assert_eq!(regs.get(Register::D1), Immediate::DWord(512));
    assert_eq!(regs.get(Register::Q0), Immediate::QWord(7331));
}
