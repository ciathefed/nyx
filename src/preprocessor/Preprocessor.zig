const std = @import("std");
const fs = std.fs;
const fehler = @import("fehler");
const Allocator = std.mem.Allocator;
const ArrayList = std.array_list.Managed;
const StringInterner = @import("../StringInterner.zig");
const StringId = StringInterner.StringId;
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

const MacroInfo = struct {
    params: []StringId,
    body: []ast.Statement,
    span: Span,
};

io: std.Io,
filename: []const u8,
input: []const u8,
program: []ast.Statement,
interner: *StringInterner,
definitions: std.AutoHashMap(StringId, ?*ast.Expression),
macros: std.AutoHashMap(StringId, MacroInfo),
include_paths: ArrayList([]const u8),
reporter: *fehler.ErrorReporter,
arena: std.heap.ArenaAllocator,

pub fn init(
    io: std.Io,
    gpa: Allocator,
    filename: []const u8,
    input: []const u8,
    program: []ast.Statement,
    interner: *StringInterner,
    reporter: *fehler.ErrorReporter,
    include_paths: ?[][]const u8,
) !Preprocessor {
    var default_definitions = try defaults.getDefaultDefinitions(gpa, interner);
    defer default_definitions.deinit();

    var definitions = std.AutoHashMap(StringId, ?*ast.Expression).init(gpa);
    errdefer definitions.deinit();

    var arena = std.heap.ArenaAllocator.init(gpa);

    var iter = default_definitions.iterator();
    while (iter.next()) |def| {
        const expr = try arena.allocator().create(ast.Expression);
        expr.* = def.value_ptr.*.*;
        try definitions.put(def.key_ptr.*, expr);
    }

    var cleanup_iter = default_definitions.iterator();
    while (cleanup_iter.next()) |def| {
        gpa.destroy(def.value_ptr.*);
    }

    return Preprocessor{
        .io = io,
        .filename = filename,
        .input = input,
        .program = program,
        .interner = interner,
        .definitions = definitions,
        .macros = std.AutoHashMap(StringId, MacroInfo).init(gpa),
        .include_paths = if (include_paths) |paths|
            ArrayList([]const u8).fromOwnedSlice(gpa, paths)
        else
            ArrayList([]const u8).init(gpa),
        .reporter = reporter,
        .arena = arena,
    };
}

pub fn deinit(self: *Preprocessor) void {
    self.definitions.deinit();
    self.macros.deinit();
    self.include_paths.deinit();
    self.arena.deinit();
}

