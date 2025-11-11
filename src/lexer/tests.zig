const std = @import("std");
const testing = std.testing;
const ArrayList = std.array_list.Managed;
const StringInterner = @import("../StringInterner.zig");
const StringId = StringInterner.StringId;
const Token = @import("Token.zig");
const Lexer = @import("Lexer.zig");

const LexResult = struct {
    tokens: []Token,
    lexer: Lexer,
    interner: StringInterner,

    fn deinit(self: *LexResult, allocator: std.mem.Allocator) void {
        allocator.free(self.tokens);
        self.interner.deinit();
    }
};

fn lex(allocator: std.mem.Allocator, input: []const u8) !LexResult {
    var interner = StringInterner.init(allocator);
    errdefer interner.deinit();

    var tokens = ArrayList(Token).init(allocator);
    errdefer tokens.deinit();

    var lexer = Lexer.init("test.nyx", input, &interner, allocator);

    while (true) {
        const token = lexer.nextToken();
        try tokens.append(token);
        if (token.kind == .eof or token.kind == .illegal) break;
    }

    return LexResult{
        .tokens = try tokens.toOwnedSlice(),
        .lexer = lexer,
        .interner = interner,
    };
}

test "single character" {
    const input = ":,+-[]";
    var result = try lex(testing.allocator, input);
    defer result.deinit(testing.allocator);

    try testing.expectEqual(@as(usize, 7), result.tokens.len);
    try testing.expectEqual(Token.Kind.colon, result.tokens[0].kind);
    try testing.expectEqual(Token.Kind.comma, result.tokens[1].kind);
    try testing.expectEqual(Token.Kind.plus, result.tokens[2].kind);
    try testing.expectEqual(Token.Kind.minus, result.tokens[3].kind);
    try testing.expectEqual(Token.Kind.lbracket, result.tokens[4].kind);
    try testing.expectEqual(Token.Kind.rbracket, result.tokens[5].kind);
}

test "numbers" {
    const cases = [_]struct {
        input: []const u8,
        expected_kind: Token.Kind,
        expected_len: usize,
    }{
        .{ .input = "69", .expected_kind = .integer, .expected_len = 2 },
        .{ .input = "420", .expected_kind = .integer, .expected_len = 2 },
        .{ .input = "1337", .expected_kind = .integer, .expected_len = 2 },
    };

    for (cases) |case| {
        var result = try lex(testing.allocator, case.input);
        defer result.deinit(testing.allocator);

        try testing.expectEqual(case.expected_len, result.tokens.len);
        try testing.expectEqual(case.expected_kind, result.tokens[0].kind);
    }
}

test "hexadecimal numbers" {
    const cases = [_][]const u8{ "0x42", "0xFF", "0xDEADBEEF" };

    for (cases) |case| {
        var result = try lex(testing.allocator, case);
        defer result.deinit(testing.allocator);

        try testing.expectEqual(@as(usize, 2), result.tokens.len);
        try testing.expectEqual(Token.Kind.hexadecimal, result.tokens[0].kind);
    }
}

test "binary numbers" {
    const cases = [_][]const u8{ "0b0", "0b1010", "0B1101" };

    for (cases) |case| {
        var result = try lex(testing.allocator, case);
        defer result.deinit(testing.allocator);

        try testing.expectEqual(@as(usize, 2), result.tokens.len);
        try testing.expectEqual(Token.Kind.binary, result.tokens[0].kind);
    }
}

test "octal numbers" {
    const cases = [_][]const u8{ "0o0", "0o123", "0O777" };

    for (cases) |case| {
        var result = try lex(testing.allocator, case);
        defer result.deinit(testing.allocator);

        try testing.expectEqual(@as(usize, 2), result.tokens.len);
        try testing.expectEqual(Token.Kind.octal, result.tokens[0].kind);
    }
}

test "identifiers" {
    const cases = [_][]const u8{ "variable_name", "_long_long_long_12345_name" };

    for (cases) |case| {
        var result = try lex(testing.allocator, case);
        defer result.deinit(testing.allocator);

        try testing.expectEqual(@as(usize, 2), result.tokens.len);
        try testing.expectEqual(Token.Kind.identifier, result.tokens[0].kind);
    }
}

