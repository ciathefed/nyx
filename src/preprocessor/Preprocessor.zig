const std = @import("std");
const fs = std.fs;
const fehler = @import("fehler");
const Allocator = std.mem.Allocator;
const ArrayList = std.array_list.Managed;
const Lexer = @import("../lexer/Lexer.zig");
const Parser = @import("../parser/Parser.zig");
const Span = @import("../Span.zig");
const ast = @import("../parser/ast.zig");
const utils = @import("../utils.zig");
const defaults = @import("defaults.zig");

const Preprocessor = @This();

const ConditionalType = enum {
    ifdef,
    ifndef,
};

const ConditionalInfo = struct {
    result: bool,
    seen_else: bool,
    type: ConditionalType,
    span: Span,
};

filename: []const u8,
input: []const u8,
program: []ast.Statement,
definitions: std.StringHashMap(*ast.Expression),
include_paths: ArrayList([]const u8),
included_files: std.StringHashMap(void),
reporter: *fehler.ErrorReporter,
arena: std.heap.ArenaAllocator,

pub fn init(
    filename: []const u8,
    input: []const u8,
    program: []ast.Statement,
    reporter: *fehler.ErrorReporter,
    include_paths: ?[][]const u8,
    allocator: Allocator,
) !Preprocessor {
    var default_definitions = try defaults.getDefaultDefinitons(allocator);
    defer default_definitions.deinit();
    var definitions = std.StringHashMap(*ast.Expression).init(allocator);
    errdefer definitions.deinit();

    var arena = std.heap.ArenaAllocator.init(allocator);

    var iter = default_definitions.iterator();
    while (iter.next()) |def| {
        const expr = try arena.allocator().create(ast.Expression);
        expr.* = def.value_ptr.*.*;
        try definitions.put(def.key_ptr.*, expr);
    }

    var cleanup_iter = default_definitions.iterator();
    while (cleanup_iter.next()) |def| {
        allocator.destroy(def.value_ptr.*);
    }

    return Preprocessor{
        .filename = filename,
        .input = input,
        .program = program,
        .definitions = definitions,
        .include_paths = if (include_paths) |paths|
            ArrayList([]const u8).fromOwnedSlice(allocator, paths)
        else
            ArrayList([]const u8).init(allocator),
        .included_files = std.StringHashMap(void).init(allocator),
        .reporter = reporter,
        .arena = arena,
    };
}

pub fn deinit(self: *Preprocessor) void {
    self.definitions.deinit();
    self.include_paths.deinit();
    self.included_files.deinit();
    self.arena.deinit();
}

pub fn process(self: *Preprocessor) ![]ast.Statement {
    const arena_alloc = self.arena.allocator();
    var processed_statements = try ArrayList(ast.Statement).initCapacity(arena_alloc, self.program.len);

    for (self.program) |stmt| {
        switch (stmt) {
            .define => |v| {
                const name = switch (v.expr1.*) {
                    .identifier => |ident| ident,
                    else => return self.reportError("invalid define key", v.span),
                };
                try self.definitions.put(name, v.expr2);
            },
            .include => |v| {
                const file_path = switch (v.expr.*) {
                    .string_literal => |str| str,
                    else => return self.reportError("invalid include path", v.span),
                };
                const included_statements = try self.processInclude(file_path, v.span);
                try processed_statements.appendSlice(included_statements);
            },
            else => try processed_statements.append(stmt),
        }
    }

    const conditional_statements = try self.processConditionals(try processed_statements.toOwnedSlice());
    defer arena_alloc.free(conditional_statements);

    var final_statements = try ArrayList(ast.Statement).initCapacity(arena_alloc, conditional_statements.len);

    for (conditional_statements) |stmt| {
        const new_stmt = try self.processStatement(stmt);
        if (new_stmt) |s| {
            try final_statements.append(s);
        }
    }

    return final_statements.toOwnedSlice();
}

