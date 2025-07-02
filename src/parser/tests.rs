use super::*;

use miette::{NamedSource, Result};
use pretty_assertions::assert_eq;

fn parse(input: &str) -> Result<Vec<Statement>> {
    let lexer = Lexer::new(NamedSource::new("parser_tests.nyx", input.to_string()));
    let mut parser = Parser::new(lexer);
    parser.parse()
}

#[test]
fn label() {
    let tests = vec![
        (
            "_start:",
            vec![Statement::Label("_start".into(), (0, 7).into())],
        ),
        (
            "very_very_very_very_long_label:",
            vec![Statement::Label(
                "very_very_very_very_long_label".into(),
                (0, 31).into(),
            )],
        ),
        (
            "label_with_numbers_1337:",
            vec![Statement::Label(
                "label_with_numbers_1337".into(),
                (0, 24).into(),
            )],
        ),
    ];

    for (input, expected) in tests {
        let ast = parse(input).unwrap();
        assert_eq!(expected, ast);
    }
}

#[test]
fn instructions() {
    let tests = vec![
        ("nop", vec![Statement::Nop((0, 3).into())]),
        (
            "mov q0, 1337",
            vec![Statement::Mov(
                Expression::Register(Register::Q0),
                Expression::IntegerLiteral(1337),
                (0, 12).into(),
            )],
        ),
        (
            "ldr q0, [w0, 10]",
            vec![Statement::Ldr(
                Expression::Register(Register::Q0),
                Expression::Address(
                    Box::new(Expression::Register(Register::W0)),
                    Some(Box::new(Expression::IntegerLiteral(10))),
                ),
                (0, 16).into(),
            )],
        ),
        (
            "str d0, [buffer]",
            vec![Statement::Str(
                Expression::Register(Register::D0),
                Expression::Address(Box::new(Expression::Identifier("buffer".into())), None),
                (0, 16).into(),
            )],
        ),
        (
            "push q0",
            vec![Statement::Push(
                None,
                Expression::Register(Register::Q0),
                (0, 7).into(),
            )],
        ),
        (
            "pop FLOAT ff0",
            vec![Statement::Pop(
                Some(Expression::DataSize(DataSize::Float)),
                Expression::Register(Register::FF0),
                (0, 13).into(),
            )],
        ),
        (
            "cmp q0, 13",
            vec![Statement::Cmp(
                Expression::Register(Register::Q0),
                Expression::IntegerLiteral(13),
                (0, 10).into(),
            )],
        ),
        (
            "call function_name",
            vec![Statement::Call(
                Expression::Identifier("function_name".into()),
                (0, 18).into(),
            )],
        ),
        (
            "inc q0",
            vec![Statement::Inc(
                Expression::Register(Register::Q0),
                (0, 6).into(),
            )],
        ),
        (
            "dec q0",
            vec![Statement::Dec(
                Expression::Register(Register::Q0),
                (0, 6).into(),
            )],
        ),
        ("ret", vec![Statement::Ret((0, 3).into())]),
        ("hlt", vec![Statement::Hlt((0, 3).into())]),
    ];

    for (input, expected) in tests {
        let ast = parse(input).unwrap();
        assert_eq!(expected, ast);
    }
}

#[test]
fn arithmetic_operations() {
    let tests = vec![
        (
            "add q0, q1, q2",
            vec![Statement::Add(
                Expression::Register(Register::Q0),
                Expression::Register(Register::Q1),
                Expression::Register(Register::Q2),
                (0, 14).into(),
            )],
        ),
        (
            "sub d0, d1, 42",
            vec![Statement::Sub(
                Expression::Register(Register::D0),
                Expression::Register(Register::D1),
                Expression::IntegerLiteral(42),
                (0, 14).into(),
            )],
        ),
        (
            "mul w0, w1, w2",
            vec![Statement::Mul(
                Expression::Register(Register::W0),
                Expression::Register(Register::W1),
                Expression::Register(Register::W2),
                (0, 14).into(),
            )],
        ),
        (
            "div b0, b1, 10",
            vec![Statement::Div(
                Expression::Register(Register::B0),
                Expression::Register(Register::B1),
                Expression::IntegerLiteral(10),
                (0, 14).into(),
            )],
        ),
    ];

    for (input, expected) in tests {
        let ast = parse(input).unwrap();
        assert_eq!(expected, ast);
    }
}

