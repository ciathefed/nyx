use crate::{lexer::Lexer, parser::Parser, vm::register::Register};

use super::*;

fn preprocess(input: &str) -> Result<Vec<Statement>> {
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let mut program = PreProcessor::new(parser.parse()?);
    program.process()
}

#[test]
fn define() {
    let tests = vec![(
        r#"
        #define NUMBER 1337
        _start:
            mov q0, NUMBER
            hlt
        "#,
        vec![
            Statement::Label("_start".into()),
            Statement::Mov(
                Expression::Register(Register::Q0),
                Expression::IntegerLiteral(1337),
            ),
            Statement::Hlt,
        ],
    )];

    for (input, expected) in tests {
        let program = preprocess(input);
        assert_ne!(program.is_err(), true);
        assert_eq!(expected, program.unwrap());
    }
}