test "preprocessor directives" {
    const cases = [_]struct {
        input: []const u8,
        kind: Token.Kind,
    }{
        .{ .input = "#define", .kind = .kw_define },
        .{ .input = "#include", .kind = .kw_include },
        .{ .input = "#ifdef", .kind = .kw_ifdef },
        .{ .input = "#ifndef", .kind = .kw_ifndef },
        .{ .input = "#else", .kind = .kw_else },
        .{ .input = "#endif", .kind = .kw_endif },
        .{ .input = "#error", .kind = .kw_error },
    };

    for (cases) |case| {
        var result = try lex(testing.allocator, case.input);
        defer result.deinit(testing.allocator);

        try testing.expectEqual(@as(usize, 2), result.tokens.len);
        try testing.expectEqual(case.kind, result.tokens[0].kind);
    }
}

test "basic instructions" {
    const cases = [_]struct {
        input: []const u8,
        kind: Token.Kind,
    }{
        .{ .input = "nop", .kind = .kw_nop },
        .{ .input = "mov", .kind = .kw_mov },
        .{ .input = "ldr", .kind = .kw_ldr },
        .{ .input = "str", .kind = .kw_str },
        .{ .input = "push", .kind = .kw_push },
        .{ .input = "pop", .kind = .kw_pop },
        .{ .input = "cmp", .kind = .kw_cmp },
        .{ .input = "call", .kind = .kw_call },
        .{ .input = "ret", .kind = .kw_ret },
        .{ .input = "inc", .kind = .kw_inc },
        .{ .input = "dec", .kind = .kw_dec },
        .{ .input = "syscall", .kind = .kw_syscall },
        .{ .input = "hlt", .kind = .kw_hlt },
    };

    for (cases) |case| {
        var result = try lex(testing.allocator, case.input);
        defer result.deinit(testing.allocator);

        try testing.expectEqual(@as(usize, 2), result.tokens.len);
        try testing.expectEqual(case.kind, result.tokens[0].kind);
    }
}

test "arithmetic instructions" {
    const cases = [_]struct {
        input: []const u8,
        kind: Token.Kind,
    }{
        .{ .input = "add", .kind = .kw_add },
        .{ .input = "sub", .kind = .kw_sub },
        .{ .input = "mul", .kind = .kw_mul },
        .{ .input = "div", .kind = .kw_div },
    };

    for (cases) |case| {
        var result = try lex(testing.allocator, case.input);
        defer result.deinit(testing.allocator);

        try testing.expectEqual(@as(usize, 2), result.tokens.len);
        try testing.expectEqual(case.kind, result.tokens[0].kind);
    }
}

test "bitwise instructions" {
    const cases = [_]struct {
        input: []const u8,
        kind: Token.Kind,
    }{
        .{ .input = "and", .kind = .kw_and },
        .{ .input = "or", .kind = .kw_or },
        .{ .input = "xor", .kind = .kw_xor },
    };

    for (cases) |case| {
        var result = try lex(testing.allocator, case.input);
        defer result.deinit(testing.allocator);

        try testing.expectEqual(@as(usize, 2), result.tokens.len);
        try testing.expectEqual(case.kind, result.tokens[0].kind);
    }
}

test "shift instructions" {
    const cases = [_]struct {
        input: []const u8,
        kind: Token.Kind,
    }{
        .{ .input = "shl", .kind = .kw_shl },
        .{ .input = "shr", .kind = .kw_shr },
    };

    for (cases) |case| {
        var result = try lex(testing.allocator, case.input);
        defer result.deinit(testing.allocator);

        try testing.expectEqual(@as(usize, 2), result.tokens.len);
        try testing.expectEqual(case.kind, result.tokens[0].kind);
    }
}

test "jump instructions" {
    const cases = [_]struct {
        input: []const u8,
        kind: Token.Kind,
    }{
        .{ .input = "jmp", .kind = .kw_jmp },
        .{ .input = "jeq", .kind = .kw_jeq },
        .{ .input = "jne", .kind = .kw_jne },
        .{ .input = "jlt", .kind = .kw_jlt },
        .{ .input = "jgt", .kind = .kw_jgt },
        .{ .input = "jle", .kind = .kw_jle },
        .{ .input = "jge", .kind = .kw_jge },
    };

    for (cases) |case| {
        var result = try lex(testing.allocator, case.input);
        defer result.deinit(testing.allocator);

        try testing.expectEqual(@as(usize, 2), result.tokens.len);
        try testing.expectEqual(case.kind, result.tokens[0].kind);
    }
}

