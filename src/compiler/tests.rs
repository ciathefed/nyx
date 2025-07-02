use crate::{lexer::Lexer, parser::Parser, vm::register::Register};

use miette::{NamedSource, Result};

use super::*;

fn compile(input: &str) -> Result<Vec<u8>> {
    let named_source = NamedSource::new("compiler_tests.nyx", input.to_string());

    let lexer = Lexer::new(named_source.clone());
    let mut parser = Parser::new(lexer);
    let mut compiler = Compiler::new(parser.parse()?, named_source);
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
                ADDRESSING_VARIANT_2,
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
        (
            "push b0",
            vec![
                Opcode::PushReg as u8,
                DataSize::Byte as u8,
                Register::B0 as u8,
            ],
        ),
        (
            "push WORD b0",
            vec![
                Opcode::PushReg as u8,
                DataSize::Word as u8,
                Register::B0 as u8,
            ],
        ),
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
        (
            "pop b0",
            vec![
                Opcode::PopReg as u8,
                DataSize::Byte as u8,
                Register::B0 as u8,
            ],
        ),
        (
            "pop WORD b0",
            vec![
                Opcode::PopReg as u8,
                DataSize::Word as u8,
                Register::B0 as u8,
            ],
        ),
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
fn call() {
    let tests = vec![
        (
            "call function_name function_name: hlt",
            vec![
                Opcode::CallImm as u8,
                0x09,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                Opcode::Hlt as u8,
            ],
        ),
        ("call q0", vec![Opcode::CallReg as u8, Register::Q0 as u8]),
    ];

    for (input, expected) in tests {
        let bytecode = compile(input);
        assert_ne!(bytecode.is_err(), true);
        assert_eq!(expected, bytecode.unwrap());
    }
}

#[test]
fn ret() {
    let tests = vec![("ret", vec![Opcode::Ret as u8])];

    for (input, expected) in tests {
        let bytecode = compile(input);
        assert_ne!(bytecode.is_err(), true);
        assert_eq!(expected, bytecode.unwrap());
    }
}

#[test]
fn inc() {
    let tests = vec![("inc q0", vec![Opcode::Inc as u8, Register::Q0 as u8])];

    for (input, expected) in tests {
        let bytecode = compile(input);
        assert_ne!(bytecode.is_err(), true);
        assert_eq!(expected, bytecode.unwrap());
    }
}

#[test]
fn dec() {
    let tests = vec![("dec q0", vec![Opcode::Dec as u8, Register::Q0 as u8])];

    for (input, expected) in tests {
        let bytecode = compile(input);
        assert_ne!(bytecode.is_err(), true);
        assert_eq!(expected, bytecode.unwrap());
    }
}

#[test]
fn db() {
    let tests = vec![
        ("db 69", vec![0x45]),
        (
            r#"db "Hello, World", 10, 0"#,
            vec![
                0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x2C, 0x20, 0x57, 0x6F, 0x72, 0x6C, 0x64, 0x0A, 0x00,
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

#[test]
fn arithmetic_operations() {
    let tests = vec![
        (
            "add q0, q1, q2",
            vec![
                Opcode::AddRegRegReg as u8,
                Register::Q0 as u8,
                Register::Q1 as u8,
                Register::Q2 as u8,
            ],
        ),
        (
            "sub d0, d1, 42",
            vec![
                Opcode::SubRegRegImm as u8,
                Register::D0 as u8,
                Register::D1 as u8,
                42,
                0,
                0,
                0,
            ],
        ),
        (
            "mul w0, w1, w2",
            vec![
                Opcode::MulRegRegReg as u8,
                Register::W0 as u8,
                Register::W1 as u8,
                Register::W2 as u8,
            ],
        ),
        (
            "div b0, b1, 10",
            vec![
                Opcode::DivRegRegImm as u8,
                Register::B0 as u8,
                Register::B1 as u8,
                10,
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
fn bitwise_operations() {
    let tests = vec![
        (
            "and q0, q1, q2",
            vec![
                Opcode::AndRegRegReg as u8,
                Register::Q0 as u8,
                Register::Q1 as u8,
                Register::Q2 as u8,
            ],
        ),
        (
            "or d0, d1, 255",
            vec![
                Opcode::OrRegRegImm as u8,
                Register::D0 as u8,
                Register::D1 as u8,
                255,
                0,
                0,
                0,
            ],
        ),
        (
            "xor w0, w1, w2",
            vec![
                Opcode::XorRegRegReg as u8,
                Register::W0 as u8,
                Register::W1 as u8,
                Register::W2 as u8,
            ],
        ),
        (
            "shl b0, b1, 4",
            vec![
                Opcode::ShlRegRegImm as u8,
                Register::B0 as u8,
                Register::B1 as u8,
                4,
            ],
        ),
        (
            "shr q0, q1, q2",
            vec![
                Opcode::ShrRegRegReg as u8,
                Register::Q0 as u8,
                Register::Q1 as u8,
                Register::Q2 as u8,
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
fn jump_operations() {
    let tests = vec![
        (
            "_start: jmp _start",
            vec![
                Opcode::JmpImm as u8,
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
            "jne 0x37",
            vec![
                Opcode::JneImm as u8,
                0x37,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
            ],
        ),
        ("jge q0", vec![Opcode::JgeReg as u8, Register::Q0 as u8]),
    ];

    for (input, expected) in tests {
        let bytecode = compile(input);
        assert_ne!(bytecode.is_err(), true);
        assert_eq!(expected, bytecode.unwrap());
    }
}

#[test]
fn sections() {
    let input = r#"
.section text
start:
    mov q0, 42
    nop

.section data
value:
    db 0x12, 0x34

.section text
end:
    hlt
"#;

    let source = NamedSource::new("test.asm", input.to_string());
    let lexer = Lexer::new(source.clone());
    let mut parser = Parser::new(lexer);
    let program = parser.parse().expect("Failed to parse");

    let mut compiler = Compiler::new(program, source);
    let bytecode = compiler.compile().expect("Failed to compile");

    assert!(!bytecode.is_empty());

    assert!(bytecode.len() > 10);
}

#[test]
fn float_arithmetic_operations() {
    let tests = vec![
        (
            "add ff0, ff1, ff2",
            vec![
                Opcode::AddRegRegReg as u8,
                Register::FF0 as u8,
                Register::FF1 as u8,
                Register::FF2 as u8,
            ],
        ),
        (
            "sub dd0, dd1, 3.14",
            vec![
                Opcode::SubRegRegImm as u8,
                Register::DD0 as u8,
                Register::DD1 as u8,
                0x1f,
                0x85,
                0xeb,
                0x51,
                0xb8,
                0x1e,
                0x09,
                0x40,
            ],
        ),
        (
            "mul ff0, ff1, 2.5",
            vec![
                Opcode::MulRegRegImm as u8,
                Register::FF0 as u8,
                Register::FF1 as u8,
                0x00,
                0x00,
                0x20,
                0x40,
            ],
        ),
        (
            "div dd0, dd1, dd2",
            vec![
                Opcode::DivRegRegReg as u8,
                Register::DD0 as u8,
                Register::DD1 as u8,
                Register::DD2 as u8,
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
fn float_comparison_operations() {
    let tests = vec![
        (
            "cmp ff0, 1.5",
            vec![
                Opcode::CmpRegImm as u8,
                Register::FF0 as u8,
                0x00,
                0x00,
                0xc0,
                0x3f,
            ],
        ),
        (
            "cmp dd0, 3.14159",
            vec![
                Opcode::CmpRegImm as u8,
                Register::DD0 as u8,
                0x6E,
                0x86,
                0x1B,
                0xF0,
                0xF9,
                0x21,
                0x09,
                0x40,
            ],
        ),
        (
            "cmp ff0, ff1",
            vec![
                Opcode::CmpRegReg as u8,
                Register::FF0 as u8,
                Register::FF1 as u8,
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
fn float_bitwise_operations_should_fail() {
    let tests = vec![
        "and ff0, ff1, ff2",
        "or dd0, dd1, dd2",
        "xor ff0, ff1, 42",
        "shl dd0, dd1, 2",
        "shr ff0, ff1, ff2",
        "and q0, ff1, q2",
        "or ff0, q1, ff2",
    ];

    for input in tests {
        let bytecode = compile(input);
        assert!(
            bytecode.is_err(),
            "Expected error for float bitwise operation: {}",
            input
        );
    }
}

#[test]
fn mixed_integer_float_arithmetic() {
    let tests = vec![
        (
            "add dd0, dd1, 42",
            vec![
                Opcode::AddRegRegImm as u8,
                Register::DD0 as u8,
                Register::DD1 as u8,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x45,
                0x40,
            ],
        ),
        (
            "mul ff0, ff1, 10",
            vec![
                Opcode::MulRegRegImm as u8,
                Register::FF0 as u8,
                Register::FF1 as u8,
                0x00,
                0x00,
                0x20,
                0x41,
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
fn section_label_resolution() {
    let input = r#"
.section text
start:
    mov q0, data_value

.section data
data_value:
    db 42

.section text
end:
    mov q1, data_value
    hlt
"#;

    let source = NamedSource::new("test.asm", input.to_string());
    let lexer = Lexer::new(source.clone());
    let mut parser = Parser::new(lexer);
    let program = parser.parse().expect("Failed to parse");

    let mut compiler = Compiler::new(program, source);
    let result = compiler.compile();

    assert!(result.is_ok());
}

#[test]
fn multiple_section_switches() {
    let input = r#"
.section text
func1:
    mov q0, 1

.section data
var1:
    db 1

.section text
func2:
    mov q1, 2

.section data
var2:
    db 2

.section text
main:
    hlt
"#;

    let source = NamedSource::new("test.asm", input.to_string());
    let lexer = Lexer::new(source.clone());
    let mut parser = Parser::new(lexer);
    let program = parser.parse().expect("Failed to parse");

    let mut compiler = Compiler::new(program, source);
    let result = compiler.compile();

    assert!(result.is_ok());
}
