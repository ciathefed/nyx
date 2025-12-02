const std = @import("std");
const process = std.process;
const ascii = std.ascii;
const mem = std.mem;
const heap = std.heap;
const fmt = std.fmt;
const Allocator = std.mem.Allocator;
const ArrayList = std.array_list.Managed;
const fehler = @import("fehler");
const Span = @import("../Span.zig");
const Lexer = @import("../lexer/Lexer.zig");
const Token = @import("../lexer/Token.zig");
const Register = @import("../vm/register.zig").Register;
const DataSize = @import("immediate.zig").DataSize;
const ast = @import("ast.zig");

const Parser = @This();

lexer: *Lexer,
reporter: *fehler.ErrorReporter,
prev_token: Token,
cur_token: Token,
peek_token: Token,
arena: heap.ArenaAllocator,

pub fn init(
    lexer: *Lexer,
    reporter: *fehler.ErrorReporter,
    allocator: Allocator,
) Parser {
    var cur_token = lexer.nextToken();
    var peek_token = lexer.nextToken();

    while (cur_token.kind == .newline) {
        cur_token = peek_token;
        peek_token = lexer.nextToken();
    }

    const arena = heap.ArenaAllocator.init(allocator);

    return Parser{
        .lexer = lexer,
        .reporter = reporter,
        .prev_token = cur_token,
        .cur_token = cur_token,
        .peek_token = peek_token,
        .arena = arena,
    };
}

pub fn deinit(self: *Parser) void {
    self.arena.deinit();
}

pub fn parse(self: *Parser) ![]ast.Statement {
    var stmts = ArrayList(ast.Statement).init(self.arena.allocator());
    while (self.cur_token.kind != .eof) {
        try stmts.append(try self.parseStatement());
    }
    return try stmts.toOwnedSlice();
}

