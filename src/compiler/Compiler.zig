// TODO: a lot of these "else => {}" statements default to a very general error,
//       they could be specific erros instead

const std = @import("std");
const process = std.process;
const mem = std.mem;
const Allocator = mem.Allocator;
const ArrayList = std.array_list.Managed;
const Bytecode = @import("Bytecode.zig");
const Opcode = @import("opcode.zig").Opcode;
const Span = @import("../Span.zig");
const DataSize = @import("../parser/immediate.zig").DataSize;
const fehler = @import("fehler");
const ast = @import("../parser/ast.zig");

const Compiler = @This();

pub const addressing_variant_1: u8 = 0x00; // [REGISTER, ?INTEGER]
pub const addressing_variant_2: u8 = 0x01; // [INTEGER, ?INTEGER]

pub const Entry = union(enum) {
    address: u64,
    fixup: Entry.Fixup,

    pub const Fixup = struct {
        label: []const u8,
        span: Span,
    };
};

const Label = struct {
    section: Bytecode.Section,
    addr: usize,
};

const Fixup = struct {
    size: DataSize,
    label: []const u8,
    span: Span,
};

program: []ast.Statement,
bytecode: Bytecode,
labels: std.StringHashMap(Label),
fixups: std.AutoHashMap(Label, Fixup),
externs: ArrayList([]const u8),
entry: ?Entry,
filename: []const u8,
input: []const u8,
reporter: *fehler.ErrorReporter,
allocator: Allocator,

pub fn init(
    program: []ast.Statement,
    filename: []const u8,
    input: []const u8,
    reporter: *fehler.ErrorReporter,
    allocator: Allocator,
) !Compiler {
    return Compiler{
        .program = program,
        .bytecode = try .init(4 * program.len, allocator),
        .labels = .init(allocator),
        .fixups = .init(allocator),
        .externs = .init(allocator),
        .entry = null,
        .filename = filename,
        .input = input,
        .reporter = reporter,
        .allocator = allocator,
    };
}

pub fn deinit(self: *Compiler) void {
    self.bytecode.deinit();
    self.labels.deinit();
    self.fixups.deinit();
    self.externs.deinit();
}

