use super::*;

use anyhow::Result;
use pretty_assertions::{assert_eq, assert_ne};

fn parse(input: &str) -> Result<Vec<Statement>> {
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    parser.parse()
}

#[test]
fn label() {
    let tests = vec![
        ("_start:", vec![Statement::Label("_start".into())]),
        (
            "very_very_very_very_long_label:",
            vec![Statement::Label("very_very_very_very_long_label".into())],
        ),
        (
            "label_with_numbers_1337:",
            vec![Statement::Label("label_with_numbers_1337".into())],
        ),
    ];

    for (input, expected) in tests {
        let ast = parse(input);
        assert_ne!(ast.is_err(), true);
        assert_eq!(expected, ast.unwrap());
    }
}

#[test]
fn instructions() {
    let tests = vec![
        ("hlt", vec![Statement::Hlt]),
        (
            "mov q0, 1337",
            vec![Statement::Mov(
                Expression::Register(Register::Q0),
                Expression::IntegerLiteral(1337),
            )],
        ),
        (
            "ldr q0, [w0, 10]",
            vec![Statement::Ldr(
                Expression::DataSize(DataSize::QWord),
                Expression::Register(Register::Q0),
                Expression::Address(
                    Box::new(Expression::Register(Register::W0)),
                    Some(Box::new(Expression::IntegerLiteral(10))),
                ),
            )],
        ),
        (
            "str BYTE d0, [buffer]",
            vec![Statement::Str(
                Expression::DataSize(DataSize::Byte),
                Expression::Register(Register::D0),
                Expression::Address(Box::new(Expression::Identifier("buffer".into())), None),
            )],
        ),
        (
            "push q0",
            vec![Statement::Push(
                Expression::DataSize(DataSize::QWord),
                Expression::Register(Register::Q0),
            )],
        ),
        (
            "pop FLOAT ff0",
            vec![Statement::Pop(
                Expression::DataSize(DataSize::Float),
                Expression::Register(Register::FF0),
            )],
        ),
    ];

    for (input, expected) in tests {
        let ast = parse(input);
        assert_ne!(ast.is_err(), true);
        assert_eq!(expected, ast.unwrap());
    }
}
