use miette::{NamedSource, Result};
use pretty_assertions::assert_eq;
use tempfile::TempDir;

use crate::{lexer::Lexer, parser::Parser, vm::register::Register};

use super::*;

fn preprocess(input: &str) -> Result<Vec<Statement>> {
    let input = Arc::new(NamedSource::new(
        "preprocessor_tests.nyx",
        input.to_string(),
    ));
    let lexer = Lexer::new(input.clone());
    let mut parser = Parser::new(lexer);
    let mut program = Preprocessor::new(parser.parse()?, input);
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

#[test]
fn include_basic() {
    let temp_dir = TempDir::new().unwrap();
    let include_path = temp_dir.path().join("header.nyx");

    fs::write(
        &include_path,
        r#"#define MAGIC_NUMBER 42
helper_function:
    mov q0, MAGIC_NUMBER
    ret"#,
    )
    .unwrap();

    let main_code = format!(
        r#"#include "{}"
_start:
    call helper_function
    hlt"#,
        include_path.file_name().unwrap().to_str().unwrap()
    );

    let expected = vec![
        Statement::Label("helper_function".into(), (24, 40).into()),
        Statement::Mov(
            Expression::Register(Register::Q0),
            Expression::IntegerLiteral(42),
            (45, 65).into(),
        ),
        Statement::Ret((70, 73).into()),
        Statement::Label("_start".into(), (22, 29).into()),
        Statement::Call(
            Expression::Identifier("helper_function".into()),
            (34, 54).into(),
        ),
        Statement::Hlt((59, 62).into()),
    ];

    let input = Arc::new(NamedSource::new("test.nyx", main_code));
    let lexer = Lexer::new(input.clone());
    let mut parser = Parser::new(lexer);
    let mut preprocessor = Preprocessor::new(parser.parse().unwrap(), input)
        .with_include_paths(vec![temp_dir.path().to_path_buf()]);

    let result = preprocessor.process();
    assert!(result.is_ok());
    assert_eq!(expected, result.unwrap());
}

#[test]
fn include_with_defines() {
    let temp_dir = TempDir::new().unwrap();
    let constants_path = temp_dir.path().join("constants.nyx");

    fs::write(
        &constants_path,
        r#"#define STACK_SIZE 1024
#define HEAP_START 2048
#define MAX_ITERATIONS 100"#,
    )
    .unwrap();

    let main_code = format!(
        r#"#include "{}"
#define BUFFER_SIZE STACK_SIZE
_start:
    mov q0, BUFFER_SIZE
    mov q1, HEAP_START
    mov q2, MAX_ITERATIONS
    hlt"#,
        constants_path.file_name().unwrap().to_str().unwrap()
    );

    let expected = vec![
        Statement::Label("_start".into(), (56, 63).into()),
        Statement::Mov(
            Expression::Register(Register::Q0),
            Expression::IntegerLiteral(1024),
            (68, 87).into(),
        ),
        Statement::Mov(
            Expression::Register(Register::Q1),
            Expression::IntegerLiteral(2048),
            (92, 110).into(),
        ),
        Statement::Mov(
            Expression::Register(Register::Q2),
            Expression::IntegerLiteral(100),
            (115, 137).into(),
        ),
        Statement::Hlt((142, 145).into()),
    ];

    let input = Arc::new(NamedSource::new("test.nyx", main_code));
    let lexer = Lexer::new(input.clone());
    let mut parser = Parser::new(lexer);
    let mut preprocessor = Preprocessor::new(parser.parse().unwrap(), input)
        .with_include_paths(vec![temp_dir.path().to_path_buf()]);

    let result = preprocessor.process();
    assert!(result.is_ok());
    assert_eq!(expected, result.unwrap());
}

#[test]
fn include_nested() {
    let temp_dir = TempDir::new().unwrap();

    let math_ops_path = temp_dir.path().join("math_ops.nyx");
    fs::write(
        &math_ops_path,
        r#"add_func:
    add q0, q1, q2
    ret"#,
    )
    .unwrap();

    let utils_path = temp_dir.path().join("utils.nyx");
    fs::write(
        &utils_path,
        format!(
            r#"#include "{}"
#define RESULT_REG q0"#,
            math_ops_path.file_name().unwrap().to_str().unwrap()
        ),
    )
    .unwrap();

    let main_code = format!(
        r#"#include "{}"
_start:
    call add_func
    mov RESULT_REG, q3
    hlt"#,
        utils_path.file_name().unwrap().to_str().unwrap()
    );

    let input = Arc::new(NamedSource::new("test.nyx", main_code));
    let lexer = Lexer::new(input.clone());
    let mut parser = Parser::new(lexer);
    let mut preprocessor = Preprocessor::new(parser.parse().unwrap(), input)
        .with_include_paths(vec![temp_dir.path().to_path_buf()]);

    let result = preprocessor.process();
    assert!(result.is_ok());

    let statements = result.unwrap();
    assert!(statements.len() > 3);
}

#[test]
fn include_multiple_files() {
    let temp_dir = TempDir::new().unwrap();

    let constants_path = temp_dir.path().join("constants.nyx");
    fs::write(
        &constants_path,
        r#"#define PI 314
#define E 271"#,
    )
    .unwrap();

    let functions_path = temp_dir.path().join("functions.nyx");
    fs::write(
        &functions_path,
        r#"square:
    mul q0, q0, q0
    ret"#,
    )
    .unwrap();

    let main_code = format!(
        r#"#include "{}"
#include "{}"
_start:
    mov q0, PI
    call square
    mov q1, E
    hlt"#,
        constants_path.file_name().unwrap().to_str().unwrap(),
        functions_path.file_name().unwrap().to_str().unwrap()
    );

    let input = Arc::new(NamedSource::new("test.nyx", main_code));
    let lexer = Lexer::new(input.clone());
    let mut parser = Parser::new(lexer);
    let mut preprocessor = Preprocessor::new(parser.parse().unwrap(), input)
        .with_include_paths(vec![temp_dir.path().to_path_buf()]);

    let result = preprocessor.process();
    assert!(result.is_ok());

    let statements = result.unwrap();
    assert!(
        statements
            .iter()
            .any(|s| matches!(s, Statement::Label(name, _) if name == "square"))
    );
    assert!(
        statements
            .iter()
            .any(|s| matches!(s, Statement::Label(name, _) if name == "_start"))
    );
}

#[test]
fn include_file_not_found() {
    let main_code = r#"#include "nonexistent.nyx"
_start:
    hlt"#;

    let input = Arc::new(NamedSource::new("test.nyx", main_code.to_string()));
    let lexer = Lexer::new(input.clone());
    let mut parser = Parser::new(lexer);
    let mut preprocessor = Preprocessor::new(parser.parse().unwrap(), input);

    let result = preprocessor.process();
    assert!(result.is_err());
}

#[test]
fn include_circular_dependency() {
    let temp_dir = TempDir::new().unwrap();

    let file_a_path = temp_dir.path().join("a.nyx");
    let file_b_path = temp_dir.path().join("b.nyx");

    fs::write(
        &file_a_path,
        format!(
            r#"#include "{}"
#define FROM_A 1"#,
            file_b_path.file_name().unwrap().to_str().unwrap()
        ),
    )
    .unwrap();

    fs::write(
        &file_b_path,
        format!(
            r#"#include "{}"
#define FROM_B 2"#,
            file_a_path.file_name().unwrap().to_str().unwrap()
        ),
    )
    .unwrap();

    let main_code = format!(
        r#"#include "{}"
_start:
    hlt"#,
        file_a_path.file_name().unwrap().to_str().unwrap()
    );

    let input = Arc::new(NamedSource::new("test.nyx", main_code));
    let lexer = Lexer::new(input.clone());
    let mut parser = Parser::new(lexer);
    let mut preprocessor = Preprocessor::new(parser.parse().unwrap(), input)
        .with_include_paths(vec![temp_dir.path().to_path_buf()]);

    let result = preprocessor.process();
    assert!(result.is_err());
}

#[test]
fn include_with_multiple_include_paths() {
    let temp_dir1 = TempDir::new().unwrap();
    let temp_dir2 = TempDir::new().unwrap();

    let file1_path = temp_dir1.path().join("common.nyx");
    let file2_path = temp_dir2.path().join("specific.nyx");

    fs::write(&file1_path, r#"#define COMMON_CONST 100"#).unwrap();
    fs::write(&file2_path, r#"#define SPECIFIC_CONST 200"#).unwrap();

    let main_code = r#"#include "common.nyx"
#include "specific.nyx"
_start:
    mov q0, COMMON_CONST
    mov q1, SPECIFIC_CONST
    hlt"#;

    let input = Arc::new(NamedSource::new("test.nyx", main_code.to_string()));
    let lexer = Lexer::new(input.clone());
    let mut parser = Parser::new(lexer);
    let mut preprocessor =
        Preprocessor::new(parser.parse().unwrap(), input).with_include_paths(vec![
            temp_dir1.path().to_path_buf(),
            temp_dir2.path().to_path_buf(),
        ]);

    let result = preprocessor.process();
    assert!(result.is_ok());

    let statements = result.unwrap();
    assert!(
        statements
            .iter()
            .any(|s| matches!(s, Statement::Mov(_, Expression::IntegerLiteral(100), _)))
    );
    assert!(
        statements
            .iter()
            .any(|s| matches!(s, Statement::Mov(_, Expression::IntegerLiteral(200), _)))
    );
}

#[test]
fn ifdef_true_branch() {
    let input = r#"
#define ENABLE 1
#ifdef ENABLE
_start:
    mov q0, 123
#endif
    hlt"#;

    let expected = vec![
        Statement::Label("_start".into(), (32, 39).into()),
        Statement::Mov(
            Expression::Register(Register::Q0),
            Expression::IntegerLiteral(123),
            (44, 55).into(),
        ),
        Statement::Hlt((67, 70).into()),
    ];

    let program = preprocess(input);
    assert!(program.is_ok());
    assert_eq!(expected, program.unwrap());
}

#[test]
fn ifdef_false_branch() {
    let input = r#"
#ifdef MISSING
_start:
    mov q0, 123
#endif
    hlt"#;

    let expected = vec![Statement::Hlt((51, 54).into())];

    let program = preprocess(input);
    assert!(program.is_ok());
    assert_eq!(expected, program.unwrap());
}

#[test]
fn ifndef_true_branch() {
    let input = r#"
#ifndef MISSING
_start:
    mov q0, 456
#endif
    hlt"#;

    let expected = vec![
        Statement::Label("_start".into(), (17, 24).into()),
        Statement::Mov(
            Expression::Register(Register::Q0),
            Expression::IntegerLiteral(456),
            (29, 40).into(),
        ),
        Statement::Hlt((52, 55).into()),
    ];

    let program = preprocess(input);
    assert!(program.is_ok());
    assert_eq!(expected, program.unwrap());
}

#[test]
fn ifndef_false_branch() {
    let input = r#"
#define FEATURE 1
#ifndef FEATURE
_start:
    mov q0, 999
#endif
    hlt"#;

    let expected = vec![Statement::Hlt((70, 73).into())];

    let program = preprocess(input);
    assert!(program.is_ok());
    assert_eq!(expected, program.unwrap());
}

#[test]
fn ifdef_with_else_true_branch() {
    let input = r#"
#define DEBUG 1
#ifdef DEBUG
_start:
    mov q0, 111
#else
    mov q0, 222
#endif
    hlt"#;

    let expected = vec![
        Statement::Label("_start".into(), (30, 37).into()),
        Statement::Mov(
            Expression::Register(Register::Q0),
            Expression::IntegerLiteral(111),
            (42, 53).into(),
        ),
        Statement::Hlt((87, 90).into()),
    ];

    let program = preprocess(input);
    assert!(program.is_ok());
    assert_eq!(expected, program.unwrap());
}

#[test]
fn ifdef_with_else_false_branch() {
    let input = r#"
#ifdef DEBUG
_start:
    mov q0, 111
#else
    mov q0, 222
#endif
    hlt"#;

    let expected = vec![
        Statement::Mov(
            Expression::Register(Register::Q0),
            Expression::IntegerLiteral(222),
            (48, 59).into(),
        ),
        Statement::Hlt((71, 74).into()),
    ];

    let program = preprocess(input);
    assert!(program.is_ok());
    assert_eq!(expected, program.unwrap());
}

#[test]
fn error() {
    let input = r#"#error "this is an error!""#;

    let result = preprocess(input);

    assert!(result.is_err());

    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("this is an error!"),
        "error message missing or wrong: {err}"
    );
}