pub fn compile(self: *Compiler) ![]u8 {
    for (self.program) |stmt| {
        switch (stmt) {
            .label => |v| {
                const offset = self.bytecode.len(self.bytecode.current_section);
                try self.labels.put(v.name, .{ .section = self.bytecode.current_section, .addr = offset });
                if (mem.eql(u8, v.name, "_start") and self.entry == null) {
                    self.entry = .{ .fixup = .{ .label = v.name, .span = v.span } };
                }
            },
            .section => |v| self.bytecode.current_section = switch (v.type) {
                .text => .text,
                .data => .data,
            },
            .entry => |v| {
                switch (v.expr.*) {
                    .integer_literal => |int| self.entry = .{ .address = @intCast(int) },
                    .identifier => |ident| self.entry = .{ .fixup = .{ .label = ident, .span = v.span } },
                    else => {
                        self.report(.err, "unsupported operand", v.span, 1);
                        return error.CompilerError;
                    },
                }
            },
            .ascii => |v| {
                switch (v.expr.*) {
                    .string_literal => |str| {
                        try self.bytecode.extend(str);
                    },
                    else => {
                        self.report(.err, "unsupported operand", v.span, 1);
                        return error.CompilerError;
                    },
                }
            },
            .asciz => |v| {
                switch (v.expr.*) {
                    .string_literal => |str| {
                        try self.bytecode.extend(str);
                        try self.bytecode.push(0x00);
                    },
                    else => {
                        self.report(.err, "unsupported operand", v.span, 1);
                        return error.CompilerError;
                    },
                }
            },
            .@"extern" => |v| {
                switch (v.expr.*) {
                    .identifier => |str| try self.externs.append(str),
                    else => {
                        self.report(.err, "unsupported operand", v.span, 1);
                        return error.CompilerError;
                    },
                }
            },
            .nop => try self.bytecode.push(Opcode.nop),
            .mov => |v| try self.compileMov(v.expr1, v.expr2, v.span),
            .ldr => |v| try self.compileLdrOrStr(v.expr1, v.expr2, Opcode.ldr, v.span),
            .str => |v| try self.compileLdrOrStr(v.expr1, v.expr2, Opcode.str, v.span),
            .sti => |v| try self.compileSti(v.expr1, v.expr2, v.expr3, v.span),
            .push => |v| try self.compilePush(v.data_size, v.expr, v.span),
            .pop => |v| try self.compilePop(v.data_size, v.expr, v.span),
            .add => |v| try self.compileAritmetic(v.expr1, v.expr2, v.expr3, .add, v.span),
            .sub => |v| try self.compileAritmetic(v.expr1, v.expr2, v.expr3, .sub, v.span),
            .mul => |v| try self.compileAritmetic(v.expr1, v.expr2, v.expr3, .mul, v.span),
            .div => |v| try self.compileAritmetic(v.expr1, v.expr2, v.expr3, .div, v.span),
            .@"and" => |v| try self.compileBitwise(v.expr1, v.expr2, v.expr3, .@"and", v.span),
            .@"or" => |v| try self.compileBitwise(v.expr1, v.expr2, v.expr3, .@"or", v.span),
            .xor => |v| try self.compileBitwise(v.expr1, v.expr2, v.expr3, .xor, v.span),
            .shl => |v| try self.compileBitwise(v.expr1, v.expr2, v.expr3, .shl, v.span),
            .shr => |v| try self.compileBitwise(v.expr1, v.expr2, v.expr3, .shr, v.span),
            .cmp => |v| try self.compileCmp(v.expr1, v.expr2, v.span),
            .jmp => |v| try self.compileJump(v.expr, .jmp, v.span),
            .jne => |v| try self.compileJump(v.expr, .jne, v.span),
            .jeq => |v| try self.compileJump(v.expr, .jeq, v.span),
            .jlt => |v| try self.compileJump(v.expr, .jlt, v.span),
            .jgt => |v| try self.compileJump(v.expr, .jgt, v.span),
            .jle => |v| try self.compileJump(v.expr, .jle, v.span),
            .jge => |v| try self.compileJump(v.expr, .jge, v.span),
            .call => |v| try self.compileCall(v.expr, v.span),
            .ret => try self.bytecode.push(Opcode.ret),
            .inc => |v| try self.compileIncOrDec(v.expr, .inc, v.span),
            .dec => |v| try self.compileIncOrDec(v.expr, .dec, v.span),
            .syscall => try self.bytecode.push(Opcode.syscall),
            .hlt => try self.bytecode.push(Opcode.hlt),
            .db => |v| {
                for (v.exprs) |expr| {
                    switch (expr.*) {
                        .integer_literal => |int| try self.bytecode.push(
                            @as(u8, @intCast(int)),
                        ),
                        .string_literal => |str| try self.bytecode.extend(
                            str,
                        ),
                        else => {
                            self.report(.err, "unsupported operand", v.span, 1);
                            return error.CompilerError;
                        },
                    }
                }
            },
            .resb => |v| {
                switch (v.expr.*) {
                    .integer_literal => |int| try self.bytecode.grow(@intCast(int)),
                    else => {
                        self.report(.err, "unsupported operand", v.span, 1);
                        return error.CompilerError;
                    },
                }
            },
            else => |other| {
                const span = other.span();
                self.report(.err, "unsupported operation", span, 1);
                return error.CompilerError;
            },
        }
    }

    var fixup_iter = self.fixups.iterator();
    while (fixup_iter.next()) |fixup| {
        if (self.labels.get(fixup.value_ptr.label)) |label| {
            const pos = switch (label.section) {
                .text => label.addr,
                .data => self.bytecode.len(.text) + label.addr,
            };

            switch (fixup.value_ptr.size) {
                .byte => self.bytecode.writeU8At(fixup.key_ptr.section, fixup.key_ptr.addr, @intCast(pos)),
                .word => self.bytecode.writeU16At(fixup.key_ptr.section, fixup.key_ptr.addr, @intCast(pos)),
                .dword => self.bytecode.writeU32At(fixup.key_ptr.section, fixup.key_ptr.addr, @intCast(pos)),
                .qword => self.bytecode.writeU64At(fixup.key_ptr.section, fixup.key_ptr.addr, @intCast(pos)),
                else => unreachable,
            }
        } else {
            self.report(.err, "undefined label", fixup.value_ptr.span, 1);
            return error.CompilerError;
        }
    }

    const entry: u64 = if (self.entry) |entry| switch (entry) {
        .address => |v| v,
        .fixup => |v| blk: {
            if (self.labels.get(v.label)) |label| {
                const pos = switch (label.section) {
                    .text => label.addr,
                    .data => self.bytecode.len(.text) + label.addr,
                };
                break :blk @intCast(pos);
            } else {
                self.report(.err, "undefined label", v.span, 1);
                return error.CompilerError;
            }
        },
    } else 0x00;

    var bytecode = ArrayList(u8).init(self.allocator);
    try bytecode.appendSlice(&mem.toBytes(entry));
    const final = try self.bytecode.finalize(self.allocator);
    defer self.allocator.free(final);
    try bytecode.appendSlice(final);

    return bytecode.toOwnedSlice();
}

