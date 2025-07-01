use miette::{NamedSource, Result};

use crate::{lexer::Lexer, parser::Parser, vm::register::Register};

use super::*;

fn preprocess(input: &str) -> Result<Vec<Statement>> {
    let lexer = Lexer::new(NamedSource::new(
        "preprocessor_tests.nyx",
        input.to_string(),
    ));
    let mut parser = Parser::new(lexer);
    let mut program = Preprocessor::new(parser.parse()?);
    program.process()
}

#[test]
fn define() {
    let tests = vec![(
        r#"#define NUMBER 1337
_start:
    mov q0, NUMBER
    hlt"#,
        vec![
            Statement::Label("_start".into(), (20, 27).into()),
            Statement::Mov(
                Expression::Register(Register::Q0),
                Expression::IntegerLiteral(1337),
                (32, 46).into(),
            ),
            Statement::Hlt((51, 54).into()),
        ],
    )];

    for (input, expected) in tests {
        let program = preprocess(input);
        assert_ne!(program.is_err(), true);
        assert_eq!(expected, program.unwrap());
    }
}

#[test]
fn define_arithmetic() {
    let tests = vec![(
        r#"#define A 10
#define B 5
_start:
    mov q0, A
    mov q1, B
    add q2, q0, q1
    sub q3, q0, B
    mul q4, q0, A
    hlt"#,
        vec![
            Statement::Label("_start".into(), (25, 32).into()),
            Statement::Mov(
                Expression::Register(Register::Q0),
                Expression::IntegerLiteral(10),
                (37, 46).into(),
            ),
            Statement::Mov(
                Expression::Register(Register::Q1),
                Expression::IntegerLiteral(5),
                (51, 60).into(),
            ),
            Statement::Add(
                Expression::Register(Register::Q2),
                Expression::Register(Register::Q0),
                Expression::Register(Register::Q1),
                (65, 79).into(),
            ),
            Statement::Sub(
                Expression::Register(Register::Q3),
                Expression::Register(Register::Q0),
                Expression::IntegerLiteral(5),
                (84, 97).into(),
            ),
            Statement::Mul(
                Expression::Register(Register::Q4),
                Expression::Register(Register::Q0),
                Expression::IntegerLiteral(10),
                (102, 115).into(),
            ),
            Statement::Hlt((120, 123).into()),
        ],
    )];

    for (input, expected) in tests {
        let program = preprocess(input);
        assert_ne!(program.is_err(), true);
        assert_eq!(expected, program.unwrap());
    }
}

#[test]
fn define_nested() {
    let tests = vec![(
        r#"#define VALUE 42
#define DOUBLE_VALUE VALUE
_start:
    mov q0, DOUBLE_VALUE
    hlt"#,
        vec![
            Statement::Label("_start".into(), (44, 51).into()),
            Statement::Mov(
                Expression::Register(Register::Q0),
                Expression::IntegerLiteral(42),
                (56, 76).into(),
            ),
            Statement::Hlt((81, 84).into()),
        ],
    )];

    for (input, expected) in tests {
        let program = preprocess(input);
        assert_ne!(program.is_err(), true);
        assert_eq!(expected, program.unwrap());
    }
}

#[test]
fn define_addressing() {
    let tests = vec![(
        r#"#define BUFFER_ADDR 1000
#define OFFSET 16
_start:
    ldr q0, [BUFFER_ADDR, OFFSET]
    str q1, [BUFFER_ADDR]
    hlt"#,
        vec![
            Statement::Label("_start".into(), (43, 50).into()),
            Statement::Ldr(
                Expression::Register(Register::Q0),
                Expression::Address(
                    Box::new(Expression::IntegerLiteral(1000)),
                    Some(Box::new(Expression::IntegerLiteral(16))),
                ),
                (55, 84).into(),
            ),
            Statement::Str(
                Expression::Register(Register::Q1),
                Expression::Address(Box::new(Expression::IntegerLiteral(1000)), None),
                (89, 110).into(),
            ),
            Statement::Hlt((115, 118).into()),
        ],
    )];

    for (input, expected) in tests {
        let program = preprocess(input);
        assert_ne!(program.is_err(), true);
        assert_eq!(expected, program.unwrap());
    }
}
