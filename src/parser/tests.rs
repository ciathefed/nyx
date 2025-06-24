use super::*;

use miette::Result;
use pretty_assertions::assert_eq;

fn parse(input: &str) -> Result<Vec<Statement>> {
    let lexer = Lexer::new(input);
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
        ("hlt", vec![Statement::Hlt((0, 3).into())]),
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
    ];

    for (input, expected) in tests {
        let ast = parse(input).unwrap();
        assert_eq!(expected, ast);
    }
}
