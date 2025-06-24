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
