const std = @import("std");
const testing = std.testing;
const ArrayList = std.array_list.Managed;
const Token = @import("../lexer/Token.zig");
const Lexer = @import("../lexer/Lexer.zig");
const Parser = @import("Parser.zig");
const ast = @import("ast.zig");
const DataSize = @import("immediate.zig").DataSize;
const fehler = @import("fehler");

const ParseResult = struct {
    reporter: fehler.ErrorReporter,
    lexer: *Lexer,
    parser: *Parser,
    stmts: []ast.Statement,

    fn deinit(self: *ParseResult, allocator: std.mem.Allocator) void {
        self.parser.deinit();
        allocator.destroy(self.parser);
        self.lexer.deinit();
        allocator.destroy(self.lexer);
        self.reporter.deinit();
    }
};

fn parse(allocator: std.mem.Allocator, input: []const u8) !ParseResult {
    var reporter = fehler.ErrorReporter.init(allocator);
    try reporter.addSource("test.nyx", input);

    const lexer: *Lexer = try allocator.create(Lexer);
    lexer.* = .init("test.nyx", input, allocator);

    var parser: *Parser = try allocator.create(Parser);
    parser.* = .init(lexer, &reporter, allocator);

    return ParseResult{
        .reporter = reporter,
        .lexer = lexer,
        .parser = parser,
        .stmts = try parser.parse(),
    };
}

test "label" {
    const Test = struct {
        input: []const u8,
        name: []const u8,
        start: usize,
        end: usize,
    };

    const tests = [_]Test{
        .{ .input = "_start:", .name = "_start", .start = 0, .end = 6 },
        .{ .input = "very_very_very_very_long_label:", .name = "very_very_very_very_long_label", .start = 0, .end = 30 },
        .{ .input = "label_with_numbers_1337:", .name = "label_with_numbers_1337", .start = 0, .end = 23 },
    };

    for (tests) |t| {
        var res = try parse(testing.allocator, t.input);
        defer res.deinit(testing.allocator);

        try testing.expectEqual(@as(usize, 1), res.stmts.len);
        try testing.expect(res.stmts[0] == .label);
        try testing.expectEqualStrings(t.name, res.stmts[0].label.name);
        try testing.expectEqual(t.start, res.stmts[0].label.span.start);
        try testing.expectEqual(t.end, res.stmts[0].label.span.end);
        try testing.expectEqualStrings("test.nyx", res.stmts[0].label.span.filename);
    }
}

