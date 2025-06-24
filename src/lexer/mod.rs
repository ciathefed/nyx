use miette::NamedSource;

use crate::lexer::token::{Token, TokenKind, lookup_ident};

pub mod token;

#[cfg(test)]
mod tests;

pub struct Lexer {
    pub(crate) input: NamedSource<String>,
    pos: usize,
    read_pos: usize,
    ch: char,
}

impl Lexer {
    pub fn new(input: NamedSource<String>) -> Self {
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
            '\0' => Token::new(TokenKind::Eof, "", (start, self.read_pos)),
            ',' => Token::new(TokenKind::Comma, self.ch, (start, self.read_pos)),
            ':' => Token::new(TokenKind::Colon, self.ch, (start, self.read_pos)),
            '+' => Token::new(TokenKind::Plus, self.ch, (start, self.read_pos)),
            '-' => Token::new(TokenKind::Minus, self.ch, (start, self.read_pos)),
            '[' => Token::new(TokenKind::LBracket, self.ch, (start, self.read_pos)),
            ']' => Token::new(TokenKind::RBracket, self.ch, (start, self.read_pos)),
            '#' => self.read_directive(),
            '"' => self.read_string(),
            _ => {
                if self.ch.is_digit(10) {
                    return self.read_number();
                }

                if self.ch.is_alphabetic() || self.ch == '_' {
                    return self.read_identifier();
                }

                Token::new(TokenKind::Illegal, self.ch, (start, self.read_pos))
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

        while self.ch.is_digit(10) {
            self.read_char();
        }

        if self.ch == '.' && self.peek_char().is_digit(10) {
            self.read_char();
            while self.ch.is_digit(10) {
                self.read_char();
            }

            let literal = &self.input.inner()[start..self.pos];
            Token::new(TokenKind::Float, literal, (start, self.pos))
        } else {
            let literal = &self.input.inner()[start..self.pos];
            Token::new(TokenKind::Integer, literal, (start, self.pos))
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
        let start = self.pos + 1;
        loop {
            self.read_char();

            if self.ch == '"' || self.ch == '\0' {
                break;
            }
        }

        return Token::new(
            TokenKind::String,
            &self.input.inner()[start..self.pos],
            (start - 1, self.pos + 1),
        );
    }

    fn peek_char(&mut self) -> char {
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