test "data declaration directives" {
    const cases = [_]struct {
        input: []const u8,
        kind: Token.Kind,
    }{
        .{ .input = "db", .kind = .kw_db },
        .{ .input = "resb", .kind = .kw_resb },
    };

    for (cases) |case| {
        var result = try lex(testing.allocator, case.input);
        defer result.deinit(testing.allocator);

        try testing.expectEqual(@as(usize, 2), result.tokens.len);
        try testing.expectEqual(case.kind, result.tokens[0].kind);
    }
}

test "float numbers" {
    const cases = [_][]const u8{ "3.14", "0.5", "123.456", "0.0", "999.999" };

    for (cases) |case| {
        var result = try lex(testing.allocator, case);
        defer result.deinit(testing.allocator);

        try testing.expectEqual(@as(usize, 2), result.tokens.len);
        try testing.expectEqual(Token.Kind.float, result.tokens[0].kind);
    }
}

test "mixed numbers" {
    const input1 = "42 3.14 0xFF";
    var result1 = try lex(testing.allocator, input1);
    defer result1.deinit(testing.allocator);

    try testing.expectEqual(@as(usize, 4), result1.tokens.len);
    try testing.expectEqual(Token.Kind.integer, result1.tokens[0].kind);
    try testing.expectEqual(Token.Kind.float, result1.tokens[1].kind);
    try testing.expectEqual(Token.Kind.hexadecimal, result1.tokens[2].kind);

    const input2 = "0b1010 420.69 0o777";
    var result2 = try lex(testing.allocator, input2);
    defer result2.deinit(testing.allocator);

    try testing.expectEqual(@as(usize, 4), result2.tokens.len);
    try testing.expectEqual(Token.Kind.binary, result2.tokens[0].kind);
    try testing.expectEqual(Token.Kind.float, result2.tokens[1].kind);
    try testing.expectEqual(Token.Kind.octal, result2.tokens[2].kind);
}

test "register tokens" {
    const input1 = "b0 w1 d2 q3";
    var result1 = try lex(testing.allocator, input1);
    defer result1.deinit(testing.allocator);

    try testing.expectEqual(@as(usize, 5), result1.tokens.len);
    for (result1.tokens[0..4]) |token| {
        try testing.expectEqual(Token.Kind.register, token.kind);
    }

    const input2 = "ff0 dd1 ip sp bp";
    var result2 = try lex(testing.allocator, input2);
    defer result2.deinit(testing.allocator);

    try testing.expectEqual(@as(usize, 6), result2.tokens.len);
    for (result2.tokens[0..5]) |token| {
        try testing.expectEqual(Token.Kind.register, token.kind);
    }
}

test "data size tokens" {
    const input1 = "byte word dword qword";
    var result1 = try lex(testing.allocator, input1);
    defer result1.deinit(testing.allocator);

    try testing.expectEqual(@as(usize, 5), result1.tokens.len);
    for (result1.tokens[0..4]) |token| {
        try testing.expectEqual(Token.Kind.data_size, token.kind);
    }

    const input2 = "float double";
    var result2 = try lex(testing.allocator, input2);
    defer result2.deinit(testing.allocator);

    try testing.expectEqual(@as(usize, 3), result2.tokens.len);
    for (result2.tokens[0..2]) |token| {
        try testing.expectEqual(Token.Kind.data_size, token.kind);
    }
}

test "section names" {
    const input = "text data";
    var result = try lex(testing.allocator, input);
    defer result.deinit(testing.allocator);

    try testing.expectEqual(@as(usize, 3), result.tokens.len);
    try testing.expectEqual(Token.Kind.section_name, result.tokens[0].kind);
    try testing.expectEqual(Token.Kind.section_name, result.tokens[1].kind);
}