fn processStatement(self: *Preprocessor, stmt: ast.Statement) !?ast.Statement {
    const arena_alloc = self.arena.allocator();

    return switch (stmt) {
        .label, .section, .nop, .ret, .syscall, .hlt => stmt,
        .@"error" => |v| switch (v.expr.*) {
            .string_literal => |message| return self.reportError(message, v.span),
            else => return self.reportError("expected string literal in #error directive", v.span),
        },
        .define => |v| .{ .define = .{
            .expr1 = try self.substituteExpr(v.expr1),
            .expr2 = try self.substituteExpr(v.expr2),
            .span = v.span,
        } },
        .include, .ifdef, .ifndef, .@"else", .endif => null,
        .entry => |v| .{ .entry = .{ .expr = try self.substituteExpr(v.expr), .span = v.span } },
        .ascii => |v| .{ .ascii = .{ .expr = try self.substituteExpr(v.expr), .span = v.span } },
        .asciz => |v| .{ .asciz = .{ .expr = try self.substituteExpr(v.expr), .span = v.span } },
        .jmp => |v| .{ .jmp = .{ .expr = try self.substituteExpr(v.expr), .span = v.span } },
        .jeq => |v| .{ .jeq = .{ .expr = try self.substituteExpr(v.expr), .span = v.span } },
        .jne => |v| .{ .jne = .{ .expr = try self.substituteExpr(v.expr), .span = v.span } },
        .jlt => |v| .{ .jlt = .{ .expr = try self.substituteExpr(v.expr), .span = v.span } },
        .jgt => |v| .{ .jgt = .{ .expr = try self.substituteExpr(v.expr), .span = v.span } },
        .jle => |v| .{ .jle = .{ .expr = try self.substituteExpr(v.expr), .span = v.span } },
        .jge => |v| .{ .jge = .{ .expr = try self.substituteExpr(v.expr), .span = v.span } },
        .call => |v| .{ .call = .{ .expr = try self.substituteExpr(v.expr), .span = v.span } },
        .inc => |v| .{ .inc = .{ .expr = try self.substituteExpr(v.expr), .span = v.span } },
        .dec => |v| .{ .dec = .{ .expr = try self.substituteExpr(v.expr), .span = v.span } },
        .neg => |v| .{ .neg = .{ .expr = try self.substituteExpr(v.expr), .span = v.span } },
        .resb => |v| .{ .resb = .{ .expr = try self.substituteExpr(v.expr), .span = v.span } },
        .mov => |v| .{ .mov = .{ .expr1 = try self.substituteExpr(v.expr1), .expr2 = try self.substituteExpr(v.expr2), .span = v.span } },
        .ldr => |v| .{ .ldr = .{ .expr1 = try self.substituteExpr(v.expr1), .expr2 = try self.substituteExpr(v.expr2), .span = v.span } },
        .str => |v| .{ .str = .{ .expr1 = try self.substituteExpr(v.expr1), .expr2 = try self.substituteExpr(v.expr2), .span = v.span } },
        .sti => |v| .{ .sti = .{
            .expr1 = try self.substituteExpr(v.expr1),
            .expr2 = try self.substituteExpr(v.expr2),
            .expr3 = try self.substituteExpr(v.expr3),
            .span = v.span,
        } },
        .cmp => |v| .{ .cmp = .{ .expr1 = try self.substituteExpr(v.expr1), .expr2 = try self.substituteExpr(v.expr2), .span = v.span } },
        .push => |v| .{ .push = .{
            .data_size = if (v.data_size) |size| try self.substituteExpr(size) else null,
            .expr = try self.substituteExpr(v.expr),
            .span = v.span,
        } },
        .pop => |v| .{ .pop = .{
            .data_size = if (v.data_size) |size| try self.substituteExpr(size) else null,
            .expr = try self.substituteExpr(v.expr),
            .span = v.span,
        } },
        .add => |v| .{ .add = .{ .expr1 = try self.substituteExpr(v.expr1), .expr2 = try self.substituteExpr(v.expr2), .expr3 = try self.substituteExpr(v.expr3), .span = v.span } },
        .sub => |v| .{ .sub = .{ .expr1 = try self.substituteExpr(v.expr1), .expr2 = try self.substituteExpr(v.expr2), .expr3 = try self.substituteExpr(v.expr3), .span = v.span } },
        .mul => |v| .{ .mul = .{ .expr1 = try self.substituteExpr(v.expr1), .expr2 = try self.substituteExpr(v.expr2), .expr3 = try self.substituteExpr(v.expr3), .span = v.span } },
        .div => |v| .{ .div = .{ .expr1 = try self.substituteExpr(v.expr1), .expr2 = try self.substituteExpr(v.expr2), .expr3 = try self.substituteExpr(v.expr3), .span = v.span } },
        .@"and" => |v| .{ .@"and" = .{ .expr1 = try self.substituteExpr(v.expr1), .expr2 = try self.substituteExpr(v.expr2), .expr3 = try self.substituteExpr(v.expr3), .span = v.span } },
        .@"or" => |v| .{ .@"or" = .{ .expr1 = try self.substituteExpr(v.expr1), .expr2 = try self.substituteExpr(v.expr2), .expr3 = try self.substituteExpr(v.expr3), .span = v.span } },
        .xor => |v| .{ .xor = .{ .expr1 = try self.substituteExpr(v.expr1), .expr2 = try self.substituteExpr(v.expr2), .expr3 = try self.substituteExpr(v.expr3), .span = v.span } },
        .shl => |v| .{ .shl = .{ .expr1 = try self.substituteExpr(v.expr1), .expr2 = try self.substituteExpr(v.expr2), .expr3 = try self.substituteExpr(v.expr3), .span = v.span } },
        .shr => |v| .{ .shr = .{ .expr1 = try self.substituteExpr(v.expr1), .expr2 = try self.substituteExpr(v.expr2), .expr3 = try self.substituteExpr(v.expr3), .span = v.span } },
        .db => |v| .{ .db = .{
            .exprs = blk: {
                var new_exprs = try ArrayList(*ast.Expression).initCapacity(arena_alloc, v.exprs.len);
                for (v.exprs) |expr| {
                    new_exprs.appendAssumeCapacity(try self.substituteExpr(expr));
                }
                break :blk try new_exprs.toOwnedSlice();
            },
            .span = v.span,
        } },
    };
}