test "instructions" {
    const tests = [_]struct {
        input: []const u8,
        check: *const fn (ast.Statement) anyerror!void,
    }{
        .{
            .input = "nop",
            .check = struct {
                fn f(stmt: ast.Statement) !void {
                    try testing.expect(stmt == .nop);
                    try testing.expectEqual(@as(usize, 0), stmt.nop.start);
                    try testing.expectEqual(@as(usize, 2), stmt.nop.end);
                }
            }.f,
        },
        .{
            .input = "mov q0, 1337",
            .check = struct {
                fn f(stmt: ast.Statement) !void {
                    try testing.expect(stmt == .mov);
                    try testing.expect(stmt.mov.expr1.* == .register);
                    try testing.expect(stmt.mov.expr1.* == .register);
                    try testing.expect(stmt.mov.expr2.* == .integer_literal);
                    try testing.expectEqual(@as(i64, 1337), stmt.mov.expr2.integer_literal);
                }
            }.f,
        },
        .{
            .input = "ldr q0, [w0, 10]",
            .check = struct {
                fn f(stmt: ast.Statement) !void {
                    try testing.expect(stmt == .ldr);
                    try testing.expect(stmt.ldr.expr1.* == .register);
                    try testing.expect(stmt.ldr.expr2.* == .address);
                    try testing.expect(stmt.ldr.expr2.address.base.* == .register);
                    try testing.expect(stmt.ldr.expr2.address.offset != null);
                    try testing.expect(stmt.ldr.expr2.address.offset.?.* == .integer_literal);
                    try testing.expectEqual(@as(i64, 10), stmt.ldr.expr2.address.offset.?.integer_literal);
                }
            }.f,
        },
        .{
            .input = "str d0, [buffer]",
            .check = struct {
                fn f(stmt: ast.Statement) !void {
                    try testing.expect(stmt == .str);
                    try testing.expect(stmt.str.expr1.* == .register);
                    try testing.expect(stmt.str.expr2.* == .address);
                    try testing.expect(stmt.str.expr2.address.base.* == .identifier);
                    try testing.expectEqualStrings("buffer", stmt.str.expr2.address.base.identifier);
                    try testing.expect(stmt.str.expr2.address.offset == null);
                }
            }.f,
        },
        .{
            .input = "push q0",
            .check = struct {
                fn f(stmt: ast.Statement) !void {
                    try testing.expect(stmt == .push);
                    try testing.expect(stmt.push.data_size == null);
                    try testing.expect(stmt.push.expr.* == .register);
                }
            }.f,
        },
        .{
            .input = "pop float ff0",
            .check = struct {
                fn f(stmt: ast.Statement) !void {
                    try testing.expect(stmt == .pop);
                    try testing.expect(stmt.pop.data_size != null);
                    try testing.expect(stmt.pop.data_size.?.* == .data_size);
                    try testing.expectEqual(DataSize.float, stmt.pop.data_size.?.data_size);
                    try testing.expect(stmt.pop.expr.* == .register);
                }
            }.f,
        },
        .{
            .input = "cmp q0, 13",
            .check = struct {
                fn f(stmt: ast.Statement) !void {
                    try testing.expect(stmt == .cmp);
                    try testing.expect(stmt.cmp.expr1.* == .register);
                    try testing.expect(stmt.cmp.expr2.* == .integer_literal);
                    try testing.expectEqual(@as(i64, 13), stmt.cmp.expr2.integer_literal);
                }
            }.f,
        },
        .{
            .input = "call function_name",
            .check = struct {
                fn f(stmt: ast.Statement) !void {
                    try testing.expect(stmt == .call);
                    try testing.expect(stmt.call.expr.* == .identifier);
                    try testing.expectEqualStrings("function_name", stmt.call.expr.identifier);
                }
            }.f,
        },
        .{
            .input = "inc q0",
            .check = struct {
                fn f(stmt: ast.Statement) !void {
                    try testing.expect(stmt == .inc);
                    try testing.expect(stmt.inc.expr.* == .register);
                }
            }.f,
        },
        .{
            .input = "dec q0",
            .check = struct {
                fn f(stmt: ast.Statement) !void {
                    try testing.expect(stmt == .dec);
                    try testing.expect(stmt.dec.expr.* == .register);
                }
            }.f,
        },
        .{
            .input = "ret",
            .check = struct {
                fn f(stmt: ast.Statement) !void {
                    try testing.expect(stmt == .ret);
                }
            }.f,
        },
        .{
            .input = "hlt",
            .check = struct {
                fn f(stmt: ast.Statement) !void {
                    try testing.expect(stmt == .hlt);
                }
            }.f,
        },
    };

    for (tests) |t| {
        var res = try parse(testing.allocator, t.input);
        defer res.deinit(testing.allocator);
        try testing.expectEqual(@as(usize, 1), res.stmts.len);
        try t.check(res.stmts[0]);
    }
}

test "arithmetic operations" {
    const tests = [_]struct {
        input: []const u8,
        check: *const fn (ast.Statement) anyerror!void,
    }{
        .{
            .input = "add q0, q1, q2",
            .check = struct {
                fn f(stmt: ast.Statement) !void {
                    try testing.expect(stmt == .add);
                    try testing.expect(stmt.add.expr1.* == .register);
                    try testing.expect(stmt.add.expr2.* == .register);
                    try testing.expect(stmt.add.expr3.* == .register);
                }
            }.f,
        },
        .{
            .input = "sub d0, d1, 42",
            .check = struct {
                fn f(stmt: ast.Statement) !void {
                    try testing.expect(stmt == .sub);
                    try testing.expect(stmt.sub.expr1.* == .register);
                    try testing.expect(stmt.sub.expr2.* == .register);
                    try testing.expect(stmt.sub.expr3.* == .integer_literal);
                    try testing.expectEqual(@as(i64, 42), stmt.sub.expr3.integer_literal);
                }
            }.f,
        },
        .{
            .input = "mul w0, w1, w2",
            .check = struct {
                fn f(stmt: ast.Statement) !void {
                    try testing.expect(stmt == .mul);
                    try testing.expect(stmt.mul.expr1.* == .register);
                    try testing.expect(stmt.mul.expr2.* == .register);
                    try testing.expect(stmt.mul.expr3.* == .register);
                }
            }.f,
        },
        .{
            .input = "div b0, b1, 10",
            .check = struct {
                fn f(stmt: ast.Statement) !void {
                    try testing.expect(stmt == .div);
                    try testing.expect(stmt.div.expr1.* == .register);
                    try testing.expect(stmt.div.expr2.* == .register);
                    try testing.expect(stmt.div.expr3.* == .integer_literal);
                    try testing.expectEqual(@as(i64, 10), stmt.div.expr3.integer_literal);
                }
            }.f,
        },
    };

    for (tests) |t| {
        var res = try parse(testing.allocator, t.input);
        defer res.deinit(testing.allocator);
        try testing.expectEqual(@as(usize, 1), res.stmts.len);
        try t.check(res.stmts[0]);
    }
}

