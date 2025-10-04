const std = @import("std");
const ascii = std.ascii;
const Allocator = std.mem.Allocator;
const ArrayList = std.array_list.Managed;
const Token = @import("Token.zig");
const Span = @import("../Span.zig");

const Lexer = @This();

input: []const u8,
pos: usize = 0,
read_pos: usize = 0,
ch: u8 = 0,
strings: ArrayList([]const u8),
allocator: Allocator,

pub fn init(input: []const u8, allocator: Allocator) Lexer {
    var lexer = Lexer{
        .input = input,
        .strings = .init(allocator),
        .allocator = allocator,
    };
    lexer.readChar();
    return lexer;
}

pub fn deinit(self: *Lexer) void {
    for (self.strings.items) |string| {
        self.allocator.free(string);
    }
    self.strings.deinit();
}

pub fn nextToken(self: *Lexer) Token {
    self.skipWhitespace();

    const start = self.pos;

    const token = switch (self.ch) {
        0 => Token.init(.eof, "", .{ .start = start, .end = start }),
        ',' => Token.init(.comma, ",", .{ .start = start, .end = start }),
        ':' => Token.init(.colon, ":", .{ .start = start, .end = start }),
        '+' => Token.init(.plus, "+", .{ .start = start, .end = start }),
        '-' => Token.init(.minus, "-", .{ .start = start, .end = start }),
        '*' => Token.init(.asterisk, "*", .{ .start = start, .end = start }),
        '/' => Token.init(.slash, "/", .{ .start = start, .end = start }),
        '|' => Token.init(.pipe, "|", .{ .start = start, .end = start }),
        '&' => Token.init(.ampersand, "&", .{ .start = start, .end = start }),
        '^' => Token.init(.caret, "^", .{ .start = start, .end = start }),
        '(' => Token.init(.lparen, "(", .{ .start = start, .end = start }),
        ')' => Token.init(.rparen, ")", .{ .start = start, .end = start }),
        '[' => Token.init(.lbracket, "[", .{ .start = start, .end = start }),
        ']' => Token.init(.rbracket, "]", .{ .start = start, .end = start }),
        '#' => return self.readDirective(),
        '.' => return self.readDirective(),
        '"' => return self.readString(),
        ';' => return self.skipComment(),
        else => {
            if (ascii.isDigit(self.ch)) return self.readNumber();
            if (ascii.isAlphabetic(self.ch) or self.ch == '_') return self.readIdentifier();
            return Token.init(.illegal, "", .{ .start = start, .end = start });
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
                return Token.init(.hexadecimal, literal, .{ .start = start, .end = self.pos - 1 });
            },
            'b', 'B' => {
                self.readChar();
                self.readChar();
                while (ascii.isHex(self.ch)) self.readChar();
                const literal = self.input[start..self.pos];
                return Token.init(.binary, literal, .{ .start = start, .end = self.pos - 1 });
            },
            'o', 'O' => {
                self.readChar();
                self.readChar();
                while (ascii.isHex(self.ch)) self.readChar();
                const literal = self.input[start..self.pos];
                return Token.init(.octal, literal, .{ .start = start, .end = self.pos - 1 });
            },
            else => {},
        }
    }

    while (ascii.isDigit(self.ch)) self.readChar();

    if (self.ch == '.' and ascii.isDigit(self.peekChar())) {
        self.readChar();
        while (ascii.isDigit(self.ch)) self.readChar();

        const literal = self.input[start..self.pos];
        return Token.init(.float, literal, .{ .start = start, .end = self.pos - 1 });
    } else {
        const literal = self.input[start..self.pos];
        return Token.init(.integer, literal, .{ .start = start, .end = self.pos - 1 });
    }
}

fn readIdentifier(self: *Lexer) Token {
    const start = self.pos;
    while (ascii.isAlphanumeric(self.ch) or self.ch == '_') {
        self.readChar();
    }

    const literal = self.input[start..self.pos];
    const kind = Token.lookupIdent(literal);

    return Token.init(kind, literal, .{ .start = start, .end = self.pos - 1 });
}

fn readDirective(self: *Lexer) Token {
    const start = self.pos;
    self.readChar();
    while (ascii.isAlphanumeric(self.ch) or self.ch == '_') {
        self.readChar();
    }

    const literal = self.input[start..self.pos];
    const kind = Token.lookupIdent(literal);

    return Token.init(kind, literal, .{ .start = start, .end = self.pos - 1 });
}

fn readString(self: *Lexer) Token {
    const start = self.pos;
    self.readChar();

    var result = ArrayList(u8).init(self.allocator);
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

    const literal = result.toOwnedSlice() catch unreachable;
    self.strings.append(literal) catch unreachable;

    return Token.init(.string, literal, .{ .start = start, .end = end });
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