fn compileMov(self: *Compiler, lhs: *ast.Expression, rhs: *ast.Expression, span: Span) !void {
    switch (lhs.*) {
        .register => |dest| {
            switch (rhs.*) {
                .register => |src| {
                    try self.bytecode.push(Opcode.mov_reg_reg);
                    try self.bytecode.push(dest);
                    try self.bytecode.push(src);
                    return;
                },
                .integer_literal => |int| {
                    try self.bytecode.push(Opcode.mov_reg_imm);
                    try self.bytecode.push(dest);
                    switch (DataSize.fromRegister(dest)) {
                        .byte => try self.bytecode.push(@as(u8, @intCast(int))),
                        .word => try self.bytecode.extend(&mem.toBytes(@as(u16, @intCast(int)))),
                        .dword => try self.bytecode.extend(&mem.toBytes(@as(u32, @intCast(int)))),
                        .qword => try self.bytecode.extend(&mem.toBytes(@as(u64, @intCast(int)))),
                        .float => try self.bytecode.extend(&mem.toBytes(@as(f32, @floatFromInt(int)))),
                        .double => try self.bytecode.extend(&mem.toBytes(@as(f64, @floatFromInt(int)))),
                    }
                    return;
                },
                .float_literal => |float| {
                    try self.bytecode.push(Opcode.mov_reg_imm);
                    try self.bytecode.push(dest);
                    switch (DataSize.fromRegister(dest)) {
                        .byte => try self.bytecode.push(@as(u8, @intFromFloat(float))),
                        .word => try self.bytecode.extend(&mem.toBytes(@as(u16, @intFromFloat(float)))),
                        .dword => try self.bytecode.extend(&mem.toBytes(@as(u32, @intFromFloat(float)))),
                        .qword => try self.bytecode.extend(&mem.toBytes(@as(u64, @intFromFloat(float)))),
                        .float => try self.bytecode.extend(&mem.toBytes(@as(f32, @floatCast(float)))),
                        .double => try self.bytecode.extend(&mem.toBytes(@as(f64, @floatCast(float)))),
                    }
                    return;
                },
                .identifier => |ident| {
                    try self.bytecode.push(Opcode.mov_reg_imm);
                    try self.bytecode.push(dest);
                    const size = DataSize.fromRegister(dest);
                    const offset = self.bytecode.len(self.bytecode.current_section);
                    try self.fixups.put(
                        .{ .section = self.bytecode.current_section, .addr = offset },
                        .{ .size = size, .label = ident, .span = span },
                    );
                    switch (size) {
                        .byte => try self.bytecode.push(@as(u8, 0x00)),
                        .word => try self.bytecode.extend(&mem.toBytes(@as(u16, 0x00))),
                        .dword => try self.bytecode.extend(&mem.toBytes(@as(u32, 0x00))),
                        .qword => try self.bytecode.extend(&mem.toBytes(@as(u64, 0x00))),
                        .float => unreachable,
                        .double => unreachable,
                    }
                    return;
                },
                else => {},
            }
        },
        else => {},
    }

    return self.reportError("unsupported operands", span);
}

fn compileLdrOrStr(
    self: *Compiler,
    lhs: *ast.Expression,
    rhs: *ast.Expression,
    opcode: Opcode,
    span: Span,
) !void {
    const l = switch (lhs.*) {
        .register => |reg| reg,
        else => return self.reportError("left operand must be a register", span),
    };

    const r = switch (rhs.*) {
        .address => |addr| addr,
        else => return self.reportError("right operand must be an address", span),
    };

    const offset = if (r.offset) |o| blk: {
        switch (o.*) {
            .integer_literal => |offset| break :blk offset,
            else => return self.reportError("offset must be an integer literal", span),
        }
    } else 0;

    switch (r.base.*) {
        .register => |base| {
            try self.bytecode.push(opcode);
            try self.bytecode.push(l);
            try self.bytecode.push(addressing_variant_1);
            try self.bytecode.push(base);
            try self.bytecode.extend(&mem.toBytes(@as(u64, @bitCast(offset))));
        },
        .integer_literal => |base| {
            try self.bytecode.push(opcode);
            try self.bytecode.push(l);
            try self.bytecode.push(addressing_variant_2);
            try self.bytecode.extend(&mem.toBytes(@as(u64, @bitCast(base))));
            try self.bytecode.extend(&mem.toBytes(@as(u64, @bitCast(offset))));
        },
        .identifier => |base| {
            try self.bytecode.push(opcode);
            try self.bytecode.push(l);
            try self.bytecode.push(addressing_variant_2);
            try self.fixups.put(
                .{ .section = self.bytecode.current_section, .addr = self.bytecode.len(self.bytecode.current_section) },
                .{ .size = .qword, .label = base, .span = span },
            );
            try self.bytecode.extend(&mem.toBytes(@as(u64, 0x00)));
            try self.bytecode.extend(&mem.toBytes(@as(u64, @bitCast(offset))));
        },
        else => return self.reportError("unsupported address base type", span),
    }
}

