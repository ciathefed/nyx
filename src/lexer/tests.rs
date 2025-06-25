use pretty_assertions::assert_eq;

use super::*;

fn lex(input: &str) -> Vec<Token> {
    let lexer = Lexer::new(NamedSource::new("lexer_tests.nyx", input.to_string()));
    lexer.collect()
}

#[test]
fn single_character() {
    let input = ":,+-[]";

    let tokens = lex(input);

    assert_eq!(
        tokens,
        vec![
            Token::new(TokenKind::Colon, ":", (0, 1)),
            Token::new(TokenKind::Comma, ",", (1, 2)),
            Token::new(TokenKind::Plus, "+", (2, 3)),
            Token::new(TokenKind::Minus, "-", (3, 4)),
            Token::new(TokenKind::LBracket, "[", (4, 5)),
            Token::new(TokenKind::RBracket, "]", (5, 6)),
        ]
    )
}

#[test]
fn numbers() {
    let tests = vec![
        ("69", vec![Token::new(TokenKind::Integer, "69", (0, 2))]),
        ("420", vec![Token::new(TokenKind::Integer, "420", (0, 3))]),
        ("1337", vec![Token::new(TokenKind::Integer, "1337", (0, 4))]),
    ];

    for (input, expected) in tests {
        let tokens = lex(input);
        assert_eq!(tokens, expected);
    }
}

#[test]
fn hexadecimal_numbers() {
    let tests = vec![
        (
            "0x42",
            vec![Token::new(TokenKind::Hexadecimal, "0x42", (0, 4))],
        ),
        (
            "0xFF",
            vec![Token::new(TokenKind::Hexadecimal, "0xFF", (0, 4))],
        ),
        (
            "0xDEADBEEF",
            vec![Token::new(TokenKind::Hexadecimal, "0xDEADBEEF", (0, 10))],
        ),
    ];

    for (input, expected) in tests {
        let tokens = lex(input);
        assert_eq!(tokens, expected);
    }
}

#[test]
fn binary_numbers() {
    let tests = vec![
        ("0b0", vec![Token::new(TokenKind::Binary, "0b0", (0, 3))]),
        (
            "0b1010",
            vec![Token::new(TokenKind::Binary, "0b1010", (0, 6))],
        ),
        (
            "0B1101",
            vec![Token::new(TokenKind::Binary, "0B1101", (0, 6))],
        ),
    ];

    for (input, expected) in tests {
        let tokens = lex(input);
        assert_eq!(tokens, expected);
    }
}

#[test]
fn octal_numbers() {
    let tests = vec![
        ("0o0", vec![Token::new(TokenKind::Octal, "0o0", (0, 3))]),
        ("0o123", vec![Token::new(TokenKind::Octal, "0o123", (0, 5))]),
        ("0O777", vec![Token::new(TokenKind::Octal, "0O777", (0, 5))]),
    ];

    for (input, expected) in tests {
        let tokens = lex(input);
        assert_eq!(tokens, expected);
    }
}

#[test]
fn identifiers() {
    let tests = vec![
        (
            "variable_name",
            vec![Token::new(TokenKind::Identifier, "variable_name", (0, 13))],
        ),
        (
            "_long_long_long_12345_name",
            vec![Token::new(
                TokenKind::Identifier,
                "_long_long_long_12345_name",
                (0, 26),
            )],
        ),
    ];

    for (input, expected) in tests {
        let tokens = lex(input);
        assert_eq!(tokens, expected);
    }
}

#[test]
fn preprocessor_directives() {
    let tests = vec![(
        "#define",
        vec![Token::new(TokenKind::KwDefine, "#define", (0, 7))],
    )];

    for (input, expected) in tests {
        let tokens = lex(input);
        assert_eq!(tokens, expected);
    }
}

#[test]
fn instructions() {
    let tests = vec![
        ("nop", vec![Token::new(TokenKind::KwNop, "nop", (0, 3))]),
        ("mov", vec![Token::new(TokenKind::KwMov, "mov", (0, 3))]),
        ("ldr", vec![Token::new(TokenKind::KwLdr, "ldr", (0, 3))]),
        ("str", vec![Token::new(TokenKind::KwStr, "str", (0, 3))]),
        ("push", vec![Token::new(TokenKind::KwPush, "push", (0, 4))]),
        ("pop", vec![Token::new(TokenKind::KwPop, "pop", (0, 3))]),
        ("hlt", vec![Token::new(TokenKind::KwHlt, "hlt", (0, 3))]),
    ];

    for (input, expected) in tests {
        let tokens = lex(input);
        assert_eq!(tokens, expected);
    }
}

#[test]
fn data_decleration_directives() {
    let tests = vec![("db", vec![Token::new(TokenKind::KwDb, "db", (0, 2))])];

    for (input, expected) in tests {
        let tokens = lex(input);
        assert_eq!(tokens, expected);
    }
}

#[test]
fn strings() {
    let tests = vec![
        (
            r#""this is a string!""#,
            vec![Token::new(TokenKind::String, "this is a string!", (0, 19))],
        ),
        (
            r#""this is a very very very very very long string!""#,
            vec![Token::new(
                TokenKind::String,
                "this is a very very very very very long string!",
                (0, 49),
            )],
        ),
    ];

    for (input, expected) in tests {
        let tokens = lex(input);
        assert_eq!(tokens, expected);
    }
}
