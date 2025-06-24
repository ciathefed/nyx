use pretty_assertions::assert_eq;

use super::*;

fn lex(input: &str) -> Vec<Token> {
    let lexer = Lexer::new(input);
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
fn identifier() {
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
fn keyword() {
    let tests = vec![
        ("mov", vec![Token::new(TokenKind::KwMov, "mov", (0, 3))]),
        ("ldr", vec![Token::new(TokenKind::KwLdr, "ldr", (0, 3))]),
        ("str", vec![Token::new(TokenKind::KwStr, "str", (0, 3))]),
        ("push", vec![Token::new(TokenKind::KwPush, "push", (0, 4))]),
        ("pop", vec![Token::new(TokenKind::KwPop, "pop", (0, 3))]),
    ];

    for (input, expected) in tests {
        let tokens = lex(input);
        assert_eq!(tokens, expected);
    }
}

#[test]
fn string() {
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