test "bitwise operations" {
    const tests = [_]struct {
        input: []const u8,
        check: *const fn (ast.Statement) anyerror!void,
    }{
        .{
            .input = "and q0, q1, q2",
            .check = struct {
                fn f(stmt: ast.Statement) !void {
                    try testing.expect(stmt == .@"and");
                    try testing.expect(stmt.@"and".expr1.* == .register);
                    try testing.expect(stmt.@"and".expr2.* == .register);
                    try testing.expect(stmt.@"and".expr3.* == .register);
                }
            }.f,
        },
        .{
            .input = "or d0, d1, 255",
            .check = struct {
                fn f(stmt: ast.Statement) !void {
                    try testing.expect(stmt == .@"or");
                    try testing.expect(stmt.@"or".expr1.* == .register);
                    try testing.expect(stmt.@"or".expr2.* == .register);
                    try testing.expect(stmt.@"or".expr3.* == .integer_literal);
                    try testing.expectEqual(@as(i64, 255), stmt.@"or".expr3.integer_literal);
                }
            }.f,
        },
        .{
            .input = "xor w0, w1, w2",
            .check = struct {
                fn f(stmt: ast.Statement) !void {
                    try testing.expect(stmt == .xor);
                    try testing.expect(stmt.xor.expr1.* == .register);
                    try testing.expect(stmt.xor.expr2.* == .register);
                    try testing.expect(stmt.xor.expr3.* == .register);
                }
            }.f,
        },
    };

    for (tests) |t| {
        var res = try parse(testing.allocator, t.input);
        defer res.deinit(testing.allocator);
        try testing.expectEqual(@as(usize, 1), res.stmts.len);
        try t.check(res.stmts[0]);
    }
}

test "shift operations" {
    const tests = [_]struct {
        input: []const u8,
        check: *const fn (ast.Statement) anyerror!void,
    }{
        .{
            .input = "shl b0, b1, 4",
            .check = struct {
                fn f(stmt: ast.Statement) !void {
                    try testing.expect(stmt == .shl);
                    try testing.expect(stmt.shl.expr1.* == .register);
                    try testing.expect(stmt.shl.expr2.* == .register);
                    try testing.expect(stmt.shl.expr3.* == .integer_literal);
                    try testing.expectEqual(@as(i64, 4), stmt.shl.expr3.integer_literal);
                }
            }.f,
        },
        .{
            .input = "shr q0, q1, q2",
            .check = struct {
                fn f(stmt: ast.Statement) !void {
                    try testing.expect(stmt == .shr);
                    try testing.expect(stmt.shr.expr1.* == .register);
                    try testing.expect(stmt.shr.expr2.* == .register);
                    try testing.expect(stmt.shr.expr3.* == .register);
                }
            }.f,
        },
    };

    for (tests) |t| {
        var res = try parse(testing.allocator, t.input);
        defer res.deinit(testing.allocator);
        try testing.expectEqual(@as(usize, 1), res.stmts.len);
        try t.check(res.stmts[0]);
    }
}

test "jump operations" {
    const tests = [_]struct {
        input: []const u8,
        check: *const fn (ast.Statement) anyerror!void,
    }{
        .{
            .input = "jmp 0x37",
            .check = struct {
                fn f(stmt: ast.Statement) !void {
                    try testing.expect(stmt == .jmp);
                    try testing.expect(stmt.jmp.expr.* == .integer_literal);
                    try testing.expectEqual(@as(i64, 0x37), stmt.jmp.expr.integer_literal);
                }
            }.f,
        },
        .{
            .input = "jne _exit",
            .check = struct {
                fn f(stmt: ast.Statement) !void {
                    try testing.expect(stmt == .jne);
                    try testing.expect(stmt.jne.expr.* == .identifier);
                    try testing.expectEqualStrings("_exit", stmt.jne.expr.identifier);
                }
            }.f,
        },
        .{
            .input = "jge q0",
            .check = struct {
                fn f(stmt: ast.Statement) !void {
                    try testing.expect(stmt == .jge);
                    try testing.expect(stmt.jge.expr.* == .register);
                }
            }.f,
        },
    };

    for (tests) |t| {
        var res = try parse(testing.allocator, t.input);
        defer res.deinit(testing.allocator);
        try testing.expectEqual(@as(usize, 1), res.stmts.len);
        try t.check(res.stmts[0]);
    }
}

