use pretty_assertions::assert_eq;

use super::*;

fn lex(input: &str) -> Vec<Token> {
    let lexer = Lexer::new(Arc::new(NamedSource::new(
        "lexer_tests.nyx",
        input.to_string(),
    )));
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
    let tests = vec![
        (
            "#define",
            vec![Token::new(TokenKind::KwDefine, "#define", (0, 7))],
        ),
        (
            "#include",
            vec![Token::new(TokenKind::KwInclude, "#include", (0, 8))],
        ),
        (
            "#ifdef",
            vec![Token::new(TokenKind::KwIfDef, "#ifdef", (0, 6))],
        ),
        (
            "#ifndef",
            vec![Token::new(TokenKind::KwIfNDef, "#ifndef", (0, 7))],
        ),
        (
            "#else",
            vec![Token::new(TokenKind::KwElse, "#else", (0, 5))],
        ),
        (
            "#endif",
            vec![Token::new(TokenKind::KwEndIf, "#endif", (0, 6))],
        ),
    ];

    for (input, expected) in tests {
        let tokens = lex(input);
        assert_eq!(tokens, expected);
    }
}

#[test]
fn basic_instructions() {
    let tests = vec![
        ("nop", vec![Token::new(TokenKind::KwNop, "nop", (0, 3))]),
        ("mov", vec![Token::new(TokenKind::KwMov, "mov", (0, 3))]),
        ("ldr", vec![Token::new(TokenKind::KwLdr, "ldr", (0, 3))]),
        ("str", vec![Token::new(TokenKind::KwStr, "str", (0, 3))]),
        ("push", vec![Token::new(TokenKind::KwPush, "push", (0, 4))]),
        ("pop", vec![Token::new(TokenKind::KwPop, "pop", (0, 3))]),
        ("cmp", vec![Token::new(TokenKind::KwCmp, "cmp", (0, 3))]),
        ("call", vec![Token::new(TokenKind::KwCall, "call", (0, 4))]),
        ("ret", vec![Token::new(TokenKind::KwRet, "ret", (0, 3))]),
        ("inc", vec![Token::new(TokenKind::KwInc, "inc", (0, 3))]),
        ("dec", vec![Token::new(TokenKind::KwDec, "dec", (0, 3))]),
        (
            "syscall",
            vec![Token::new(TokenKind::KwSyscall, "syscall", (0, 7))],
        ),
        ("hlt", vec![Token::new(TokenKind::KwHlt, "hlt", (0, 3))]),
    ];

    for (input, expected) in tests {
        let tokens = lex(input);
        assert_eq!(tokens, expected);
    }
}

#[test]
fn arithmetic_instructions() {
    let tests = vec![
        ("add", vec![Token::new(TokenKind::KwAdd, "add", (0, 3))]),
        ("sub", vec![Token::new(TokenKind::KwSub, "sub", (0, 3))]),
        ("mul", vec![Token::new(TokenKind::KwMul, "mul", (0, 3))]),
        ("div", vec![Token::new(TokenKind::KwDiv, "div", (0, 3))]),
    ];

    for (input, expected) in tests {
        let tokens = lex(input);
        assert_eq!(tokens, expected);
    }
}

#[test]
fn bitwise_instructions() {
    let tests = vec![
        ("and", vec![Token::new(TokenKind::KwAnd, "and", (0, 3))]),
        ("or", vec![Token::new(TokenKind::KwOr, "or", (0, 2))]),
        ("xor", vec![Token::new(TokenKind::KwXor, "xor", (0, 3))]),
    ];

    for (input, expected) in tests {
        let tokens = lex(input);
        assert_eq!(tokens, expected);
    }
}

#[test]
fn shift_instructions() {
    let tests = vec![
        ("shl", vec![Token::new(TokenKind::KwShl, "shl", (0, 3))]),
        ("shr", vec![Token::new(TokenKind::KwShr, "shr", (0, 3))]),
    ];

    for (input, expected) in tests {
        let tokens = lex(input);
        assert_eq!(tokens, expected);
    }
}

#[test]
fn jump_instructions() {
    let tests = vec![
        ("jmp", vec![Token::new(TokenKind::KwJmp, "jmp", (0, 3))]),
        ("jeq", vec![Token::new(TokenKind::KwJeq, "jeq", (0, 3))]),
        ("jne", vec![Token::new(TokenKind::KwJne, "jne", (0, 3))]),
        ("jlt", vec![Token::new(TokenKind::KwJlt, "jlt", (0, 3))]),
        ("jgt", vec![Token::new(TokenKind::KwJgt, "jgt", (0, 3))]),
        ("jle", vec![Token::new(TokenKind::KwJle, "jle", (0, 3))]),
        ("jge", vec![Token::new(TokenKind::KwJge, "jge", (0, 3))]),
    ];

    for (input, expected) in tests {
        let tokens = lex(input);
        assert_eq!(tokens, expected);
    }
}

#[test]
fn data_decleration_directives() {
    let tests = vec![
        ("db", vec![Token::new(TokenKind::KwDb, "db", (0, 2))]),
        ("resb", vec![Token::new(TokenKind::KwResb, "resb", (0, 4))]),
    ];

    for (input, expected) in tests {
        let tokens = lex(input);
        assert_eq!(tokens, expected);
    }
}

#[test]
fn float_numbers() {
    let tests = vec![
        ("3.14", vec![Token::new(TokenKind::Float, "3.14", (0, 4))]),
        ("0.5", vec![Token::new(TokenKind::Float, "0.5", (0, 3))]),
        (
            "123.456",
            vec![Token::new(TokenKind::Float, "123.456", (0, 7))],
        ),
        ("0.0", vec![Token::new(TokenKind::Float, "0.0", (0, 3))]),
        (
            "999.999",
            vec![Token::new(TokenKind::Float, "999.999", (0, 7))],
        ),
    ];

    for (input, expected) in tests {
        let tokens = lex(input);
        assert_eq!(tokens, expected);
    }
}

