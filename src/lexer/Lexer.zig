const std = @import("std");
const ascii = std.ascii;
const Allocator = std.mem.Allocator;
const ArrayList = std.array_list.Managed;
const Token = @import("Token.zig");
const Span = @import("../Span.zig");
const StringInterner = @import("../StringInterner.zig");

const Lexer = @This();

filename: []const u8,
input: []const u8,
pos: usize = 0,
read_pos: usize = 0,
ch: u8 = 0,
interner: *StringInterner,
allocator: Allocator,

pub fn init(filename: []const u8, input: []const u8, interner: *StringInterner, allocator: Allocator) Lexer {
    var lexer = Lexer{
        .filename = filename,
        .input = input,
        .interner = interner,
        .allocator = allocator,
    };
    lexer.readChar();
    return lexer;
}

pub fn nextToken(self: *Lexer) Token {
    const start = self.pos;

    if (self.ch == '\n') {
        self.readChar();
        return Token.init(.newline, "\n", .init(start, start, self.filename));
    }

    self.skipWhitespace();

    const token = switch (self.ch) {
        0 => Token.init(.eof, "", .init(start, start, self.filename)),
        ',' => Token.init(.comma, ",", .init(start, start, self.filename)),
        ':' => Token.init(.colon, ":", .init(start, start, self.filename)),
        '+' => Token.init(.plus, "+", .init(start, start, self.filename)),
        '-' => Token.init(.minus, "-", .init(start, start, self.filename)),
        '*' => Token.init(.asterisk, "*", .init(start, start, self.filename)),
        '/' => Token.init(.slash, "/", .init(start, start, self.filename)),
        '|' => Token.init(.pipe, "|", .init(start, start, self.filename)),
        '&' => Token.init(.ampersand, "&", .init(start, start, self.filename)),
        '^' => Token.init(.caret, "^", .init(start, start, self.filename)),
        '(' => Token.init(.lparen, "(", .init(start, start, self.filename)),
        ')' => Token.init(.rparen, ")", .init(start, start, self.filename)),
        '[' => Token.init(.lbracket, "[", .init(start, start, self.filename)),
        ']' => Token.init(.rbracket, "]", .init(start, start, self.filename)),
        '#' => return self.readDirective(),
        '.' => return self.readDirective(),
        '"' => return self.readString(),
        ';' => return self.skipComment(),
        else => {
            if (ascii.isDigit(self.ch)) return self.readNumber();
            if (ascii.isAlphabetic(self.ch) or self.ch == '_') return self.readIdentifier();
            return Token.init(.illegal, "", .init(start, start, self.filename));
        },
    };

    self.readChar();
    return token;
}

fn readChar(self: *Lexer) void {
    if (self.read_pos >= self.input.len) {
        self.ch = 0;
    } else {
        self.ch = self.input[self.read_pos];
    }
    self.pos = self.read_pos;
    self.read_pos += 1;
}

fn readNumber(self: *Lexer) Token {
    const start = self.pos;

    if (self.ch == '0') {
        switch (self.peekChar()) {
            'x', 'X' => {
                self.readChar();
                self.readChar();
                while (ascii.isHex(self.ch)) self.readChar();
                const literal = self.input[start..self.pos];
                return Token.init(.hexadecimal, literal, .init(start, self.pos - 1, self.filename));
            },
            'b', 'B' => {
                self.readChar();
                self.readChar();
                while (ascii.isHex(self.ch)) self.readChar();
                const literal = self.input[start..self.pos];
                return Token.init(.binary, literal, .init(start, self.pos - 1, self.filename));
            },
            'o', 'O' => {
                self.readChar();
                self.readChar();
                while (ascii.isHex(self.ch)) self.readChar();
                const literal = self.input[start..self.pos];
                return Token.init(.octal, literal, .init(start, self.pos - 1, self.filename));
            },
            else => {},
        }
    }

    while (ascii.isDigit(self.ch)) self.readChar();

    if (self.ch == '.' and ascii.isDigit(self.peekChar())) {
        self.readChar();
        while (ascii.isDigit(self.ch)) self.readChar();

        const literal = self.input[start..self.pos];
        return Token.init(.float, literal, .init(start, self.pos - 1, self.filename));
    } else {
        const literal = self.input[start..self.pos];
        return Token.init(.integer, literal, .init(start, self.pos - 1, self.filename));
    }
}

fn readIdentifier(self: *Lexer) Token {
    const start = self.pos;
    while (ascii.isAlphanumeric(self.ch) or self.ch == '_') {
        self.readChar();
    }

    const literal = self.input[start..self.pos];
    const kind = Token.lookupIdent(literal);

    if (kind == .identifier) {
        const id = self.interner.intern(literal) catch unreachable;
        return Token.initWithId(kind, id, .init(start, self.pos - 1, self.filename));
    }

    return Token.init(kind, literal, .init(start, self.pos - 1, self.filename));
}

fn readDirective(self: *Lexer) Token {
    const start = self.pos;
    self.readChar();
    while (ascii.isAlphanumeric(self.ch) or self.ch == '_') {
        self.readChar();
    }

    const literal = self.input[start..self.pos];
    const kind = Token.lookupIdent(literal);

    if (kind == .identifier) {
        const id = self.interner.intern(literal) catch unreachable;
        return Token.initWithId(kind, id, .init(start, self.pos - 1, self.filename));
    }

    return Token.init(kind, literal, .init(start, self.pos - 1, self.filename));
}

fn readString(self: *Lexer) Token {
    const start = self.pos;
    self.readChar();

    var result = ArrayList(u8).init(self.allocator);
    defer result.deinit();
    var escaped = false;

    while (true) {
        if (self.ch == 0) break;

        if (escaped) {
            switch (self.ch) {
                'n' => result.append('\n') catch unreachable,
                'r' => result.append('\r') catch unreachable,
                't' => result.append('\t') catch unreachable,
                '\\' => result.append('\\') catch unreachable,
                '"' => result.append('"') catch unreachable,
                else => |other| {
                    result.append('\\') catch unreachable;
                    result.append(other) catch unreachable;
                },
            }
            escaped = false;
        } else if (self.ch == '\\') {
            escaped = true;
        } else if (self.ch == '"') {
            break;
        } else {
            result.append(self.ch) catch unreachable;
        }

        self.readChar();
    }

    const end = self.read_pos - 1;
    if (self.ch == '"') self.readChar();

    const id = self.interner.intern(result.items) catch unreachable;

    return Token.initWithId(.string, id, .init(start, end, self.filename));
}

fn peekChar(self: *Lexer) u8 {
    return if (self.read_pos >= self.input.len)
        0
    else
        self.input[self.read_pos];
}

fn skipWhitespace(self: *Lexer) void {
    while (ascii.isWhitespace(self.ch)) {
        self.readChar();
    }
}

fn skipComment(self: *Lexer) Token {
    self.readChar();

    while (self.ch != '\n' and self.ch != 0) {
        self.readChar();
    }

    return self.nextToken();
}