fn compileSti(
    self: *Compiler,
    data_size: *ast.Expression,
    lhs: *ast.Expression,
    rhs: *ast.Expression,
    span: Span,
) !void {
    const s = switch (data_size.*) {
        .data_size => |size| size,
        else => return self.reportError("expected data size specifier", span),
    };

    const value_bytes = switch (lhs.*) {
        .integer_literal => |val| blk: {
            break :blk switch (s) {
                .byte => &mem.toBytes(@as(u8, @intCast(val))),
                .word => &mem.toBytes(@as(u16, @intCast(val))),
                .dword => &mem.toBytes(@as(u32, @intCast(val))),
                .qword => &mem.toBytes(@as(u64, @intCast(val))),
                .float => &mem.toBytes(@as(f32, @floatFromInt(val))),
                .double => &mem.toBytes(@as(f64, @floatFromInt(val))),
            };
        },
        .float_literal => |val| blk: {
            break :blk switch (s) {
                .byte => &mem.toBytes(@as(u8, @intFromFloat(val))),
                .word => &mem.toBytes(@as(u16, @intFromFloat(val))),
                .dword => &mem.toBytes(@as(u32, @intFromFloat(val))),
                .qword => &mem.toBytes(@as(u64, @intFromFloat(val))),
                .float => &mem.toBytes(@as(f32, @floatCast(val))),
                .double => &mem.toBytes(val),
            };
        },
        else => return self.reportError("left operand must be an integer or float literal", span),
    };

    const r = switch (rhs.*) {
        .address => |addr| addr,
        else => return self.reportError("right operand must be an address", span),
    };

    const offset = if (r.offset) |o| blk: {
        switch (o.*) {
            .integer_literal => |offset| break :blk offset,
            else => return self.reportError("offset must be an integer literal", span),
        }
    } else 0;

    switch (r.base.*) {
        .register => |base| {
            try self.bytecode.push(Opcode.sti);
            try self.bytecode.push(s);
            try self.bytecode.extend(value_bytes);
            try self.bytecode.push(addressing_variant_1);
            try self.bytecode.push(base);
            try self.bytecode.extend(&mem.toBytes(@as(u64, @bitCast(offset))));
        },
        .integer_literal => |base| {
            try self.bytecode.push(Opcode.sti);
            try self.bytecode.push(s);
            try self.bytecode.extend(value_bytes);
            try self.bytecode.push(addressing_variant_2);
            try self.bytecode.extend(&mem.toBytes(@as(u64, @bitCast(base))));
            try self.bytecode.extend(&mem.toBytes(@as(u64, @bitCast(offset))));
        },
        .identifier => |base| {
            try self.bytecode.push(Opcode.sti);
            try self.bytecode.push(s);
            try self.bytecode.extend(value_bytes);
            try self.bytecode.push(addressing_variant_2);
            try self.fixups.put(
                .{ .section = self.bytecode.current_section, .addr = self.bytecode.len(self.bytecode.current_section) },
                .{ .size = .qword, .label = base, .span = span },
            );
            try self.bytecode.extend(&mem.toBytes(@as(u64, 0x00)));
            try self.bytecode.extend(&mem.toBytes(@as(u64, @bitCast(offset))));
        },
        else => return self.reportError("unsupported address base type", span),
    }
}

