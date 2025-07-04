use std::sync::Arc;

use miette::NamedSource;

use crate::lexer::token::{Token, TokenKind, lookup_ident};

pub mod token;

#[cfg(test)]
mod tests;

pub struct Lexer {
    pub(crate) input: Arc<NamedSource<String>>,
    pos: usize,
    read_pos: usize,
    ch: char,
}

impl Lexer {
    pub fn new(input: Arc<NamedSource<String>>) -> Self {
        let mut lexer = Self {
            input,
            pos: 0,
            read_pos: 0,
            ch: '\0',
        };
        lexer.read_char();
        lexer
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let start = self.pos;

        let token = match self.ch {
            '\0' => Token::new_static(TokenKind::Eof, "", (start, self.read_pos)),
            ',' => Token::new_static(TokenKind::Comma, ",", (start, self.read_pos)),
            ':' => Token::new_static(TokenKind::Colon, ":", (start, self.read_pos)),
            '+' => Token::new_static(TokenKind::Plus, "+", (start, self.read_pos)),
            '-' => Token::new_static(TokenKind::Minus, "-", (start, self.read_pos)),
            '*' => Token::new_static(TokenKind::Asterisk, "*", (start, self.read_pos)),
            '/' => Token::new_static(TokenKind::Slash, "/", (start, self.read_pos)),
            '|' => Token::new_static(TokenKind::Pipe, "|", (start, self.read_pos)),
            '&' => Token::new_static(TokenKind::Ampersand, "&", (start, self.read_pos)),
            '^' => Token::new_static(TokenKind::Caret, "^", (start, self.read_pos)),
            '(' => Token::new_static(TokenKind::LParen, "(", (start, self.read_pos)),
            ')' => Token::new_static(TokenKind::RParen, ")", (start, self.read_pos)),
            '[' => Token::new_static(TokenKind::LBracket, "[", (start, self.read_pos)),
            ']' => Token::new_static(TokenKind::RBracket, "]", (start, self.read_pos)),
            '#' => return self.read_directive(),
            '.' => return self.read_directive(),
            '"' => return self.read_string(),
            ';' => return self.skip_comment(),
            _ => {
                if self.ch.is_ascii_digit() {
                    return self.read_number();
                }

                if self.ch.is_alphabetic() || self.ch == '_' {
                    return self.read_identifier();
                }

                Token::new_owned(
                    TokenKind::Illegal,
                    self.ch.to_string(),
                    (start, self.read_pos),
                )
            }
        };

        self.read_char();
        token
    }

    fn read_char(&mut self) {
        self.ch = self
            .input
            .inner()
            .chars()
            .nth(self.read_pos)
            .unwrap_or('\0');
        self.pos = self.read_pos;
        self.read_pos += 1;
    }

    fn read_number(&mut self) -> Token {
        let start = self.pos;

        if self.ch == '0' {
            match self.peek_char() {
                'x' | 'X' => {
                    self.read_char();
                    self.read_char();

                    while self.ch.is_ascii_hexdigit() {
                        self.read_char();
                    }

                    let literal = &self.input.inner()[start..self.pos];
                    Token::new(TokenKind::Hexadecimal, literal, (start, self.pos))
                }
                'b' | 'B' => {
                    self.read_char();
                    self.read_char();

                    while self.ch == '0' || self.ch == '1' {
                        self.read_char();
                    }

                    let literal = &self.input.inner()[start..self.pos];
                    Token::new(TokenKind::Binary, literal, (start, self.pos))
                }
                'o' | 'O' => {
                    self.read_char();
                    self.read_char();

                    while self.ch >= '0' && self.ch <= '7' {
                        self.read_char();
                    }

                    let literal = &self.input.inner()[start..self.pos];
                    Token::new(TokenKind::Octal, literal, (start, self.pos))
                }
                _ => {
                    while self.ch.is_ascii_digit() {
                        self.read_char();
                    }

                    if self.ch == '.' && self.peek_char().is_ascii_digit() {
                        self.read_char();
                        while self.ch.is_ascii_digit() {
                            self.read_char();
                        }

                        let literal = &self.input.inner()[start..self.pos];
                        Token::new(TokenKind::Float, literal, (start, self.pos))
                    } else {
                        let literal = &self.input.inner()[start..self.pos];
                        Token::new(TokenKind::Integer, literal, (start, self.pos))
                    }
                }
            }
        } else {
            while self.ch.is_ascii_digit() {
                self.read_char();
            }

            if self.ch == '.' && self.peek_char().is_ascii_digit() {
                self.read_char();
                while self.ch.is_ascii_digit() {
                    self.read_char();
                }

                let literal = &self.input.inner()[start..self.pos];
                Token::new(TokenKind::Float, literal, (start, self.pos))
            } else {
                let literal = &self.input.inner()[start..self.pos];
                Token::new(TokenKind::Integer, literal, (start, self.pos))
            }
        }
    }

    fn read_identifier(&mut self) -> Token {
        let start = self.pos;
        while self.ch.is_alphanumeric() || self.ch == '_' {
            self.read_char();
        }

        let literal = &self.input.inner()[start..self.pos];
        let kind = lookup_ident(literal);

        Token::new(kind, literal, (start, self.pos))
    }

    fn read_directive(&mut self) -> Token {
        let start = self.pos;
        self.read_char();
        while self.ch.is_alphanumeric() || self.ch == '_' {
            self.read_char();
        }

        let literal = &self.input.inner()[start..self.pos];
        let kind = lookup_ident(literal);

        Token::new(kind, literal, (start, self.pos))
    }

    fn read_string(&mut self) -> Token {
        let start = self.pos;
        self.read_char();

        let mut result = String::new();
        let mut escaped = false;

        loop {
            if self.ch == '\0' {
                break;
            }

            if escaped {
                match self.ch {
                    'n' => result.push('\n'),
                    'r' => result.push('\r'),
                    't' => result.push('\t'),
                    '\\' => result.push('\\'),
                    '"' => result.push('"'),
                    other => {
                        result.push('\\');
                        result.push(other);
                    }
                }
                escaped = false;
            } else if self.ch == '\\' {
                escaped = true;
            } else if self.ch == '"' {
                break;
            } else {
                result.push(self.ch);
            }

            self.read_char();
        }

        let end = self.read_pos;

        if self.ch == '"' {
            self.read_char();
        }

        Token::new(TokenKind::String, &result, (start, end))
    }

    fn peek_char(&self) -> char {
        self.input
            .inner()
            .chars()
            .nth(self.read_pos)
            .unwrap_or('\0')
    }

    fn skip_whitespace(&mut self) {
        while self.ch.is_whitespace() {
            self.read_char();
        }
    }

    fn skip_comment(&mut self) -> Token {
        self.read_char();

        while self.ch != '\n' && self.ch != '\0' {
            self.read_char();
        }

        self.next_token()
    }
}

impl Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.next_token();
        if token.kind == TokenKind::Eof {
            return None;
        }
        Some(token)
    }
}