#[test]
fn bitwise_operations() {
    let tests = vec![
        (
            "and q0, q1, q2",
            vec![Statement::And(
                Expression::Register(Register::Q0),
                Expression::Register(Register::Q1),
                Expression::Register(Register::Q2),
                (0, 14).into(),
            )],
        ),
        (
            "or d0, d1, 255",
            vec![Statement::Or(
                Expression::Register(Register::D0),
                Expression::Register(Register::D1),
                Expression::IntegerLiteral(255),
                (0, 14).into(),
            )],
        ),
        (
            "xor w0, w1, w2",
            vec![Statement::Xor(
                Expression::Register(Register::W0),
                Expression::Register(Register::W1),
                Expression::Register(Register::W2),
                (0, 14).into(),
            )],
        ),
    ];

    for (input, expected) in tests {
        let ast = parse(input).unwrap();
        assert_eq!(expected, ast);
    }
}

#[test]
fn shift_operations() {
    let tests = vec![
        (
            "shl b0, b1, 4",
            vec![Statement::Shl(
                Expression::Register(Register::B0),
                Expression::Register(Register::B1),
                Expression::IntegerLiteral(4),
                (0, 13).into(),
            )],
        ),
        (
            "shr q0, q1, q2",
            vec![Statement::Shr(
                Expression::Register(Register::Q0),
                Expression::Register(Register::Q1),
                Expression::Register(Register::Q2),
                (0, 14).into(),
            )],
        ),
    ];

    for (input, expected) in tests {
        let ast = parse(input).unwrap();
        assert_eq!(expected, ast);
    }
}

#[test]
fn jump_operations() {
    let tests = vec![
        (
            "jmp 0x37",
            vec![Statement::Jmp(
                Expression::IntegerLiteral(0x37),
                (0, 8).into(),
            )],
        ),
        (
            "jne _exit",
            vec![Statement::Jne(
                Expression::Identifier("_exit".into()),
                (0, 9).into(),
            )],
        ),
        (
            "jge q0",
            vec![Statement::Jge(
                Expression::Register(Register::Q0),
                (0, 6).into(),
            )],
        ),
    ];

    for (input, expected) in tests {
        let ast = parse(input).unwrap();
        assert_eq!(expected, ast);
    }
}

#[test]
fn expressions() {
    let tests = vec![
        (
            "mov q0, 0xFF",
            vec![Statement::Mov(
                Expression::Register(Register::Q0),
                Expression::IntegerLiteral(255),
                (0, 12).into(),
            )],
        ),
        (
            "mov q0, 0b1010",
            vec![Statement::Mov(
                Expression::Register(Register::Q0),
                Expression::IntegerLiteral(10),
                (0, 14).into(),
            )],
        ),
        (
            "mov q0, 0o777",
            vec![Statement::Mov(
                Expression::Register(Register::Q0),
                Expression::IntegerLiteral(511),
                (0, 13).into(),
            )],
        ),
        (
            "mov ff0, 3.14",
            vec![Statement::Mov(
                Expression::Register(Register::FF0),
                Expression::FloatLiteral(3.14),
                (0, 13).into(),
            )],
        ),
    ];

    for (input, expected) in tests {
        let ast = parse(input).unwrap();
        assert_eq!(expected, ast);
    }
}