test "complex program" {
    const input =
        \\.entry _start
        \\.section text
        \\_start:
        \\    mov q0, 42
        \\    add q1, q0, 100
        \\    push QWORD q1
        \\    syscall
        \\    hlt
        \\
        \\.section data
        \\message:
        \\    db "Hello", 0x00
    ;

    var result = try lex(testing.allocator, input);
    defer result.deinit(testing.allocator);

    try testing.expect(result.tokens.len > 0);
    try testing.expectEqual(Token.Kind.kw_entry, result.tokens[0].kind);
    try testing.expectEqual(Token.Kind.identifier, result.tokens[1].kind);
    try testing.expectEqual(Token.Kind.newline, result.tokens[2].kind);
    try testing.expectEqual(Token.Kind.kw_section, result.tokens[3].kind);
    try testing.expectEqual(Token.Kind.section_name, result.tokens[4].kind);
    try testing.expectEqual(Token.Kind.newline, result.tokens[5].kind);

    var instruction_count: usize = 0;
    for (result.tokens) |token| {
        if (token.kind == .kw_mov or token.kind == .kw_add or
            token.kind == .kw_push or token.kind == .kw_syscall or
            token.kind == .kw_hlt)
        {
            instruction_count += 1;
        }
    }
    try testing.expectEqual(@as(usize, 5), instruction_count);
}

test "comments" {
    const input1 = "mov q0, 42 ; this is a comment";
    var result1 = try lex(testing.allocator, input1);
    defer result1.deinit(testing.allocator);

    try testing.expectEqual(@as(usize, 5), result1.tokens.len);
    try testing.expectEqual(Token.Kind.kw_mov, result1.tokens[0].kind);
    try testing.expectEqual(Token.Kind.register, result1.tokens[1].kind);
    try testing.expectEqual(Token.Kind.comma, result1.tokens[2].kind);
    try testing.expectEqual(Token.Kind.integer, result1.tokens[3].kind);

    const input2 = "; full line comment\nmov q0, 1";
    var result2 = try lex(testing.allocator, input2);
    defer result2.deinit(testing.allocator);

    try testing.expectEqual(@as(usize, 6), result2.tokens.len);
    try testing.expectEqual(Token.Kind.kw_mov, result2.tokens[1].kind);

    const input3 = "nop ; comment\nhlt ; another comment";
    var result3 = try lex(testing.allocator, input3);
    defer result3.deinit(testing.allocator);

    try testing.expectEqual(@as(usize, 4), result3.tokens.len);
    try testing.expectEqual(Token.Kind.kw_nop, result3.tokens[0].kind);
    try testing.expectEqual(Token.Kind.kw_hlt, result3.tokens[2].kind);
}

test "strings" {
    const input1 = "\"this is a string!\"";
    var result1 = try lex(testing.allocator, input1);
    defer result1.deinit(testing.allocator);

    try testing.expectEqual(@as(usize, 2), result1.tokens.len);
    try testing.expectEqual(Token.Kind.string, result1.tokens[0].kind);
    try testing.expectEqualStrings("this is a string!", result1.interner.get(result1.tokens[0].string_id).?);

    const input2 = "\"this is a very very very very very long string!\"";
    var result2 = try lex(testing.allocator, input2);
    defer result2.deinit(testing.allocator);

    try testing.expectEqual(@as(usize, 2), result2.tokens.len);
    try testing.expectEqual(Token.Kind.string, result2.tokens[0].kind);

    const input3 = "\"escaped quote: \\\"\"";
    var result3 = try lex(testing.allocator, input3);
    defer result3.deinit(testing.allocator);

    try testing.expectEqual(@as(usize, 2), result3.tokens.len);
    try testing.expectEqual(Token.Kind.string, result3.tokens[0].kind);
    try testing.expectEqualStrings("escaped quote: \"", result3.interner.get(result3.tokens[0].string_id).?);

    const input4 = "\"newline:\\n tab:\\t backslash:\\\\ quote:\\\"\"";
    var result4 = try lex(testing.allocator, input4);
    defer result4.deinit(testing.allocator);

    try testing.expectEqual(@as(usize, 2), result4.tokens.len);
    try testing.expectEqual(Token.Kind.string, result4.tokens[0].kind);
    try testing.expectEqualStrings("newline:\n tab:\t backslash:\\ quote:\"", result4.interner.get(result4.tokens[0].string_id).?);
}