#[test]
fn mixed_numbers() {
    let tests = vec![
        (
            "42 3.14 0xFF",
            vec![
                Token::new(TokenKind::Integer, "42", (0, 2)),
                Token::new(TokenKind::Float, "3.14", (3, 7)),
                Token::new(TokenKind::Hexadecimal, "0xFF", (8, 12)),
            ],
        ),
        (
            "0b1010 420.69 0o777",
            vec![
                Token::new(TokenKind::Binary, "0b1010", (0, 6)),
                Token::new(TokenKind::Float, "420.69", (7, 13)),
                Token::new(TokenKind::Octal, "0o777", (14, 19)),
            ],
        ),
    ];

    for (input, expected) in tests {
        let tokens = lex(input);
        assert_eq!(tokens, expected);
    }
}

#[test]
fn register_tokens() {
    let tests = vec![
        (
            "b0 w1 d2 q3",
            vec![
                Token::new(TokenKind::Register, "b0", (0, 2)),
                Token::new(TokenKind::Register, "w1", (3, 5)),
                Token::new(TokenKind::Register, "d2", (6, 8)),
                Token::new(TokenKind::Register, "q3", (9, 11)),
            ],
        ),
        (
            "ff0 dd1 ip sp bp",
            vec![
                Token::new(TokenKind::Register, "ff0", (0, 3)),
                Token::new(TokenKind::Register, "dd1", (4, 7)),
                Token::new(TokenKind::Register, "ip", (8, 10)),
                Token::new(TokenKind::Register, "sp", (11, 13)),
                Token::new(TokenKind::Register, "bp", (14, 16)),
            ],
        ),
    ];

    for (input, expected) in tests {
        let tokens = lex(input);
        assert_eq!(tokens, expected);
    }
}

#[test]
fn data_size_tokens() {
    let tests = vec![
        (
            "byte word dword qword",
            vec![
                Token::new(TokenKind::DataSize, "byte", (0, 4)),
                Token::new(TokenKind::DataSize, "word", (5, 9)),
                Token::new(TokenKind::DataSize, "dword", (10, 15)),
                Token::new(TokenKind::DataSize, "qword", (16, 21)),
            ],
        ),
        (
            "float double",
            vec![
                Token::new(TokenKind::DataSize, "float", (0, 5)),
                Token::new(TokenKind::DataSize, "double", (6, 12)),
            ],
        ),
    ];

    for (input, expected) in tests {
        let tokens = lex(input);
        assert_eq!(tokens, expected);
    }
}

#[test]
fn section_names() {
    let tests = vec![(
        "text data",
        vec![
            Token::new(TokenKind::SectionName, "text", (0, 4)),
            Token::new(TokenKind::SectionName, "data", (5, 9)),
        ],
    )];

    for (input, expected) in tests {
        let tokens = lex(input);
        assert_eq!(tokens, expected);
    }
}

#[test]
fn complex_program() {
    let input = r#".entry _start
.section text
_start:
    mov q0, 42
    add q1, q0, 100
    push QWORD q1
    syscall
    hlt

.section data
message:
    db "Hello", 0x00"#;

    let tokens = lex(input);

    assert!(!tokens.is_empty());
    assert_eq!(tokens[0].kind, TokenKind::KwEntry);
    assert_eq!(tokens[1].kind, TokenKind::Identifier);
    assert_eq!(tokens[2].kind, TokenKind::KwSection);
    assert_eq!(tokens[3].kind, TokenKind::SectionName);

    let instruction_tokens: Vec<_> = tokens
        .iter()
        .filter(|t| {
            matches!(
                t.kind,
                TokenKind::KwMov
                    | TokenKind::KwAdd
                    | TokenKind::KwPush
                    | TokenKind::KwSyscall
                    | TokenKind::KwHlt
            )
        })
        .collect();
    assert_eq!(instruction_tokens.len(), 5);
}

#[test]
fn comments() {
    let tests = vec![
        (
            "mov q0, 42 ; this is a comment",
            vec![
                Token::new(TokenKind::KwMov, "mov", (0, 3)),
                Token::new(TokenKind::Register, "q0", (4, 6)),
                Token::new(TokenKind::Comma, ",", (6, 7)),
                Token::new(TokenKind::Integer, "42", (8, 10)),
            ],
        ),
        (
            "; full line comment\nmov q0, 1",
            vec![
                Token::new(TokenKind::KwMov, "mov", (20, 23)),
                Token::new(TokenKind::Register, "q0", (24, 26)),
                Token::new(TokenKind::Comma, ",", (26, 27)),
                Token::new(TokenKind::Integer, "1", (28, 29)),
            ],
        ),
        (
            "nop ; comment\nhlt ; another comment",
            vec![
                Token::new(TokenKind::KwNop, "nop", (0, 3)),
                Token::new(TokenKind::KwHlt, "hlt", (14, 17)),
            ],
        ),
    ];

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
        (
            r#""escaped quote: \"""#,
            vec![Token::new(
                TokenKind::String,
                r#"escaped quote: ""#,
                (0, 19),
            )],
        ),
        (
            r#""newline:\n tab:\t backslash:\\ quote:\"""#,
            vec![Token::new(
                TokenKind::String,
                "newline:\n tab:\t backslash:\\ quote:\"",
                (0, 41),
            )],
        ),
    ];

    for (input, expected) in tests {
        let tokens = lex(input);
        assert_eq!(tokens, expected);
    }
}