fn processInclude(self: *Preprocessor, file_path: []const u8, span: Span) anyerror![]ast.Statement {
    const arena_alloc = self.arena.allocator();

    var found_path: ?[]const u8 = null;
    for (self.include_paths.items) |include_dir| {
        const candidate = try fs.path.join(arena_alloc, &.{ include_dir, file_path });
        if (!utils.fileExists(candidate)) continue;
        found_path = candidate;
        break;
    }

    const path = found_path orelse return self.reportError("include file not found", span);

    if (self.included_files.contains(path)) {
        return self.reportError("circular include", span);
    }

    const content = try utils.readFromFile(path, arena_alloc);
    try self.included_files.put(path, {});
    try self.reporter.addSource(path, content);

    const included_statements = try self.parseFileContent(content, path);

    var sub_preprocessor = Preprocessor{
        .filename = path,
        .input = content,
        .program = included_statements,
        .definitions = try self.definitions.clone(),
        .include_paths = try self.include_paths.clone(),
        .included_files = try self.included_files.clone(),
        .reporter = self.reporter,
        .arena = std.heap.ArenaAllocator.init(arena_alloc),
    };
    defer {
        sub_preprocessor.definitions.deinit();
        sub_preprocessor.include_paths.deinit();
        sub_preprocessor.included_files.deinit();
    }

    const processed = try sub_preprocessor.process();

    var definitions_iter = sub_preprocessor.definitions.iterator();
    while (definitions_iter.next()) |entry| {
        try self.definitions.put(entry.key_ptr.*, entry.value_ptr.*);
    }

    var included_files_iter = sub_preprocessor.included_files.iterator();
    while (included_files_iter.next()) |entry| {
        try self.included_files.put(entry.key_ptr.*, entry.value_ptr.*);
    }

    return processed;
}

fn parseFileContent(self: *Preprocessor, content: []const u8, path: []const u8) ![]ast.Statement {
    var lexer = Lexer.init(path, content, self.arena.allocator());
    var parser = Parser.init(&lexer, self.reporter, self.arena.allocator());
    return parser.parse();
}