fn compilePush(self: *Compiler, data_size: ?*ast.Expression, expr: *ast.Expression, span: Span) !void {
    switch (expr.*) {
        .register => |src| {
            const size = if (data_size) |ds| switch (ds.*) {
                .data_size => |v| v,
                else => return self.reportError("expected data size specifier", span),
            } else DataSize.fromRegister(src);

            try self.bytecode.push(Opcode.push_reg);
            try self.bytecode.push(size);
            try self.bytecode.push(src);
            return;
        },
        .integer_literal => |src| {
            const size = if (data_size) |ds| switch (ds.*) {
                .data_size => |v| v,
                else => return self.reportError("expected data size specifier", span),
            } else return self.reportError("expected data size specifier", span);

            try self.bytecode.push(Opcode.push_imm);
            try self.bytecode.push(size);
            try self.bytecode.extend(switch (size) {
                .byte => &mem.toBytes(@as(u8, @intCast(src))),
                .word => &mem.toBytes(@as(u16, @intCast(src))),
                .dword => &mem.toBytes(@as(u32, @intCast(src))),
                .qword => &mem.toBytes(@as(u64, @intCast(src))),
                .float => &mem.toBytes(@as(f32, @floatFromInt(src))),
                .double => &mem.toBytes(@as(f64, @floatFromInt(src))),
            });
            return;
        },
        .float_literal => |src| {
            const size = if (data_size) |ds| switch (ds.*) {
                .data_size => |v| v,
                else => return self.reportError("expected data size specifier", span),
            } else return self.reportError("expected data size specifier", span);

            try self.bytecode.push(Opcode.push_imm);
            try self.bytecode.push(size);
            try self.bytecode.extend(switch (size) {
                .byte => &mem.toBytes(@as(u8, @intFromFloat(src))),
                .word => &mem.toBytes(@as(u16, @intFromFloat(src))),
                .dword => &mem.toBytes(@as(u32, @intFromFloat(src))),
                .qword => &mem.toBytes(@as(u64, @intFromFloat(src))),
                .float => &mem.toBytes(@as(f32, @floatCast(src))),
                .double => &mem.toBytes(@as(f64, @floatCast(src))),
            });
            return;
        },
        .identifier => |src| {
            const size = if (data_size) |ds| switch (ds.*) {
                .data_size => |v| v,
                else => return self.reportError("expected data size specifier", span),
            } else DataSize.qword;

            try self.bytecode.push(Opcode.push_imm);
            try self.bytecode.push(size);
            const offset = self.bytecode.len(self.bytecode.current_section);
            try self.fixups.put(
                .{ .section = self.bytecode.current_section, .addr = offset },
                .{ .size = .qword, .label = src, .span = span },
            );
            try self.bytecode.extend(&mem.toBytes(@as(u64, 0x00)));
            return;
        },
        .address => |src| {
            const size = if (data_size) |ds| switch (ds.*) {
                .data_size => |v| v,
                else => return self.reportError("expected data size specifier", span),
            } else return self.reportError("expected data size specifier", span);

            try self.bytecode.push(Opcode.push_addr);
            try self.bytecode.push(size);

            const offset = if (src.offset) |o| blk: {
                switch (o.*) {
                    .integer_literal => |offset| break :blk offset,
                    else => return self.reportError("offset must be an integer literal", span),
                }
            } else 0;

            switch (src.base.*) {
                .register => |base| {
                    try self.bytecode.push(addressing_variant_1);
                    try self.bytecode.push(base);
                    try self.bytecode.extend(&mem.toBytes(@as(u64, @bitCast(offset))));
                },
                .integer_literal => |base| {
                    try self.bytecode.push(addressing_variant_2);
                    try self.bytecode.extend(&mem.toBytes(@as(u64, @bitCast(base))));
                    try self.bytecode.extend(&mem.toBytes(@as(u64, @bitCast(offset))));
                },
                .identifier => |base| {
                    try self.bytecode.push(addressing_variant_2);
                    try self.fixups.put(
                        .{ .section = self.bytecode.current_section, .addr = self.bytecode.len(self.bytecode.current_section) },
                        .{ .size = .qword, .label = base, .span = span },
                    );
                    try self.bytecode.extend(&mem.toBytes(@as(u64, 0x00)));
                    try self.bytecode.extend(&mem.toBytes(@as(u64, @bitCast(offset))));
                },
                else => return self.reportError("unsupported address base type", span),
            }
            return;
        },
        else => {},
    }

    return self.reportError("unsupported operands", span);
}

