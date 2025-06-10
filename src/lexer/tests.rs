use pretty_assertions::assert_eq;

use super::*;

fn lex(input: &str) -> Vec<Token> {
    let lexer = Lexer::new(input);
    lexer.collect()
}

#[test]
fn single_character() {
    let input = ":,+-";

    let tokens = lex(input);

    assert_eq!(
        tokens,
        vec![
            Token::new(TokenKind::Colon, ":", (0, 1).into()),
            Token::new(TokenKind::Comma, ",", (1, 2).into()),
            Token::new(TokenKind::Plus, "+", (2, 3).into()),
            Token::new(TokenKind::Minus, "-", (3, 4).into())
        ]
    )
}

#[test]
fn numbers() {
    let tests = vec![
        (
            "69",
            vec![Token::new(TokenKind::Integer, "69", (0, 2).into())],
        ),
        (
            "420",
            vec![Token::new(TokenKind::Integer, "420", (0, 3).into())],
        ),
        (
            "1337",
            vec![Token::new(TokenKind::Integer, "1337", (0, 4).into())],
        ),
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
            vec![Token::new(
                TokenKind::Identifier,
                "variable_name",
                (0, 13).into(),
            )],
        ),
        (
            "_long_long_long_12345_name",
            vec![Token::new(
                TokenKind::Identifier,
                "_long_long_long_12345_name",
                (0, 26).into(),
            )],
        ),
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
            vec![Token::new(
                TokenKind::String,
                "this is a string!",
                (0, 19).into(),
            )],
        ),
        (
            r#""this is a very very very very very long string!""#,
            vec![Token::new(
                TokenKind::String,
                "this is a very very very very very long string!",
                (0, 49).into(),
            )],
        ),
    ];

    for (input, expected) in tests {
        let tokens = lex(input);
        assert_eq!(tokens, expected);
    }
}