#[test]
fn addressing_modes() {
    let tests = vec![
        (
            "ldr q0, [q1]",
            vec![Statement::Ldr(
                Expression::Register(Register::Q0),
                Expression::Address(Box::new(Expression::Register(Register::Q1)), None),
                (0, 12).into(),
            )],
        ),
        (
            "str b0, [1000]",
            vec![Statement::Str(
                Expression::Register(Register::B0),
                Expression::Address(Box::new(Expression::IntegerLiteral(1000)), None),
                (0, 14).into(),
            )],
        ),
        (
            "ldr w0, [buffer, 16]",
            vec![Statement::Ldr(
                Expression::Register(Register::W0),
                Expression::Address(
                    Box::new(Expression::Identifier("buffer".into())),
                    Some(Box::new(Expression::IntegerLiteral(16))),
                ),
                (0, 20).into(),
            )],
        ),
    ];

    for (input, expected) in tests {
        let ast = parse(input).unwrap();
        assert_eq!(expected, ast);
    }
}

#[test]
fn data_declarations() {
    let tests = vec![
        (
            r#"db 42"#,
            vec![Statement::Db(
                vec![Expression::IntegerLiteral(42)],
                (0, 5).into(),
            )],
        ),
        (
            r#"db "Hello", 0x00"#,
            vec![Statement::Db(
                vec![
                    Expression::StringLiteral("Hello".into()),
                    Expression::IntegerLiteral(0),
                ],
                (0, 16).into(),
            )],
        ),
        (
            r#"db 1, 2, 3, 4"#,
            vec![Statement::Db(
                vec![
                    Expression::IntegerLiteral(1),
                    Expression::IntegerLiteral(2),
                    Expression::IntegerLiteral(3),
                    Expression::IntegerLiteral(4),
                ],
                (0, 13).into(),
            )],
        ),
        (
            r#"resb 69"#,
            vec![Statement::Resb(
                Expression::IntegerLiteral(69),
                (0, 7).into(),
            )],
        ),
        (
            r#"resb 1024"#,
            vec![Statement::Resb(
                Expression::IntegerLiteral(1024),
                (0, 9).into(),
            )],
        ),
    ];

    for (input, expected) in tests {
        let ast = parse(input).unwrap();
        assert_eq!(expected, ast);
    }
}

#[test]
fn sections() {
    let tests = vec![
        (
            ".section text",
            vec![Statement::Section(SectionType::Text, (0, 13).into())],
        ),
        (
            ".section data",
            vec![Statement::Section(SectionType::Data, (0, 13).into())],
        ),
    ];

    for (input, expected) in tests {
        let ast = parse(input).unwrap();
        assert_eq!(expected, ast);
    }
}

#[test]
fn complex_program() {
    let input = r#"
.entry _start
.section text
_start:
    mov q0, 42
    add q1, q0, 100
    push QWORD q1
    syscall
    hlt

.section data
message:
    db "Hello, World!", 0x00"#;

    let ast = parse(input).unwrap();

    assert_eq!(ast.len(), 11);

    match &ast[0] {
        Statement::Entry(Expression::Identifier(_), _) => (),
        _ => panic!("Expected identifier"),
    }

    match &ast[1] {
        Statement::Section(SectionType::Text, _) => (),
        _ => panic!("Expected text section"),
    }

    match &ast[2] {
        Statement::Label(name, _) => assert_eq!(name, "_start"),
        _ => panic!("Expected label"),
    }

    assert!(matches!(ast[3], Statement::Mov(_, _, _)));
    assert!(matches!(ast[4], Statement::Add(_, _, _, _)));
    assert!(matches!(ast[5], Statement::Push(_, _, _)));
    assert!(matches!(ast[6], Statement::Syscall(_)));
    assert!(matches!(ast[7], Statement::Hlt(_)));

    match &ast[8] {
        Statement::Section(SectionType::Data, _) => (),
        _ => panic!("Expected data section"),
    }

    match &ast[10] {
        Statement::Db(exprs, _) => {
            assert_eq!(exprs.len(), 2);
            match &exprs[0] {
                Expression::StringLiteral(s) => assert_eq!(s, "Hello, World!"),
                _ => panic!("Expected string literal"),
            }
        }
        _ => panic!("Expected data declaration"),
    }
}