test "expressions" {
    const tests = [_]struct {
        input: []const u8,
        check: *const fn (ast.Statement) anyerror!void,
    }{
        .{
            .input = "mov q0, 0xFF",
            .check = struct {
                fn f(stmt: ast.Statement) !void {
                    try testing.expect(stmt == .mov);
                    try testing.expect(stmt.mov.expr2.* == .integer_literal);
                    try testing.expectEqual(@as(i64, 255), stmt.mov.expr2.integer_literal);
                }
            }.f,
        },
        .{
            .input = "mov q0, 0b1010",
            .check = struct {
                fn f(stmt: ast.Statement) !void {
                    try testing.expect(stmt == .mov);
                    try testing.expect(stmt.mov.expr2.* == .integer_literal);
                    try testing.expectEqual(@as(i64, 10), stmt.mov.expr2.integer_literal);
                }
            }.f,
        },
        .{
            .input = "mov q0, 0o777",
            .check = struct {
                fn f(stmt: ast.Statement) !void {
                    try testing.expect(stmt == .mov);
                    try testing.expect(stmt.mov.expr2.* == .integer_literal);
                    try testing.expectEqual(@as(i64, 511), stmt.mov.expr2.integer_literal);
                }
            }.f,
        },
        .{
            .input = "mov ff0, 3.14",
            .check = struct {
                fn f(stmt: ast.Statement) !void {
                    try testing.expect(stmt == .mov);
                    try testing.expect(stmt.mov.expr2.* == .float_literal);
                    try testing.expectEqual(@as(f64, 3.14), stmt.mov.expr2.float_literal);
                }
            }.f,
        },
    };

    for (tests) |t| {
        var res = try parse(testing.allocator, t.input);
        defer res.deinit(testing.allocator);
        try testing.expectEqual(@as(usize, 1), res.stmts.len);
        try t.check(res.stmts[0]);
    }
}

test "addressing modes" {
    const tests = [_]struct {
        input: []const u8,
        check: *const fn (ast.Statement) anyerror!void,
    }{
        .{
            .input = "ldr q0, [q1]",
            .check = struct {
                fn f(stmt: ast.Statement) !void {
                    try testing.expect(stmt == .ldr);
                    try testing.expect(stmt.ldr.expr2.* == .address);
                    try testing.expect(stmt.ldr.expr2.address.base.* == .register);
                    try testing.expect(stmt.ldr.expr2.address.offset == null);
                }
            }.f,
        },
        .{
            .input = "str b0, [1000]",
            .check = struct {
                fn f(stmt: ast.Statement) !void {
                    try testing.expect(stmt == .str);
                    try testing.expect(stmt.str.expr2.* == .address);
                    try testing.expect(stmt.str.expr2.address.base.* == .integer_literal);
                    try testing.expectEqual(@as(i64, 1000), stmt.str.expr2.address.base.integer_literal);
                    try testing.expect(stmt.str.expr2.address.offset == null);
                }
            }.f,
        },
        .{
            .input = "ldr w0, [buffer, 16]",
            .check = struct {
                fn f(stmt: ast.Statement) !void {
                    try testing.expect(stmt == .ldr);
                    try testing.expect(stmt.ldr.expr2.* == .address);
                    try testing.expect(stmt.ldr.expr2.address.base.* == .identifier);
                    try testing.expectEqualStrings("buffer", stmt.ldr.expr2.address.base.identifier);
                    try testing.expect(stmt.ldr.expr2.address.offset != null);
                    try testing.expect(stmt.ldr.expr2.address.offset.?.* == .integer_literal);
                    try testing.expectEqual(@as(i64, 16), stmt.ldr.expr2.address.offset.?.integer_literal);
                }
            }.f,
        },
    };

    for (tests) |t| {
        var res = try parse(testing.allocator, t.input);
        defer res.deinit(testing.allocator);
        try testing.expectEqual(@as(usize, 1), res.stmts.len);
        try t.check(res.stmts[0]);
    }
}