fn processConditionals(self: *Preprocessor, statements: []ast.Statement) ![]ast.Statement {
    defer self.arena.allocator().free(statements);

    var result = try ArrayList(ast.Statement).initCapacity(self.arena.allocator(), statements.len);

    var stack = ArrayList(ConditionalInfo).init(self.arena.allocator());
    defer stack.deinit();

    for (statements) |stmt| {
        switch (stmt) {
            .ifdef => |v| {
                const condition_name = switch (v.expr.*) {
                    .identifier => |ident| ident,
                    else => {
                        self.report(.err, "expected identifier for condition name", v.span, 1);
                        return error.PreProcessorError;
                    },
                };

                const is_defined = self.definitions.contains(condition_name);
                try stack.append(.{
                    .result = is_defined,
                    .seen_else = false,
                    .type = .ifdef,
                    .span = v.span,
                });
            },
            .ifndef => |v| {
                const condition_name = switch (v.expr.*) {
                    .identifier => |ident| ident,
                    else => {
                        self.report(.err, "expected identifier for condition name", v.span, 1);
                        return error.PreProcessorError;
                    },
                };

                const is_defined = self.definitions.contains(condition_name);
                try stack.append(.{
                    .result = !is_defined,
                    .seen_else = false,
                    .type = .ifndef,
                    .span = v.span,
                });
            },
            .@"else" => |span| {
                if (stack.getLastOrNull()) |_| {
                    var info = &stack.items[stack.items.len - 1];
                    if (info.seen_else) {
                        self.report(.err, "unmatched else", span, 1);
                        return error.PreProcessorError;
                    }
                    info.seen_else = true;
                } else {
                    self.report(.err, "unmatched else", span, 1);
                    return error.PreProcessorError;
                }
            },
            .endif => |span| {
                if (stack.pop() == null) {
                    self.report(.err, "unmatched endif", span, 1);
                    return error.PreProcessorError;
                }
            },
            else => {
                if (shouldIncludeStatementWithInfo(stack.items)) {
                    result.appendAssumeCapacity(stmt);
                }
            },
        }
    }

    return result.toOwnedSlice();
}

fn substituteExpr(self: *Preprocessor, expr: *ast.Expression) anyerror!*ast.Expression {
    return switch (expr.*) {
        .identifier => |name| if (self.definitions.get(name)) |replacement|
            self.substituteExpr(replacement)
        else
            expr,
        .address => |v| blk: {
            const new_base = try self.substituteExpr(v.base);
            const new_offset = if (v.offset) |offset|
                try self.substituteExpr(offset)
            else
                null;
            break :blk try self.createExpr(.{ .address = .{ .base = new_base, .offset = new_offset } });
        },
        .register, .integer_literal, .float_literal, .string_literal, .data_size => expr,
        .binary_op => |v| try self.evaluateBinaryOp(v),
    };
}

fn evaluateBinaryOp(self: *Preprocessor, v: ast.Expression.BinaryOp) !*ast.Expression {
    const lhs = try self.substituteExpr(v.lhs);
    const rhs = try self.substituteExpr(v.rhs);

    if (lhs.* == .integer_literal and rhs.* == .integer_literal) {
        const l_val = lhs.integer_literal;
        const r_val = rhs.integer_literal;

        return self.createExpr(.{
            .integer_literal = switch (v.op) {
                .add => l_val + r_val,
                .sub => l_val - r_val,
                .mul => l_val * r_val,
                .div => @divTrunc(l_val, r_val),
                .bit_or => l_val | r_val,
                .bit_and => l_val & r_val,
                .bit_xor => l_val ^ r_val,
            },
        });
    }

    if (lhs.* == .float_literal and rhs.* == .float_literal) {
        const l_val = lhs.float_literal;
        const r_val = rhs.float_literal;

        const result = switch (v.op) {
            .add => l_val + r_val,
            .sub => l_val - r_val,
            .mul => l_val * r_val,
            .div => l_val / r_val,
            else => return self.reportError("invalid operator for float", v.span),
        };

        return self.createExpr(.{ .float_literal = result });
    }

    return self.createExpr(.{ .binary_op = .{ .lhs = lhs, .op = v.op, .rhs = rhs, .span = v.span } });
}

inline fn shouldIncludeStatementWithInfo(stack: []const ConditionalInfo) bool {
    for (stack) |info| {
        if (info.seen_else) {
            if (info.result) return false;
        } else {
            if (!info.result) return false;
        }
    }
    return true;
}

inline fn createExpr(self: *Preprocessor, expr: ast.Expression) !*ast.Expression {
    const new_expr = try self.arena.allocator().create(ast.Expression);
    new_expr.* = expr;
    return new_expr;
}

fn report(
    self: *Preprocessor,
    severity: fehler.Severity,
    message: []const u8,
    span: Span,
    status: ?u8,
) void {
    const source = self.reporter.sources.get(span.filename).?;
    self.reporter.report(.{
        .severity = severity,
        .message = message,
        .range = span.toSourceRange(source),
    });
    if (status) |code| {
        std.process.exit(code);
    }
}

fn reportError(self: *Preprocessor, message: []const u8, span: Span) error{PreProcessorError} {
    self.report(.err, message, span, 1);
    return error.PreProcessorError;
}