fn compilePop(self: *Compiler, data_size: ?*ast.Expression, expr: *ast.Expression, span: Span) !void {
    switch (expr.*) {
        .register => |dest| {
            const size = if (data_size) |ds| switch (ds.*) {
                .data_size => |v| v,
                else => return self.reportError("expected data size specifier", span),
            } else DataSize.fromRegister(dest);

            try self.bytecode.push(Opcode.pop_reg);
            try self.bytecode.push(size);
            try self.bytecode.push(dest);
            return;
        },
        .address => |src| {
            const size = if (data_size) |ds| switch (ds.*) {
                .data_size => |v| v,
                else => return self.reportError("expected data size specifier", span),
            } else return self.reportError("expected data size specifier", span);

            try self.bytecode.push(Opcode.pop_addr);
            try self.bytecode.push(size);

            const offset = if (src.offset) |o| blk: {
                switch (o.*) {
                    .integer_literal => |offset| break :blk offset,
                    else => return self.reportError("offset must be an integer literal", span),
                }
            } else 0;

            switch (src.base.*) {
                .register => |base| {
                    try self.bytecode.push(addressing_variant_1);
                    try self.bytecode.push(base);
                    try self.bytecode.extend(&mem.toBytes(@as(u64, @bitCast(offset))));
                },
                .integer_literal => |base| {
                    try self.bytecode.push(addressing_variant_2);
                    try self.bytecode.extend(&mem.toBytes(@as(u64, @bitCast(base))));
                    try self.bytecode.extend(&mem.toBytes(@as(u64, @bitCast(offset))));
                },
                .identifier => |base| {
                    try self.bytecode.push(addressing_variant_2);
                    try self.fixups.put(
                        .{ .section = self.bytecode.current_section, .addr = self.bytecode.len(self.bytecode.current_section) },
                        .{ .size = .qword, .label = base, .span = span },
                    );
                    try self.bytecode.extend(&mem.toBytes(@as(u64, 0x00)));
                    try self.bytecode.extend(&mem.toBytes(@as(u64, @bitCast(offset))));
                },
                else => return self.reportError("unsupported address base type", span),
            }
            return;
        },
        else => {},
    }

    return self.reportError("unsupported operands", span);
}

fn compileAritmetic(
    self: *Compiler,
    dest: *ast.Expression,
    lhs: *ast.Expression,
    rhs: *ast.Expression,
    op: enum {
        add,
        sub,
        mul,
        div,
    },
    span: Span,
) !void {
    const dest_reg = switch (dest.*) {
        .register => |v| v,
        else => return self.reportError("first operand must be a register", span),
    };

    const lhs_reg = switch (lhs.*) {
        .register => |v| v,
        else => return self.reportError("second operand must be a register", span),
    };

    switch (rhs.*) {
        .register => |rhs_reg| {
            try self.bytecode.push(switch (op) {
                .add => Opcode.add_reg_reg_reg,
                .sub => Opcode.sub_reg_reg_reg,
                .mul => Opcode.mul_reg_reg_reg,
                .div => Opcode.div_reg_reg_reg,
            });
            try self.bytecode.push(dest_reg);
            try self.bytecode.push(lhs_reg);
            try self.bytecode.push(rhs_reg);
            return;
        },
        .integer_literal => |rhs_int| {
            try self.bytecode.push(switch (op) {
                .add => Opcode.add_reg_reg_imm,
                .sub => Opcode.sub_reg_reg_imm,
                .mul => Opcode.mul_reg_reg_imm,
                .div => Opcode.div_reg_reg_imm,
            });
            try self.bytecode.push(dest_reg);
            try self.bytecode.push(lhs_reg);
            try self.bytecode.extend(switch (DataSize.fromRegister(dest_reg)) {
                .byte => &mem.toBytes(@as(u8, @intCast(rhs_int))),
                .word => &mem.toBytes(@as(u16, @intCast(rhs_int))),
                .dword => &mem.toBytes(@as(u32, @intCast(rhs_int))),
                .qword => &mem.toBytes(@as(u64, @intCast(rhs_int))),
                .float => &mem.toBytes(@as(f32, @floatFromInt(rhs_int))),
                .double => &mem.toBytes(@as(f64, @floatFromInt(rhs_int))),
            });
            return;
        },
        .float_literal => |rhs_float| {
            try self.bytecode.push(switch (op) {
                .add => Opcode.add_reg_reg_imm,
                .sub => Opcode.sub_reg_reg_imm,
                .mul => Opcode.mul_reg_reg_imm,
                .div => Opcode.div_reg_reg_imm,
            });
            try self.bytecode.push(dest_reg);
            try self.bytecode.push(lhs_reg);
            try self.bytecode.extend(switch (DataSize.fromRegister(dest_reg)) {
                .byte => &mem.toBytes(@as(u8, @intFromFloat(rhs_float))),
                .word => &mem.toBytes(@as(u16, @intFromFloat(rhs_float))),
                .dword => &mem.toBytes(@as(u32, @intFromFloat(rhs_float))),
                .qword => &mem.toBytes(@as(u64, @intFromFloat(rhs_float))),
                .float => &mem.toBytes(@as(f32, @floatCast(rhs_float))),
                .double => &mem.toBytes(@as(f64, @floatCast(rhs_float))),
            });
            return;
        },
        else => {},
    }

    return self.reportError("unsupported operands", span);
}