test "data declarations" {
    const tests = [_]struct {
        input: []const u8,
        check: *const fn (ast.Statement) anyerror!void,
    }{
        .{
            .input = "db 42",
            .check = struct {
                fn f(stmt: ast.Statement) !void {
                    try testing.expect(stmt == .db);
                    try testing.expectEqual(@as(usize, 1), stmt.db.exprs.len);
                    try testing.expect(stmt.db.exprs[0].* == .integer_literal);
                    try testing.expectEqual(@as(i64, 42), stmt.db.exprs[0].integer_literal);
                }
            }.f,
        },
        .{
            .input = "db \"Hello\", 0x00",
            .check = struct {
                fn f(stmt: ast.Statement) !void {
                    try testing.expect(stmt == .db);
                    try testing.expectEqual(@as(usize, 2), stmt.db.exprs.len);
                    try testing.expect(stmt.db.exprs[0].* == .string_literal);
                    try testing.expectEqualStrings("Hello", stmt.db.exprs[0].string_literal);
                    try testing.expect(stmt.db.exprs[1].* == .integer_literal);
                    try testing.expectEqual(@as(i64, 0), stmt.db.exprs[1].integer_literal);
                }
            }.f,
        },
        .{
            .input = "db 1, 2, 3, 4",
            .check = struct {
                fn f(stmt: ast.Statement) !void {
                    try testing.expect(stmt == .db);
                    try testing.expectEqual(@as(usize, 4), stmt.db.exprs.len);
                    try testing.expectEqual(@as(i64, 1), stmt.db.exprs[0].integer_literal);
                    try testing.expectEqual(@as(i64, 2), stmt.db.exprs[1].integer_literal);
                    try testing.expectEqual(@as(i64, 3), stmt.db.exprs[2].integer_literal);
                    try testing.expectEqual(@as(i64, 4), stmt.db.exprs[3].integer_literal);
                }
            }.f,
        },
        .{
            .input = "resb 69",
            .check = struct {
                fn f(stmt: ast.Statement) !void {
                    try testing.expect(stmt == .resb);
                    try testing.expect(stmt.resb.expr.* == .integer_literal);
                    try testing.expectEqual(@as(i64, 69), stmt.resb.expr.integer_literal);
                }
            }.f,
        },
        .{
            .input = "resb 1024",
            .check = struct {
                fn f(stmt: ast.Statement) !void {
                    try testing.expect(stmt == .resb);
                    try testing.expect(stmt.resb.expr.* == .integer_literal);
                    try testing.expectEqual(@as(i64, 1024), stmt.resb.expr.integer_literal);
                }
            }.f,
        },
    };

    for (tests) |t| {
        var res = try parse(testing.allocator, t.input);
        defer res.deinit(testing.allocator);
        try testing.expectEqual(@as(usize, 1), res.stmts.len);
        try t.check(res.stmts[0]);
    }
}

test "sections" {
    const tests = [_]struct {
        input: []const u8,
        expected_type: ast.Statement.Section.Type,
    }{
        .{ .input = ".section text", .expected_type = .text },
        .{ .input = ".section data", .expected_type = .data },
    };

    for (tests) |t| {
        var res = try parse(testing.allocator, t.input);
        defer res.deinit(testing.allocator);
        try testing.expectEqual(@as(usize, 1), res.stmts.len);
        try testing.expect(res.stmts[0] == .section);
        try testing.expectEqual(t.expected_type, res.stmts[0].section.type);
    }
}

test "complex program" {
    const input =
        \\
        \\.entry _start
        \\.section text
        \\_start:
        \\    mov q0, 42
        \\    add q1, q0, 100
        \\    push qword q1
        \\    syscall
        \\    hlt
        \\
        \\.section data
        \\message:
        \\    .asciz "Hello, world!\n"
    ;

    var res = try parse(testing.allocator, input);
    defer res.deinit(testing.allocator);

    try testing.expectEqual(@as(usize, 11), res.stmts.len);

    try testing.expect(res.stmts[0] == .entry);
    try testing.expect(res.stmts[0].entry.expr.* == .identifier);

    try testing.expect(res.stmts[1] == .section);
    try testing.expectEqual(ast.Statement.Section.Type.text, res.stmts[1].section.type);

    try testing.expect(res.stmts[2] == .label);
    try testing.expectEqualStrings("_start", res.stmts[2].label.name);

    try testing.expect(res.stmts[3] == .mov);
    try testing.expect(res.stmts[4] == .add);
    try testing.expect(res.stmts[5] == .push);
    try testing.expect(res.stmts[6] == .syscall);
    try testing.expect(res.stmts[7] == .hlt);

    try testing.expect(res.stmts[8] == .section);
    try testing.expectEqual(ast.Statement.Section.Type.data, res.stmts[8].section.type);

    try testing.expect(res.stmts[9] == .label);
    try testing.expectEqualStrings("message", res.stmts[9].label.name);

    try testing.expect(res.stmts[10] == .asciz);
    try testing.expect(res.stmts[10].asciz.expr.* == .string_literal);
    try testing.expectEqualStrings("Hello, world!\n", res.stmts[10].asciz.expr.string_literal);
}
