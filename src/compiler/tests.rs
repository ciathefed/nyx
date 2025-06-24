use crate::{lexer::Lexer, parser::Parser, vm::register::Register};

use super::*;

fn compile(input: &str) -> Result<Vec<u8>> {
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let mut compiler = Compiler::new(parser.parse()?);
    Ok(Vec::from(compiler.compile()?))
}

#[test]
fn label() {
    let tests = vec![(
        r#"_start:
            mov b0, exit
        exit:
            hlt"#,
        vec![
            Opcode::MovRegImm as u8,
            Register::B0 as u8,
            0x03,
            Opcode::Hlt as u8,
        ],
    )];

    for (input, expected) in tests {
        let bytecode = compile(input);
        assert_ne!(bytecode.is_err(), true);
        assert_eq!(expected, bytecode.unwrap());
    }
}

#[test]
fn nop() {
    let tests = vec![("nop", vec![Opcode::Nop as u8])];

    for (input, expected) in tests {
        let bytecode = compile(input);
        assert_ne!(bytecode.is_err(), true);
        assert_eq!(expected, bytecode.unwrap());
    }
}

#[test]
fn mov() {
    let tests = vec![
        (
            "mov w0, b0",
            vec![Opcode::MovRegReg as u8, Register::W0 as u8, 0x00],
        ),
        (
            "mov b0, 10",
            vec![Opcode::MovRegImm as u8, Register::B0 as u8, 0x0A],
        ),
        (
            "mov w0, 1337",
            vec![Opcode::MovRegImm as u8, Register::W0 as u8, 0x39, 0x05],
        ),
        (
            "mov d0, 70000",
            vec![
                Opcode::MovRegImm as u8,
                Register::D0 as u8,
                0x70,
                0x11,
                0x01,
                0x00,
            ],
        ),
        (
            "mov q0, 3735928559",
            vec![
                Opcode::MovRegImm as u8,
                Register::Q0 as u8,
                0xEF,
                0xBE,
                0xAD,
                0xDE,
                0x00,
                0x00,
                0x00,
                0x00,
            ],
        ),
        (
            "mov ff0, 420.69",
            vec![
                Opcode::MovRegImm as u8,
                Register::FF0 as u8,
                0x52,
                0x58,
                0xD2,
                0x43,
            ],
        ),
        (
            "mov dd0, 1337.420",
            vec![
                Opcode::MovRegImm as u8,
                Register::DD0 as u8,
                0x48,
                0xe1,
                0x7a,
                0x14,
                0xae,
                0xe5,
                0x94,
                0x40,
            ],
        ),
    ];

    for (input, expected) in tests {
        let bytecode = compile(input);
        assert_ne!(bytecode.is_err(), true);
        assert_eq!(expected, bytecode.unwrap());
    }
}

#[test]
fn str() {
    let tests = vec![
        (
            "str b0, [q0, 8]",
            vec![
                Opcode::Str as u8,
                Register::B0 as u8,
                ADDRESSING_VARIANT_1,
                Register::Q0 as u8,
                0x08,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
            ],
        ),
        (
            "str b0, [1000, 32]",
            vec![
                Opcode::Str as u8,
                Register::B0 as u8,
                ADDRESSING_VARIANT_2,
                0xE8,
                0x03,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x20,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
            ],
        ),
    ];

    for (input, expected) in tests {
        let bytecode = compile(input);
        assert_ne!(bytecode.is_err(), true);
        assert_eq!(expected, bytecode.unwrap());
    }
}

#[test]
fn ldr() {
    let tests = vec![
        (
            "ldr b0, [q0]",
            vec![
                Opcode::Ldr as u8,
                Register::B0 as u8,
                ADDRESSING_VARIANT_1,
                Register::Q0 as u8,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
            ],
        ),
        (
            "ldr b0, [1000]",
            vec![
                Opcode::Ldr as u8,
                Register::B0 as u8,
                ADDRESSING_VARIANT_1,
                0xE8,
                0x03,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
            ],
        ),
    ];

    for (input, expected) in tests {
        let bytecode = compile(input);
        assert_ne!(bytecode.is_err(), true);
        assert_eq!(expected, bytecode.unwrap());
    }
}

#[test]
fn push() {
    let tests = vec![
        ("push b0", vec![Opcode::PushReg as u8, Register::B0 as u8]),
        (
            "push DWORD 1337",
            vec![
                Opcode::PushImm as u8,
                DataSize::DWord as u8,
                0x39,
                0x05,
                0x00,
                0x00,
            ],
        ),
        (
            "push QWORD [q0, 8]",
            vec![
                Opcode::PushAddr as u8,
                DataSize::QWord as u8,
                ADDRESSING_VARIANT_1,
                Register::Q0 as u8,
                0x08,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
            ],
        ),
    ];

    for (input, expected) in tests {
        let bytecode = compile(input);
        assert_ne!(bytecode.is_err(), true);
        assert_eq!(expected, bytecode.unwrap());
    }
}

#[test]
fn pop() {
    let tests = vec![
        ("pop b0", vec![Opcode::PopReg as u8, Register::B0 as u8]),
        (
            "pop QWORD [q0, 8]",
            vec![
                Opcode::PopAddr as u8,
                DataSize::QWord as u8,
                ADDRESSING_VARIANT_1,
                Register::Q0 as u8,
                0x08,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
            ],
        ),
        (
            "pop BYTE [1000, 16]",
            vec![
                Opcode::PopAddr as u8,
                DataSize::Byte as u8,
                ADDRESSING_VARIANT_2,
                0xE8,
                0x03,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x10,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
            ],
        ),
    ];

    for (input, expected) in tests {
        let bytecode = compile(input);
        assert_ne!(bytecode.is_err(), true);
        assert_eq!(expected, bytecode.unwrap());
    }
}

#[test]
fn hlt() {
    let tests = vec![("hlt", vec![Opcode::Hlt as u8])];

    for (input, expected) in tests {
        let bytecode = compile(input);
        assert_ne!(bytecode.is_err(), true);
        assert_eq!(expected, bytecode.unwrap());
    }
}