fn compileBitwise(
    self: *Compiler,
    dest: *ast.Expression,
    lhs: *ast.Expression,
    rhs: *ast.Expression,
    op: enum {
        @"and",
        @"or",
        xor,
        shl,
        shr,
    },
    span: Span,
) !void {
    const dest_reg = switch (dest.*) {
        .register => |v| v,
        else => return self.reportError("first operand must be a register", span),
    };

    const lhs_reg = switch (lhs.*) {
        .register => |v| v,
        else => return self.reportError("second operand must be a register", span),
    };

    switch (DataSize.fromRegister(lhs_reg)) {
        .float, .double => return self.reportError("bitwise operations not supported on floating-point registers", span),
        else => {},
    }

    switch (rhs.*) {
        .register => |rhs_reg| {
            switch (DataSize.fromRegister(rhs_reg)) {
                .float, .double => return self.reportError("bitwise operations not supported on floating-point registers", span),
                else => {},
            }

            try self.bytecode.push(switch (op) {
                .@"and" => Opcode.and_reg_reg_reg,
                .@"or" => Opcode.or_reg_reg_reg,
                .xor => Opcode.xor_reg_reg_reg,
                .shl => Opcode.shl_reg_reg_reg,
                .shr => Opcode.shr_reg_reg_reg,
            });
            try self.bytecode.push(dest_reg);
            try self.bytecode.push(lhs_reg);
            try self.bytecode.push(rhs_reg);
            return;
        },
        .integer_literal => |rhs_int| {
            try self.bytecode.push(switch (op) {
                .@"and" => Opcode.and_reg_reg_imm,
                .@"or" => Opcode.or_reg_reg_imm,
                .xor => Opcode.xor_reg_reg_imm,
                .shl => Opcode.shl_reg_reg_imm,
                .shr => Opcode.shr_reg_reg_imm,
            });
            try self.bytecode.push(dest_reg);
            try self.bytecode.push(lhs_reg);
            try self.bytecode.extend(switch (DataSize.fromRegister(dest_reg)) {
                .byte => &mem.toBytes(@as(u8, @intCast(rhs_int))),
                .word => &mem.toBytes(@as(u16, @intCast(rhs_int))),
                .dword => &mem.toBytes(@as(u32, @intCast(rhs_int))),
                .qword => &mem.toBytes(@as(u64, @intCast(rhs_int))),
                .float => &mem.toBytes(@as(f32, @floatFromInt(rhs_int))),
                .double => &mem.toBytes(@as(f64, @floatFromInt(rhs_int))),
            });
            return;
        },
        .float_literal => return self.reportError("bitwise operations not supported on floating-point numbers", span),
        else => {},
    }

    return self.reportError("unsupported operands", span);
}

fn compileCmp(
    self: *Compiler,
    lhs: *ast.Expression,
    rhs: *ast.Expression,
    span: Span,
) !void {
    switch (lhs.*) {
        .register => |lhs_reg| {
            switch (rhs.*) {
                .register => |rhs_reg| {
                    try self.bytecode.push(Opcode.cmp_reg_reg);
                    try self.bytecode.push(lhs_reg);
                    try self.bytecode.push(rhs_reg);
                    return;
                },
                .integer_literal => |rhs_int| {
                    try self.bytecode.push(Opcode.cmp_reg_imm);
                    try self.bytecode.push(lhs_reg);
                    try self.bytecode.extend(switch (DataSize.fromRegister(lhs_reg)) {
                        .byte => &mem.toBytes(@as(u8, @intCast(rhs_int))),
                        .word => &mem.toBytes(@as(u16, @intCast(rhs_int))),
                        .dword => &mem.toBytes(@as(u32, @intCast(rhs_int))),
                        .qword => &mem.toBytes(@as(u64, @intCast(rhs_int))),
                        .float => &mem.toBytes(@as(f32, @floatFromInt(rhs_int))),
                        .double => &mem.toBytes(@as(f64, @floatFromInt(rhs_int))),
                    });
                    return;
                },
                .float_literal => |rhs_float| {
                    try self.bytecode.push(Opcode.cmp_reg_imm);
                    try self.bytecode.push(lhs_reg);
                    try self.bytecode.extend(switch (DataSize.fromRegister(lhs_reg)) {
                        .byte => &mem.toBytes(@as(u8, @intFromFloat(rhs_float))),
                        .word => &mem.toBytes(@as(u16, @intFromFloat(rhs_float))),
                        .dword => &mem.toBytes(@as(u32, @intFromFloat(rhs_float))),
                        .qword => &mem.toBytes(@as(u64, @intFromFloat(rhs_float))),
                        .float => &mem.toBytes(@as(f32, @floatCast(rhs_float))),
                        .double => &mem.toBytes(@as(f64, @floatCast(rhs_float))),
                    });
                    return;
                },
                else => {},
            }
        },
        // .integer_literal => |lhs_int| {},
        // .float_literal => |lhs_float| {},
        else => {},
    }

    return self.reportError("unsupported operands", span);
}