fn parseStatement(self: *Parser) !ast.Statement {
    const cur_span = self.cur_token.span;
    switch (self.cur_token.kind) {
        .identifier => {
            if (self.peekTokenIs(.colon)) {
                const name_id = self.cur_token.string_id;
                self.nextToken();
                self.nextToken();
                return .{ .label = .{
                    .name = name_id,
                    .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
                } };
            } else {
                self.report(.err, "unexpected token", self.cur_token.span, 1);
                return error.ParserError;
            }
        },
        .kw_error => {
            self.nextToken();
            const message = try self.parseExpression();
            return .{ .@"error" = .{
                .expr = message,
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_define => {
            self.nextToken();

            if (!self.curTokenIs(.identifier)) {
                self.report(.err, "expected identifier after #define", self.cur_token.span, 1);
                return error.ParserError;
            }

            const name_id = self.cur_token.string_id;
            const name = try self.arena.allocator().create(ast.Expression);
            name.* = .{ .identifier = name_id };

            self.nextTokenRaw();

            var expr: ?*ast.Expression = null;
            if (self.curTokenIs(.newline) or self.curTokenIs(.eof)) {
                self.nextToken();
            } else {
                expr = try self.parseExpression();
            }

            return .{ .define = .{
                .name = name,
                .expr = expr,
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_include => {
            self.nextToken();
            const path = try self.parseExpression();
            return .{ .include = .{
                .expr = path,
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_ifdef => {
            self.nextToken();
            const expr = try self.parseExpression();
            return .{ .ifdef = .{
                .expr = expr,
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_ifndef => {
            self.nextToken();
            const expr = try self.parseExpression();
            return .{ .ifndef = .{
                .expr = expr,
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_else => {
            self.nextToken();
            return .{ .@"else" = .init(cur_span.start, self.prev_token.span.end, cur_span.filename) };
        },
        .kw_endif => {
            self.nextToken();
            return .{ .endif = .init(cur_span.start, self.prev_token.span.end, cur_span.filename) };
        },
        .kw_section => {
            self.nextToken();

            const section_type: ast.Statement.Section.Type = switch (self.cur_token.kind) {
                .identifier => blk: {
                    const ident = self.lexer.interner.get(self.cur_token.string_id).?;
                    if (mem.eql(u8, ident, "text")) {
                        break :blk .text;
                    } else if (mem.eql(u8, ident, "data")) {
                        break :blk .data;
                    } else {
                        self.report(.err, "unknown section", self.cur_token.span, 1);
                        return error.ParserError;
                    }
                },
                else => {
                    self.report(.err, "expected section name (text or data)", self.cur_token.span, 1);
                    return error.ParserError;
                },
            };

            self.nextToken();

            return .{ .section = .{
                .type = section_type,
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_entry => {
            self.nextToken();
            const expr = try self.parseExpression();
            return .{ .entry = .{
                .expr = expr,
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_ascii => {
            self.nextToken();
            const expr = try self.parseExpression();
            return .{ .ascii = .{
                .expr = expr,
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_asciz => {
            self.nextToken();
            const expr = try self.parseExpression();
            return .{ .asciz = .{
                .expr = expr,
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_extern => {
            self.nextToken();
            const expr = try self.parseExpression();
            return .{ .@"extern" = .{
                .expr = expr,
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_nop => {
            self.nextToken();
            return .{ .nop = .init(cur_span.start, self.prev_token.span.end, cur_span.filename) };
        },
        .kw_mov => {
            self.nextToken();
            const dest = try self.parseExpression();
            self.nextToken();
            const src = try self.parseExpression();
            return .{ .mov = .{
                .expr1 = dest,
                .expr2 = src,
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_ldr => {
            self.nextToken();
            const dest = try self.parseExpression();
            self.nextToken();
            const src = try self.parseExpression();
            return .{ .ldr = .{
                .expr1 = dest,
                .expr2 = src,
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_str => {
            self.nextToken();
            const src = try self.parseExpression();
            self.nextToken();
            const dest = try self.parseExpression();
            return .{ .str = .{
                .expr1 = src,
                .expr2 = dest,
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_sti => {
            self.nextToken();
            const size = try self.parseExpression();
            const src = try self.parseExpression();
            self.nextToken();
            const dest = try self.parseExpression();
            return .{ .sti = .{
                .expr1 = size,
                .expr2 = src,
                .expr3 = dest,
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_push => {
            self.nextToken();
            const size = if (self.curTokenIs(.data_size))
                try self.parseExpression()
            else
                null;
            const src = try self.parseExpression();
            return .{ .push = .{
                .data_size = size,
                .expr = src,
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_pop => {
            self.nextToken();
            const size = if (self.curTokenIs(.data_size))
                try self.parseExpression()
            else
                null;
            const dest = try self.parseExpression();
            return .{ .pop = .{
                .data_size = size,
                .expr = dest,
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_add => {
            self.nextToken();
            const dest = try self.parseExpression();
            try self.expect_cur(.comma);
            const lhs = try self.parseExpression();
            try self.expect_cur(.comma);
            const rhs = try self.parseExpression();
            return .{ .add = .{
                .expr1 = dest,
                .expr2 = lhs,
                .expr3 = rhs,
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_sub => {
            self.nextToken();
            const dest = try self.parseExpression();
            try self.expect_cur(.comma);
            const lhs = try self.parseExpression();
            try self.expect_cur(.comma);
            const rhs = try self.parseExpression();
            return .{ .sub = .{
                .expr1 = dest,
                .expr2 = lhs,
                .expr3 = rhs,
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_mul => {
            self.nextToken();
            const dest = try self.parseExpression();
            try self.expect_cur(.comma);
            const lhs = try self.parseExpression();
            try self.expect_cur(.comma);
            const rhs = try self.parseExpression();
            return .{ .mul = .{
                .expr1 = dest,
                .expr2 = lhs,
                .expr3 = rhs,
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_div => {
            self.nextToken();
            const dest = try self.parseExpression();
            try self.expect_cur(.comma);
            const lhs = try self.parseExpression();
            try self.expect_cur(.comma);
            const rhs = try self.parseExpression();
            return .{ .div = .{
                .expr1 = dest,
                .expr2 = lhs,
                .expr3 = rhs,
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_and => {
            self.nextToken();
            const dest = try self.parseExpression();
            try self.expect_cur(.comma);
            const lhs = try self.parseExpression();
            try self.expect_cur(.comma);
            const rhs = try self.parseExpression();
            return .{ .@"and" = .{
                .expr1 = dest,
                .expr2 = lhs,
                .expr3 = rhs,
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_or => {
            self.nextToken();
            const dest = try self.parseExpression();
            try self.expect_cur(.comma);
            const lhs = try self.parseExpression();
            try self.expect_cur(.comma);
            const rhs = try self.parseExpression();
            return .{ .@"or" = .{
                .expr1 = dest,
                .expr2 = lhs,
                .expr3 = rhs,
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_xor => {
            self.nextToken();
            const dest = try self.parseExpression();
            try self.expect_cur(.comma);
            const lhs = try self.parseExpression();
            try self.expect_cur(.comma);
            const rhs = try self.parseExpression();
            return .{ .xor = .{
                .expr1 = dest,
                .expr2 = lhs,
                .expr3 = rhs,
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_shl => {
            self.nextToken();
            const dest = try self.parseExpression();
            try self.expect_cur(.comma);
            const lhs = try self.parseExpression();
            try self.expect_cur(.comma);
            const rhs = try self.parseExpression();
            return .{ .shl = .{
                .expr1 = dest,
                .expr2 = lhs,
                .expr3 = rhs,
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_shr => {
            self.nextToken();
            const dest = try self.parseExpression();
            try self.expect_cur(.comma);
            const lhs = try self.parseExpression();
            try self.expect_cur(.comma);
            const rhs = try self.parseExpression();
            return .{ .shr = .{
                .expr1 = dest,
                .expr2 = lhs,
                .expr3 = rhs,
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_cmp => {
            self.nextToken();
            const lhs = try self.parseExpression();
            self.nextToken();
            const rhs = try self.parseExpression();
            return .{ .cmp = .{
                .expr1 = lhs,
                .expr2 = rhs,
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_jmp => {
            self.nextToken();
            const expr = try self.parseExpression();
            return .{ .jmp = .{
                .expr = expr,
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_jeq => {
            self.nextToken();
            const expr = try self.parseExpression();
            return .{ .jeq = .{
                .expr = expr,
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_jne => {
            self.nextToken();
            const expr = try self.parseExpression();
            return .{ .jne = .{
                .expr = expr,
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_jlt => {
            self.nextToken();
            const expr = try self.parseExpression();
            return .{ .jlt = .{
                .expr = expr,
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_jgt => {
            self.nextToken();
            const expr = try self.parseExpression();
            return .{ .jgt = .{
                .expr = expr,
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_jle => {
            self.nextToken();
            const expr = try self.parseExpression();
            return .{ .jle = .{
                .expr = expr,
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_jge => {
            self.nextToken();
            const expr = try self.parseExpression();
            return .{ .jge = .{
                .expr = expr,
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_call => {
            self.nextToken();
            const expr = try self.parseExpression();
            return .{ .call = .{
                .expr = expr,
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_ret => {
            self.nextToken();
            return .{ .ret = .init(cur_span.start, self.prev_token.span.end, cur_span.filename) };
        },
        .kw_inc => {
            self.nextToken();
            const expr = try self.parseExpression();
            return .{ .inc = .{
                .expr = expr,
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_dec => {
            self.nextToken();
            const expr = try self.parseExpression();
            return .{ .dec = .{
                .expr = expr,
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_neg => {
            self.nextToken();
            const expr = try self.parseExpression();
            return .{ .neg = .{
                .expr = expr,
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_syscall => {
            self.nextToken();
            return .{
                .syscall = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            };
        },
        .kw_hlt => {
            self.nextToken();
            return .{
                .hlt = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            };
        },
        .kw_db => {
            self.nextToken();
            var exprs = ArrayList(*ast.Expression).init(self.arena.allocator());

            while (true) {
                try exprs.append(try self.parseExpression());
                if (self.curTokenIs(.comma)) {
                    self.nextToken();
                    continue;
                }
                break;
            }

            return .{ .db = .{
                .exprs = try exprs.toOwnedSlice(),
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_dw => {
            self.nextToken();
            var exprs = ArrayList(*ast.Expression).init(self.arena.allocator());

            while (true) {
                try exprs.append(try self.parseExpression());
                if (self.curTokenIs(.comma)) {
                    self.nextToken();
                    continue;
                }
                break;
            }

            return .{ .dw = .{
                .exprs = try exprs.toOwnedSlice(),
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_dd => {
            self.nextToken();
            var exprs = ArrayList(*ast.Expression).init(self.arena.allocator());

            while (true) {
                try exprs.append(try self.parseExpression());
                if (self.curTokenIs(.comma)) {
                    self.nextToken();
                    continue;
                }
                break;
            }

            return .{ .dd = .{
                .exprs = try exprs.toOwnedSlice(),
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_dq => {
            self.nextToken();
            var exprs = ArrayList(*ast.Expression).init(self.arena.allocator());

            while (true) {
                try exprs.append(try self.parseExpression());
                if (self.curTokenIs(.comma)) {
                    self.nextToken();
                    continue;
                }
                break;
            }

            return .{ .dq = .{
                .exprs = try exprs.toOwnedSlice(),
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_resb => {
            self.nextToken();
            const expr = try self.parseExpression();
            return .{ .resb = .{
                .expr = expr,
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_resw => {
            self.nextToken();
            const expr = try self.parseExpression();
            return .{ .resw = .{
                .expr = expr,
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_resd => {
            self.nextToken();
            const expr = try self.parseExpression();
            return .{ .resd = .{
                .expr = expr,
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .kw_resq => {
            self.nextToken();
            const expr = try self.parseExpression();
            return .{ .resq = .{
                .expr = expr,
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        else => {
            self.report(.err, "unexpected token", self.cur_token.span, 1);
            return error.ParserError;
        },
    }
}

fn parseExpression(self: *Parser) anyerror!*ast.Expression {
    const expr = try self.arena.allocator().create(ast.Expression);
    expr.* = try self.parseBinaryExpression(0);
    return expr;
}

fn parseBinaryExpression(self: *Parser, min_prec: u8) anyerror!ast.Expression {
    const cur_span = self.cur_token.span;
    var lhs = try self.parsePrimary();

    while (true) {
        const op: ast.Expression.BinaryOp.Op = switch (self.cur_token.kind) {
            .plus => .add,
            .minus => .sub,
            .asterisk => .mul,
            .slash => .div,
            .pipe => .bit_or,
            .ampersand => .bit_and,
            .caret => .bit_xor,
            else => break,
        };

        const prec = binaryPrecedence(op);
        if (prec < min_prec) break;

        self.nextToken();
        const rhs = try self.parseBinaryExpression(prec + 1);

        const lhs_ptr = try self.arena.allocator().create(ast.Expression);
        lhs_ptr.* = lhs;

        const rhs_ptr = try self.arena.allocator().create(ast.Expression);
        rhs_ptr.* = rhs;

        lhs = .{ .binary_op = .{
            .lhs = lhs_ptr,
            .op = op,
            .rhs = rhs_ptr,
            .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
        } };
    }

    return lhs;
}

fn parsePrimary(self: *Parser) anyerror!ast.Expression {
    switch (self.cur_token.kind) {
        .minus => {
            const cur_span = self.cur_token.span;
            self.nextToken();
            const expr = try self.parsePrimary();

            const expr_ptr = try self.arena.allocator().create(ast.Expression);
            expr_ptr.* = expr;

            return .{ .unary_op = .{
                .op = .neg,
                .expr = expr_ptr,
                .span = .init(cur_span.start, self.prev_token.span.end, cur_span.filename),
            } };
        },
        .identifier => {
            const id = self.cur_token.string_id;
            self.nextToken();
            return .{ .identifier = id };
        },
        .register => {
            const reg = Register.fromString(self.cur_token.literal) catch {
                self.report(.err, "invalid register", self.cur_token.span, 1);
                return error.ParserError;
            };
            self.nextToken();
            return .{ .register = reg };
        },
        .integer => {
            const int = fmt.parseInt(i64, self.cur_token.literal, 10) catch {
                self.report(.err, "invalid integer", self.cur_token.span, 1);
                return error.ParserError;
            };
            self.nextToken();
            return .{ .integer_literal = int };
        },
        .hexadecimal => {
            const int = fmt.parseInt(i64, self.cur_token.literal[2..], 16) catch {
                self.report(.err, "invalid hexadecimal number", self.cur_token.span, 1);
                return error.ParserError;
            };
            self.nextToken();
            return .{ .integer_literal = int };
        },
        .binary => {
            const int = fmt.parseInt(i64, self.cur_token.literal[2..], 2) catch {
                self.report(.err, "invalid binary number", self.cur_token.span, 1);
                return error.ParserError;
            };
            self.nextToken();
            return .{ .integer_literal = int };
        },
        .octal => {
            const int = fmt.parseInt(i64, self.cur_token.literal[2..], 8) catch {
                self.report(.err, "invalid octal number", self.cur_token.span, 1);
                return error.ParserError;
            };
            self.nextToken();
            return .{ .integer_literal = int };
        },
        .float => {
            const float = fmt.parseFloat(f64, self.cur_token.literal) catch {
                self.report(.err, "invalid float", self.cur_token.span, 1);
                return error.ParserError;
            };
            self.nextToken();
            return .{ .float_literal = float };
        },
        .string => {
            const id = self.cur_token.string_id;
            self.nextToken();
            return .{ .string_literal = id };
        },
        .data_size => {
            const literal = self.cur_token.literal;
            const size = DataSize.fromString(literal) catch {
                self.report(.err, "invalid data size", self.cur_token.span, 1);
                return error.ParserError;
            };
            self.nextToken();
            return .{ .data_size = size };
        },
        .lbracket => {
            self.nextToken();

            const base = try self.parseExpression();
            const offset = if (self.curTokenIs(.comma)) blk: {
                self.nextToken();
                break :blk try self.parseExpression();
            } else null;

            if (!self.curTokenIs(.rbracket)) {
                const msg = try fmt.allocPrint(
                    self.arena.allocator(),
                    "expected \"{s}\", got \"{s}\" instead",
                    .{ "]", self.cur_token.literal },
                );
                self.report(.err, msg, self.cur_token.span, 1);
                return error.ParserError;
            }

            self.nextToken();

            return .{ .address = .{
                .base = base,
                .offset = offset,
            } };
        },
        .lparen => {
            self.nextToken();
            const expr_ptr = try self.parseExpression();
            if (!self.curTokenIs(.rparen)) {
                const msg = try fmt.allocPrint(
                    self.arena.allocator(),
                    "expected \"{s}\", got \"{s}\" instead",
                    .{ ")", self.cur_token.literal },
                );
                self.report(.err, msg, self.cur_token.span, 1);
                return error.ParserError;
            }
            self.nextToken();
            return expr_ptr.*;
        },
        else => {
            self.report(.err, "unexpected token", self.cur_token.span, 1);
            return error.ParserError;
        },
    }
}

fn report(
    self: *Parser,
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
        process.exit(code);
    }
}

fn binaryPrecedence(op: ast.Expression.BinaryOp.Op) u8 {
    return switch (op) {
        .mul, .div => 20,
        .add, .sub => 10,
        .bit_and => 5,
        .bit_or => 4,
        .bit_xor => 3,
    };
}

fn nextTokenRaw(self: *Parser) void {
    self.prev_token = self.cur_token;
    self.cur_token = self.peek_token;
    self.peek_token = self.lexer.nextToken();
}

fn nextToken(self: *Parser) void {
    self.prev_token = self.cur_token;
    self.cur_token = self.peek_token;
    self.peek_token = self.lexer.nextToken();

    while (self.cur_token.kind == .newline) {
        self.cur_token = self.peek_token;
        self.peek_token = self.lexer.nextToken();
    }
}

fn curTokenIs(self: *Parser, kind: Token.Kind) bool {
    return self.cur_token.kind == kind;
}

fn peekTokenIs(self: *Parser, kind: Token.Kind) bool {
    return self.peek_token.kind == kind;
}

fn expect_cur(self: *Parser, kind: Token.Kind) !void {
    if (self.curTokenIs(kind)) {
        self.nextToken();
    } else {
        self.report(.err, "unexpected token", self.peek_token.span, 1);
        return error.ParserError;
    }
}
