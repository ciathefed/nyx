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
        mov d1, q0
        mov dd0, 4.20
        mov ff1, dd0
        hlt
    "#;
    let vm = run(input, 0, None)?;

    assert_eq!(vm.halted, true);
    assert_eq!(vm.regs.ip(), 27);
    assert_eq!(vm.regs.get(Register::Q0), Immediate::QWord(1337));
    assert_eq!(vm.regs.get(Register::D1), Immediate::DWord(1337));
    assert_eq!(vm.regs.get(Register::DD0), Immediate::Double(4.20));
    assert_eq!(vm.regs.get(Register::FF1), Immediate::Float(4.20));
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
        pop QWORD q0
        hlt
    "#;
    let vm = run(input, 0, None)?;

    assert!(vm.halted);
    assert_eq!(vm.regs.sp(), vm.mem.storage.len());
    assert_eq!(vm.regs.get(Register::Q0), Immediate::QWord(1337));
    Ok(())
}

#[test]
fn cmp() -> Result<()> {
    let input = r#"
        mov q0, 1337
        cmp q0, 1337
        hlt
    "#;
    let vm = run(input, 0, None)?;

    assert!(vm.halted);
    assert_eq!(vm.regs.ip(), 21);
    assert!(vm.flags.eq);
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
fn register_independence() {
    let mut regs = Registers::new();

    regs.set(Register::D1, Immediate::DWord(512)).unwrap();
    regs.set(Register::Q0, Immediate::QWord(7331)).unwrap();

    assert_eq!(regs.get(Register::D1), Immediate::DWord(512));
    assert_eq!(regs.get(Register::Q0), Immediate::QWord(7331));
}

#[test]
fn arithmetic_operations() -> Result<()> {
    let input = r#"
        mov q0, 10
        mov q1, 5
        add q2, q0, q1
        sub q3, q0, q1
        mul q4, q0, q1
        div q5, q0, q1
        hlt
    "#;
    let vm = run(input, 0, None)?;

    assert!(vm.halted);
    assert_eq!(vm.regs.get(Register::Q0), Immediate::QWord(10));
    assert_eq!(vm.regs.get(Register::Q1), Immediate::QWord(5));
    assert_eq!(vm.regs.get(Register::Q2), Immediate::QWord(15));
    assert_eq!(vm.regs.get(Register::Q3), Immediate::QWord(5));
    assert_eq!(vm.regs.get(Register::Q4), Immediate::QWord(50));
    assert_eq!(vm.regs.get(Register::Q5), Immediate::QWord(2));
    Ok(())
}

#[test]
fn arithmetic_immediate() -> Result<()> {
    let input = r#"
        mov q0, 20
        add q1, q0, 5
        sub q2, q0, 3
        mul q3, q0, 2
        div q4, q0, 4
        hlt
    "#;
    let vm = run(input, 0, None)?;

    assert!(vm.halted);
    assert_eq!(vm.regs.get(Register::Q0), Immediate::QWord(20));
    assert_eq!(vm.regs.get(Register::Q1), Immediate::QWord(25));
    assert_eq!(vm.regs.get(Register::Q2), Immediate::QWord(17));
    assert_eq!(vm.regs.get(Register::Q3), Immediate::QWord(40));
    assert_eq!(vm.regs.get(Register::Q4), Immediate::QWord(5));
    Ok(())
}

#[test]
fn bitwise_operations() -> Result<()> {
    let input = r#"
        mov q0, 15
        mov q1, 10
        and q2, q0, q1
        or q3, q0, q1
        xor q4, q0, q1
        hlt
    "#;
    let vm = run(input, 0, None)?;

    assert!(vm.halted);
    assert_eq!(vm.regs.get(Register::Q0), Immediate::QWord(15));
    assert_eq!(vm.regs.get(Register::Q1), Immediate::QWord(10));
    assert_eq!(vm.regs.get(Register::Q2), Immediate::QWord(10));
    assert_eq!(vm.regs.get(Register::Q3), Immediate::QWord(15));
    assert_eq!(vm.regs.get(Register::Q4), Immediate::QWord(5));
    Ok(())
}

#[test]
fn shift_operations() -> Result<()> {
    let input = r#"
        mov q0, 8
        mov q1, 2
        shl q2, q0, q1
        shr q3, q2, q1
        hlt
    "#;
    let vm = run(input, 0, None)?;

    assert!(vm.halted);
    assert_eq!(vm.regs.get(Register::Q0), Immediate::QWord(8));
    assert_eq!(vm.regs.get(Register::Q1), Immediate::QWord(2));
    assert_eq!(vm.regs.get(Register::Q2), Immediate::QWord(32));
    assert_eq!(vm.regs.get(Register::Q3), Immediate::QWord(8));
    Ok(())
}

#[test]
fn floating_point_arithmetic() -> Result<()> {
    let input = r#"
        mov ff0, 3.5
        mov ff1, 1.5
        add ff2, ff0, ff1
        sub ff3, ff0, ff1
        mul ff4, ff0, ff1
        div ff5, ff0, ff1
        hlt
    "#;
    let vm = run(input, 0, None)?;

    assert!(vm.halted);

    match vm.regs.get(Register::FF2) {
        Immediate::Float(val) => assert!((val - 5.0).abs() < f32::EPSILON),
        _ => panic!("Expected float"),
    }

    match vm.regs.get(Register::FF3) {
        Immediate::Float(val) => assert!((val - 2.0).abs() < f32::EPSILON),
        _ => panic!("Expected float"),
    }

    match vm.regs.get(Register::FF4) {
        Immediate::Float(val) => assert!((val - 5.25).abs() < f32::EPSILON),
        _ => panic!("Expected float"),
    }

    match vm.regs.get(Register::FF5) {
        Immediate::Float(val) => assert!((val - (7.0 / 3.0)).abs() < f32::EPSILON),
        _ => panic!("Expected float"),
    }

    Ok(())
}

#[test]
fn mixed_register_sizes() -> Result<()> {
    let input = r#"
        mov w0, 300
        mov b1, 50
        add w2, w0, b1
        hlt
    "#;
    let vm = run(input, 0, None)?;

    assert!(vm.halted);
    assert_eq!(vm.regs.get(Register::W0), Immediate::Word(300));
    assert_eq!(vm.regs.get(Register::B1), Immediate::Byte(50));
    assert_eq!(vm.regs.get(Register::W2), Immediate::Word(350));
    Ok(())
}

#[test]
fn overflow_wrapping() -> Result<()> {
    let input = r#"
        mov b0, 255
        add b1, b0, 1
        mov w0, 65535
        add w1, w0, 1
        hlt
    "#;
    let vm = run(input, 0, None)?;

    assert!(vm.halted);
    assert_eq!(vm.regs.get(Register::B0), Immediate::Byte(255));
    assert_eq!(vm.regs.get(Register::B1), Immediate::Byte(0));
    assert_eq!(vm.regs.get(Register::W0), Immediate::Word(65535));
    assert_eq!(vm.regs.get(Register::W1), Immediate::Word(0));
    Ok(())
}