fn compileJump(
    self: *Compiler,
    expr: *ast.Expression,
    op: enum {
        jmp,
        jeq,
        jne,
        jlt,
        jgt,
        jle,
        jge,
    },
    span: Span,
) !void {
    switch (expr.*) {
        .integer_literal => |src| {
            try self.bytecode.push(switch (op) {
                .jmp => Opcode.jmp_imm,
                .jeq => Opcode.jeq_imm,
                .jne => Opcode.jne_imm,
                .jlt => Opcode.jlt_imm,
                .jgt => Opcode.jgt_imm,
                .jle => Opcode.jle_imm,
                .jge => Opcode.jge_imm,
            });
            try self.bytecode.extend(&mem.toBytes(@as(u64, @intCast(src))));
            return;
        },
        .register => |src| {
            try self.bytecode.push(switch (op) {
                .jmp => Opcode.jmp_reg,
                .jeq => Opcode.jeq_reg,
                .jne => Opcode.jne_reg,
                .jlt => Opcode.jlt_reg,
                .jgt => Opcode.jgt_reg,
                .jle => Opcode.jle_reg,
                .jge => Opcode.jge_reg,
            });
            try self.bytecode.push(src);
            return;
        },
        .identifier => |src| {
            try self.bytecode.push(switch (op) {
                .jmp => Opcode.jmp_imm,
                .jeq => Opcode.jeq_imm,
                .jne => Opcode.jne_imm,
                .jlt => Opcode.jlt_imm,
                .jgt => Opcode.jgt_imm,
                .jle => Opcode.jle_imm,
                .jge => Opcode.jge_imm,
            });
            const offset = self.bytecode.len(self.bytecode.current_section);
            try self.fixups.put(
                .{ .section = self.bytecode.current_section, .addr = offset },
                .{ .size = .qword, .label = src, .span = span },
            );
            try self.bytecode.extend(&mem.toBytes(@as(u64, 0x00)));
            return;
        },
        else => {},
    }

    return self.reportError("unsupported operand", span);
}

fn compileCall(self: *Compiler, expr: *ast.Expression, span: Span) !void {
    switch (expr.*) {
        .integer_literal => |src| {
            try self.bytecode.push(Opcode.call_imm);
            try self.bytecode.extend(&mem.toBytes(@as(u64, @intCast(src))));
            return;
        },
        .register => |src| {
            try self.bytecode.push(Opcode.call_reg);
            try self.bytecode.push(src);
            return;
        },
        .identifier => |src| {
            for (self.externs.items) |ex| {
                if (mem.eql(u8, src, ex)) {
                    try self.bytecode.push(Opcode.call_ex);
                    try self.bytecode.extend(src);
                    try self.bytecode.push(0x00);
                    return;
                }
            }

            try self.bytecode.push(Opcode.call_imm);
            const offset = self.bytecode.len(self.bytecode.current_section);
            try self.fixups.put(
                .{ .section = self.bytecode.current_section, .addr = offset },
                .{ .size = .qword, .label = src, .span = span },
            );
            try self.bytecode.extend(&mem.toBytes(@as(u64, 0x00)));
            return;
        },
        else => {},
    }

    return self.reportError("unsupported operand", span);
}

fn compileIncOrDec(
    self: *Compiler,
    expr: *ast.Expression,
    op: enum { inc, dec },
    span: Span,
) !void {
    switch (expr.*) {
        .register => |src| {
            try self.bytecode.push(switch (op) {
                .inc => Opcode.inc,
                .dec => Opcode.dec,
            });
            try self.bytecode.push(src);
            return;
        },
        else => {},
    }

    return self.reportError("unsupported operand", span);
}

fn report(
    self: *Compiler,
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

fn reportError(self: *Compiler, message: []const u8, span: Span) !void {
    self.report(.err, message, span, 1);
    return error.CompilerError;
}