pub fn process(self: *Preprocessor) ![]ast.Statement {
    const arena_alloc = self.arena.allocator();

    var processed_statements = try ArrayList(ast.Statement).initCapacity(arena_alloc, self.program.len);

    for (self.program) |stmt| {
        switch (stmt) {
            .include => |v| {
                const file_path_id = switch (v.expr.*) {
                    .string_literal => |str_id| str_id,
                    else => return self.reportError("invalid include path", v.span),
                };
                const file_path = self.interner.get(file_path_id) orelse
                    return self.reportError("invalid include path", v.span);
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
        switch (stmt) {
            .define => |v| {
                const name_id = switch (v.name.*) {
                    .identifier => |ident_id| ident_id,
                    else => return self.reportError("invalid define key", v.span),
                };
                try self.definitions.put(name_id, v.expr);
            },
            .macro_def => |v| {
                try self.macros.put(v.name, .{
                    .params = v.params,
                    .body = v.body,
                    .span = v.span,
                });
            },
            .macro_call => |v| {
                const expanded = try self.expandMacro(v);
                try final_statements.appendSlice(expanded);
            },
            else => {
                const new_stmt = try self.processStatement(stmt);
                if (new_stmt) |s| {
                    try final_statements.append(s);
                }
            },
        }
    }

    return final_statements.toOwnedSlice();
}

fn expandMacro(self: *Preprocessor, call: ast.Statement.MacroCall) ![]ast.Statement {
    const arena_alloc = self.arena.allocator();

    const macro_info = self.macros.get(call.name) orelse {
        const name_str = self.interner.get(call.name) orelse "<unknown>";
        const msg = try std.fmt.allocPrint(arena_alloc, "undefined macro: {s}", .{name_str});
        return self.reportError(msg, call.span);
    };

    if (call.args.len != macro_info.params.len) {
        const name_str = self.interner.get(call.name) orelse "<unknown>";
        const msg = try std.fmt.allocPrint(
            arena_alloc,
            "macro '{s}' expects {d} arguments, got {d}",
            .{ name_str, macro_info.params.len, call.args.len },
        );
        return self.reportError(msg, call.span);
    }

    var param_map = std.AutoHashMap(StringId, *ast.Expression).init(arena_alloc);
    defer param_map.deinit();

    for (macro_info.params, 0..) |param_id, i| {
        const substituted_arg = try self.substituteExpr(call.args[i]);
        try param_map.put(param_id, substituted_arg);
    }

    var expanded = try ArrayList(ast.Statement).initCapacity(arena_alloc, macro_info.body.len);

    for (macro_info.body) |body_stmt| {
        const substituted_stmt = try self.substituteStatement(body_stmt, &param_map);
        if (substituted_stmt) |s| {
            const processed = try self.processStatement(s);
            if (processed) |p| {
                try expanded.append(p);
            }
        }
    }

    return expanded.toOwnedSlice();
}

fn substituteStatement(self: *Preprocessor, stmt: ast.Statement, param_map: *std.AutoHashMap(StringId, *ast.Expression)) !?ast.Statement {
    const arena_alloc = self.arena.allocator();

    return switch (stmt) {
        .label, .section, .nop, .ret, .syscall, .hlt, .@"else", .endif => stmt,
        .@"error" => |v| .{ .@"error" = .{ .expr = try self.substituteExprWithParams(v.expr, param_map), .span = v.span } },
        .define => |v| .{ .define = .{
            .name = try self.substituteExprWithParams(v.name, param_map),
            .expr = if (v.expr) |expr| try self.substituteExprWithParams(expr, param_map) else null,
            .span = v.span,
        } },
        .include, .ifdef, .ifndef => null,
        .entry => |v| .{ .entry = .{ .expr = try self.substituteExprWithParams(v.expr, param_map), .span = v.span } },
        .ascii => |v| .{ .ascii = .{ .expr = try self.substituteExprWithParams(v.expr, param_map), .span = v.span } },
        .asciz => |v| .{ .asciz = .{ .expr = try self.substituteExprWithParams(v.expr, param_map), .span = v.span } },
        .@"extern" => |v| .{ .@"extern" = .{ .name = try self.substituteExprWithParams(v.name, param_map), .param_types = v.param_types, .return_type = v.return_type, .is_variadic = v.is_variadic, .span = v.span } },
        .jmp => |v| .{ .jmp = .{ .expr = try self.substituteExprWithParams(v.expr, param_map), .span = v.span } },
        .jeq => |v| .{ .jeq = .{ .expr = try self.substituteExprWithParams(v.expr, param_map), .span = v.span } },
        .jne => |v| .{ .jne = .{ .expr = try self.substituteExprWithParams(v.expr, param_map), .span = v.span } },
        .jlt => |v| .{ .jlt = .{ .expr = try self.substituteExprWithParams(v.expr, param_map), .span = v.span } },
        .jgt => |v| .{ .jgt = .{ .expr = try self.substituteExprWithParams(v.expr, param_map), .span = v.span } },
        .jle => |v| .{ .jle = .{ .expr = try self.substituteExprWithParams(v.expr, param_map), .span = v.span } },
        .jge => |v| .{ .jge = .{ .expr = try self.substituteExprWithParams(v.expr, param_map), .span = v.span } },
        .call => |v| .{ .call = .{ .expr = try self.substituteExprWithParams(v.expr, param_map), .span = v.span } },
        .call_variadic => |v| .{ .call_variadic = .{ .name = try self.substituteExprWithParams(v.name, param_map), .variadic_types = v.variadic_types, .span = v.span } },
        .inc => |v| .{ .inc = .{ .expr = try self.substituteExprWithParams(v.expr, param_map), .span = v.span } },
        .dec => |v| .{ .dec = .{ .expr = try self.substituteExprWithParams(v.expr, param_map), .span = v.span } },
        .neg => |v| .{ .neg = .{ .expr = try self.substituteExprWithParams(v.expr, param_map), .span = v.span } },
        .mov => |v| .{ .mov = .{
            .data_size = if (v.data_size) |size| try self.substituteExprWithParams(size, param_map) else null,
            .expr1 = try self.substituteExprWithParams(v.expr1, param_map),
            .expr2 = try self.substituteExprWithParams(v.expr2, param_map),
            .span = v.span,
        } },
        .cmp => |v| .{ .cmp = .{ .expr1 = try self.substituteExprWithParams(v.expr1, param_map), .expr2 = try self.substituteExprWithParams(v.expr2, param_map), .span = v.span } },
        .push => |v| .{ .push = .{
            .data_size = if (v.data_size) |size| try self.substituteExprWithParams(size, param_map) else null,
            .expr = try self.substituteExprWithParams(v.expr, param_map),
            .span = v.span,
        } },
        .pop => |v| .{ .pop = .{
            .data_size = if (v.data_size) |size| try self.substituteExprWithParams(size, param_map) else null,
            .expr = try self.substituteExprWithParams(v.expr, param_map),
            .span = v.span,
        } },
        .add => |v| .{ .add = .{ .expr1 = try self.substituteExprWithParams(v.expr1, param_map), .expr2 = try self.substituteExprWithParams(v.expr2, param_map), .expr3 = try self.substituteExprWithParams(v.expr3, param_map), .span = v.span } },
        .sub => |v| .{ .sub = .{ .expr1 = try self.substituteExprWithParams(v.expr1, param_map), .expr2 = try self.substituteExprWithParams(v.expr2, param_map), .expr3 = try self.substituteExprWithParams(v.expr3, param_map), .span = v.span } },
        .mul => |v| .{ .mul = .{ .expr1 = try self.substituteExprWithParams(v.expr1, param_map), .expr2 = try self.substituteExprWithParams(v.expr2, param_map), .expr3 = try self.substituteExprWithParams(v.expr3, param_map), .span = v.span } },
        .div => |v| .{ .div = .{ .expr1 = try self.substituteExprWithParams(v.expr1, param_map), .expr2 = try self.substituteExprWithParams(v.expr2, param_map), .expr3 = try self.substituteExprWithParams(v.expr3, param_map), .span = v.span } },
        .@"and" => |v| .{ .@"and" = .{ .expr1 = try self.substituteExprWithParams(v.expr1, param_map), .expr2 = try self.substituteExprWithParams(v.expr2, param_map), .expr3 = try self.substituteExprWithParams(v.expr3, param_map), .span = v.span } },
        .@"or" => |v| .{ .@"or" = .{ .expr1 = try self.substituteExprWithParams(v.expr1, param_map), .expr2 = try self.substituteExprWithParams(v.expr2, param_map), .expr3 = try self.substituteExprWithParams(v.expr3, param_map), .span = v.span } },
        .xor => |v| .{ .xor = .{ .expr1 = try self.substituteExprWithParams(v.expr1, param_map), .expr2 = try self.substituteExprWithParams(v.expr2, param_map), .expr3 = try self.substituteExprWithParams(v.expr3, param_map), .span = v.span } },
        .shl => |v| .{ .shl = .{ .expr1 = try self.substituteExprWithParams(v.expr1, param_map), .expr2 = try self.substituteExprWithParams(v.expr2, param_map), .expr3 = try self.substituteExprWithParams(v.expr3, param_map), .span = v.span } },
        .shr => |v| .{ .shr = .{ .expr1 = try self.substituteExprWithParams(v.expr1, param_map), .expr2 = try self.substituteExprWithParams(v.expr2, param_map), .expr3 = try self.substituteExprWithParams(v.expr3, param_map), .span = v.span } },
        .rol => |v| .{ .rol = .{ .expr1 = try self.substituteExprWithParams(v.expr1, param_map), .expr2 = try self.substituteExprWithParams(v.expr2, param_map), .expr3 = try self.substituteExprWithParams(v.expr3, param_map), .span = v.span } },
        .ror => |v| .{ .ror = .{ .expr1 = try self.substituteExprWithParams(v.expr1, param_map), .expr2 = try self.substituteExprWithParams(v.expr2, param_map), .expr3 = try self.substituteExprWithParams(v.expr3, param_map), .span = v.span } },
        .db => |v| .{ .db = .{
            .exprs = blk: {
                var new_exprs = try ArrayList(*ast.Expression).initCapacity(arena_alloc, v.exprs.len);
                for (v.exprs) |expr| {
                    new_exprs.appendAssumeCapacity(try self.substituteExprWithParams(expr, param_map));
                }
                break :blk try new_exprs.toOwnedSlice();
            },
            .span = v.span,
        } },
        .dw => |v| .{ .dw = .{
            .exprs = blk: {
                var new_exprs = try ArrayList(*ast.Expression).initCapacity(arena_alloc, v.exprs.len);
                for (v.exprs) |expr| {
                    new_exprs.appendAssumeCapacity(try self.substituteExprWithParams(expr, param_map));
                }
                break :blk try new_exprs.toOwnedSlice();
            },
            .span = v.span,
        } },
        .dd => |v| .{ .dd = .{
            .exprs = blk: {
                var new_exprs = try ArrayList(*ast.Expression).initCapacity(arena_alloc, v.exprs.len);
                for (v.exprs) |expr| {
                    new_exprs.appendAssumeCapacity(try self.substituteExprWithParams(expr, param_map));
                }
                break :blk try new_exprs.toOwnedSlice();
            },
            .span = v.span,
        } },
        .dq => |v| .{ .dq = .{
            .exprs = blk: {
                var new_exprs = try ArrayList(*ast.Expression).initCapacity(arena_alloc, v.exprs.len);
                for (v.exprs) |expr| {
                    new_exprs.appendAssumeCapacity(try self.substituteExprWithParams(expr, param_map));
                }
                break :blk try new_exprs.toOwnedSlice();
            },
            .span = v.span,
        } },
        .resb => |v| .{ .resb = .{ .expr = try self.substituteExprWithParams(v.expr, param_map), .span = v.span } },
        .resw => |v| .{ .resw = .{ .expr = try self.substituteExprWithParams(v.expr, param_map), .span = v.span } },
        .resd => |v| .{ .resd = .{ .expr = try self.substituteExprWithParams(v.expr, param_map), .span = v.span } },
        .resq => |v| .{ .resq = .{ .expr = try self.substituteExprWithParams(v.expr, param_map), .span = v.span } },
        .macro_def => null, // macro definitions inside macro bodies are ignored
        .macro_call => null, // nested macro calls inside expansion not supported
    };
}

fn substituteExprWithParams(self: *Preprocessor, expr: *ast.Expression, param_map: *std.AutoHashMap(StringId, *ast.Expression)) anyerror!*ast.Expression {
    return switch (expr.*) {
        .identifier => |name_id| blk: {
            if (param_map.get(name_id)) |replacement| {
                break :blk replacement;
            }
            if (self.definitions.get(name_id)) |replacement| {
                if (replacement) |r| {
                    break :blk self.substituteExprWithParams(r, param_map);
                }
            }
            break :blk expr;
        },
        .address => |v| blk: {
            const new_base = try self.substituteExprWithParams(v.base, param_map);
            const new_offset = if (v.offset) |offset|
                try self.substituteExprWithParams(offset, param_map)
            else
                null;
            break :blk try self.createExpr(.{ .address = .{ .base = new_base, .offset = new_offset } });
        },
        .register, .integer_literal, .float_literal, .string_literal, .data_size => expr,
        .unary_op => |v| blk: {
            const inner = try self.substituteExprWithParams(v.expr, param_map);
            break :blk try self.createExpr(.{ .unary_op = .{ .op = v.op, .expr = inner, .span = v.span } });
        },
        .binary_op => |v| blk: {
            const lhs = try self.substituteExprWithParams(v.lhs, param_map);
            const rhs = try self.substituteExprWithParams(v.rhs, param_map);
            break :blk try self.createExpr(.{ .binary_op = .{ .lhs = lhs, .op = v.op, .rhs = rhs, .span = v.span } });
        },
    };
}

fn processStatement(self: *Preprocessor, stmt: ast.Statement) !?ast.Statement {
    const arena_alloc = self.arena.allocator();

    return switch (stmt) {
        .label, .section, .nop, .ret, .syscall, .hlt => stmt,
        .@"error" => |v| switch (v.expr.*) {
            .string_literal => |message_id| {
                const message = self.interner.get(message_id) orelse
                    return self.reportError("invalid error message", v.span);
                return self.reportError(message, v.span);
            },
            else => return self.reportError("expected string literal in #error directive", v.span),
        },
        .define => |v| .{ .define = .{
            .name = try self.substituteExpr(v.name),
            .expr = if (v.expr) |expr| try self.substituteExpr(expr) else null,
            .span = v.span,
        } },
        .include, .ifdef, .ifndef, .@"else", .endif => null,
        .entry => |v| .{ .entry = .{ .expr = try self.substituteExpr(v.expr), .span = v.span } },
        .ascii => |v| .{ .ascii = .{ .expr = try self.substituteExpr(v.expr), .span = v.span } },
        .asciz => |v| .{ .asciz = .{ .expr = try self.substituteExpr(v.expr), .span = v.span } },
        .@"extern" => |v| .{ .@"extern" = .{ .name = try self.substituteExpr(v.name), .param_types = v.param_types, .return_type = v.return_type, .is_variadic = v.is_variadic, .span = v.span } },
        .jmp => |v| .{ .jmp = .{ .expr = try self.substituteExpr(v.expr), .span = v.span } },
        .jeq => |v| .{ .jeq = .{ .expr = try self.substituteExpr(v.expr), .span = v.span } },
        .jne => |v| .{ .jne = .{ .expr = try self.substituteExpr(v.expr), .span = v.span } },
        .jlt => |v| .{ .jlt = .{ .expr = try self.substituteExpr(v.expr), .span = v.span } },
        .jgt => |v| .{ .jgt = .{ .expr = try self.substituteExpr(v.expr), .span = v.span } },
        .jle => |v| .{ .jle = .{ .expr = try self.substituteExpr(v.expr), .span = v.span } },
        .jge => |v| .{ .jge = .{ .expr = try self.substituteExpr(v.expr), .span = v.span } },
        .call => |v| .{ .call = .{ .expr = try self.substituteExpr(v.expr), .span = v.span } },
        .call_variadic => |v| .{ .call_variadic = .{ .name = try self.substituteExpr(v.name), .variadic_types = v.variadic_types, .span = v.span } },
        .inc => |v| .{ .inc = .{ .expr = try self.substituteExpr(v.expr), .span = v.span } },
        .dec => |v| .{ .dec = .{ .expr = try self.substituteExpr(v.expr), .span = v.span } },
        .neg => |v| .{ .neg = .{ .expr = try self.substituteExpr(v.expr), .span = v.span } },
        .mov => |v| .{ .mov = .{
            .data_size = if (v.data_size) |size| try self.substituteExpr(size) else null,
            .expr1 = try self.substituteExpr(v.expr1),
            .expr2 = try self.substituteExpr(v.expr2),
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
        .rol => |v| .{ .rol = .{ .expr1 = try self.substituteExpr(v.expr1), .expr2 = try self.substituteExpr(v.expr2), .expr3 = try self.substituteExpr(v.expr3), .span = v.span } },
        .ror => |v| .{ .ror = .{ .expr1 = try self.substituteExpr(v.expr1), .expr2 = try self.substituteExpr(v.expr2), .expr3 = try self.substituteExpr(v.expr3), .span = v.span } },
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
        .dw => |v| .{ .dw = .{
            .exprs = blk: {
                var new_exprs = try ArrayList(*ast.Expression).initCapacity(arena_alloc, v.exprs.len);
                for (v.exprs) |expr| {
                    new_exprs.appendAssumeCapacity(try self.substituteExpr(expr));
                }
                break :blk try new_exprs.toOwnedSlice();
            },
            .span = v.span,
        } },
        .dd => |v| .{ .dd = .{
            .exprs = blk: {
                var new_exprs = try ArrayList(*ast.Expression).initCapacity(arena_alloc, v.exprs.len);
                for (v.exprs) |expr| {
                    new_exprs.appendAssumeCapacity(try self.substituteExpr(expr));
                }
                break :blk try new_exprs.toOwnedSlice();
            },
            .span = v.span,
        } },
        .dq => |v| .{ .dq = .{
            .exprs = blk: {
                var new_exprs = try ArrayList(*ast.Expression).initCapacity(arena_alloc, v.exprs.len);
                for (v.exprs) |expr| {
                    new_exprs.appendAssumeCapacity(try self.substituteExpr(expr));
                }
                break :blk try new_exprs.toOwnedSlice();
            },
            .span = v.span,
        } },
        .resb => |v| .{ .resb = .{ .expr = try self.substituteExpr(v.expr), .span = v.span } },
        .resw => |v| .{ .resw = .{ .expr = try self.substituteExpr(v.expr), .span = v.span } },
        .resd => |v| .{ .resd = .{ .expr = try self.substituteExpr(v.expr), .span = v.span } },
        .resq => |v| .{ .resq = .{ .expr = try self.substituteExpr(v.expr), .span = v.span } },
        .macro_def => null, // already handled in process()
        .macro_call => null, // already handled in process()
    };
}

fn processInclude(self: *Preprocessor, file_path: []const u8, span: Span) anyerror![]ast.Statement {
    const arena_alloc = self.arena.allocator();

    var found_path: ?[]const u8 = null;
    for (self.include_paths.items) |include_dir| {
        const candidate = try fs.path.join(arena_alloc, &.{ include_dir, file_path });
        if (!utils.fileExists(self.io, candidate)) continue;
        found_path = candidate;
        break;
    }

    const path = found_path orelse return self.reportError("include file not found", span);

    const content = try utils.readFromFile(self.io, arena_alloc, path);
    try self.reporter.addSource(path, content);

    const included_statements = try self.parseFileContent(content, path);

    var sub_preprocessor = Preprocessor{
        .io = self.io,
        .filename = path,
        .input = content,
        .program = included_statements,
        .interner = self.interner,
        .definitions = try self.definitions.clone(),
        .macros = try self.macros.clone(),
        .include_paths = try self.include_paths.clone(),
        .reporter = self.reporter,
        .arena = std.heap.ArenaAllocator.init(arena_alloc),
    };
    defer {
        sub_preprocessor.definitions.deinit();
        sub_preprocessor.macros.deinit();
        sub_preprocessor.include_paths.deinit();
    }

    const processed = try sub_preprocessor.process();

    var definitions_iter = sub_preprocessor.definitions.iterator();
    while (definitions_iter.next()) |entry| {
        try self.definitions.put(entry.key_ptr.*, entry.value_ptr.*);
    }

    var macros_iter = sub_preprocessor.macros.iterator();
    while (macros_iter.next()) |entry| {
        try self.macros.put(entry.key_ptr.*, entry.value_ptr.*);
    }

    return processed;
}

fn parseFileContent(self: *Preprocessor, content: []const u8, path: []const u8) ![]ast.Statement {
    var lexer = Lexer.init(path, content, self.interner, self.arena.allocator());
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
                    .identifier => |ident_id| ident_id,
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
                    .identifier => |ident_id| ident_id,
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
        .identifier => |name_id| blk: {
            if (self.definitions.get(name_id)) |replacement| {
                if (replacement) |r| {
                    break :blk self.substituteExpr(r);
                }
            }
            break :blk expr;
        },
        .address => |v| blk: {
            const new_base = try self.substituteExpr(v.base);
            const new_offset = if (v.offset) |offset|
                try self.substituteExpr(offset)
            else
                null;
            break :blk try self.createExpr(.{ .address = .{ .base = new_base, .offset = new_offset } });
        },
        .register, .integer_literal, .float_literal, .string_literal, .data_size => expr,
        .unary_op => |v| try self.evaluateUnaryOp(v),
        .binary_op => |v| try self.evaluateBinaryOp(v),
    };
}

fn evaluateUnaryOp(self: *Preprocessor, v: ast.Expression.UnaryOp) !*ast.Expression {
    const expr = try self.substituteExpr(v.expr);
    switch (expr.*) {
        .integer_literal => |int| {
            const result = switch (v.op) {
                .neg => blk: {
                    if (int == std.math.minInt(i64)) {
                        return self.reportError("integer overflow: cannot negate minimum value", v.span);
                    }
                    break :blk -int;
                },
            };
            return self.createExpr(.{ .integer_literal = result });
        },
        .float_literal => |float| {
            const result = switch (v.op) {
                .neg => -float,
            };
            return self.createExpr(.{ .float_literal = result });
        },
        else => {
            return self.reportError("cannot apply unary operator to non-literal expression", v.span);
        },
    }
}

fn evaluateBinaryOp(self: *Preprocessor, v: ast.Expression.BinaryOp) !*ast.Expression {
    const lhs = try self.substituteExpr(v.lhs);
    const rhs = try self.substituteExpr(v.rhs);

    if (lhs.* == .integer_literal and rhs.* == .integer_literal) {
        const l_val = lhs.integer_literal;
        const r_val = rhs.integer_literal;

        if ((v.op == .div) and r_val == 0) {
            return self.reportError("division by zero", v.span);
        }

        const result = switch (v.op) {
            .add => blk: {
                const res = @addWithOverflow(l_val, r_val);
                if (res[1] != 0) {
                    return self.reportError("integer overflow in addition", v.span);
                }
                break :blk res[0];
            },
            .sub => blk: {
                const res = @subWithOverflow(l_val, r_val);
                if (res[1] != 0) {
                    return self.reportError("integer overflow in subtraction", v.span);
                }
                break :blk res[0];
            },
            .mul => blk: {
                const res = @mulWithOverflow(l_val, r_val);
                if (res[1] != 0) {
                    return self.reportError("integer overflow in multiplication", v.span);
                }
                break :blk res[0];
            },
            .div => @divTrunc(l_val, r_val),
            .bit_or => l_val | r_val,
            .bit_and => l_val & r_val,
            .bit_xor => l_val ^ r_val,
        };

        return self.createExpr(.{ .integer_literal = result });
    }

    if (lhs.* == .float_literal and rhs.* == .float_literal) {
        const l_val = lhs.float_literal;
        const r_val = rhs.float_literal;

        if (v.op == .div and r_val == 0.0) {
            return self.reportError("division by zero", v.span);
        }

        const result = switch (v.op) {
            .add => l_val + r_val,
            .sub => l_val - r_val,
            .mul => l_val * r_val,
            .div => l_val / r_val,
            else => return self.reportError("invalid operator for float operands", v.span),
        };

        if (std.math.isNan(result)) {
            return self.reportError("operation resulted in NaN", v.span);
        }
        if (std.math.isInf(result)) {
            return self.reportError("floating point overflow", v.span);
        }

        return self.createExpr(.{ .float_literal = result });
    }

    if ((lhs.* == .integer_literal and rhs.* == .float_literal) or
        (lhs.* == .float_literal and rhs.* == .integer_literal))
    {
        return self.reportError("type mismatch: cannot operate on integer and float", v.span);
    }

    return self.createExpr(.{ .binary_op = .{
        .lhs = lhs,
        .op = v.op,
        .rhs = rhs,
        .span = v.span,
    } });
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
