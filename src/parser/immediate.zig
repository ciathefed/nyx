const std = @import("std");
const mem = std.mem;
const ascii = std.ascii;
const Register = @import("../vm/register.zig").Register;

pub const DataSize = enum {
    byte,
    word,
    dword,
    qword,
    float,
    double,

    pub fn sizeInBytes(self: DataSize) usize {
        return switch (self) {
            .byte => 1,
            .word => 2,
            .dword => 4,
            .qword => 8,
            .float => 4,
            .double => 8,
        };
    }

    pub fn fromRegister(value: Register) DataSize {
        return switch (value) {
            .b0, .b1, .b2, .b3, .b4, .b5, .b6, .b7, .b8, .b9, .b10, .b11, .b12, .b13, .b14, .b15 => .byte,
            .w0, .w1, .w2, .w3, .w4, .w5, .w6, .w7, .w8, .w9, .w10, .w11, .w12, .w13, .w14, .w15 => .word,
            .d0, .d1, .d2, .d3, .d4, .d5, .d6, .d7, .d8, .d9, .d10, .d11, .d12, .d13, .d14, .d15 => .dword,
            .q0, .q1, .q2, .q3, .q4, .q5, .q6, .q7, .q8, .q9, .q10, .q11, .q12, .q13, .q14, .q15, .ip, .sp, .bp => .qword,
            .ff0, .ff1, .ff2, .ff3, .ff4, .ff5, .ff6, .ff7, .ff8, .ff9, .ff10, .ff11, .ff12, .ff13, .ff14, .ff15 => .float,
            .dd0, .dd1, .dd2, .dd3, .dd4, .dd5, .dd6, .dd7, .dd8, .dd9, .dd10, .dd11, .dd12, .dd13, .dd14, .dd15 => .double,
        };
    }

    pub fn fromString(value: []const u8) !DataSize {
        if (ascii.eqlIgnoreCase(value, "byte")) return .byte;
        if (ascii.eqlIgnoreCase(value, "word")) return .word;
        if (ascii.eqlIgnoreCase(value, "dword")) return .dword;
        if (ascii.eqlIgnoreCase(value, "qword")) return .qword;
        if (ascii.eqlIgnoreCase(value, "float")) return .float;
        if (ascii.eqlIgnoreCase(value, "double")) return .double;
        return error.InvalidDataSize;
    }

    pub fn fromU8(value: u8) !DataSize {
        return switch (value) {
            0x00 => .byte,
            0x01 => .word,
            0x02 => .dword,
            0x03 => .qword,
            0x04 => .float,
            0x05 => .double,
            else => error.UnknownDataSize,
        };
    }

    pub fn toU8(self: DataSize) u8 {
        return @intFromEnum(self);
    }
};

pub const Immediate = union(enum) {
    byte: u8,
    word: u16,
    dword: u32,
    qword: u64,
    float: f32,
    double: f64,

    pub fn asU8(self: Immediate) u8 {
        return switch (self) {
            .byte => |v| v,
            .word => |v| @truncate(v),
            .dword => |v| @truncate(v),
            .qword => |v| @truncate(v),
            .float => |v| @truncate(@as(u8, @intFromFloat(v))),
            .double => |v| @truncate(@as(u8, @intFromFloat(v))),
        };
    }

    pub fn asU16(self: Immediate) u16 {
        return switch (self) {
            .byte => |v| @intCast(v),
            .word => |v| v,
            .dword => |v| @truncate(v),
            .qword => |v| @truncate(v),
            .float => |v| @truncate(@as(u16, @intFromFloat(v))),
            .double => |v| @truncate(@as(u16, @intFromFloat(v))),
        };
    }

    pub fn asU32(self: Immediate) u32 {
        return switch (self) {
            .byte => |v| @intCast(v),
            .word => |v| @intCast(v),
            .dword => |v| v,
            .qword => |v| @truncate(v),
            .float => |v| @truncate(@as(u32, @intFromFloat(v))),
            .double => |v| @truncate(@as(u32, @intFromFloat(v))),
        };
    }

    pub fn asU64(self: Immediate) u64 {
        return switch (self) {
            .byte => |v| @intCast(v),
            .word => |v| @intCast(v),
            .dword => |v| @intCast(v),
            .qword => |v| v,
            .float => |v| @truncate(@as(u64, @intFromFloat(v))),
            .double => |v| @truncate(@as(u64, @intFromFloat(v))),
        };
    }

    pub fn asF32(self: Immediate) f32 {
        return switch (self) {
            .byte => |v| @floatFromInt(v),
            .word => |v| @floatFromInt(v),
            .dword => |v| @floatFromInt(v),
            .qword => |v| @floatFromInt(v),
            .float => |v| v,
            .double => |v| @floatCast(v),
        };
    }

    pub fn asF64(self: Immediate) f64 {
        return switch (self) {
            .byte => |v| @floatFromInt(v),
            .word => |v| @floatFromInt(v),
            .dword => |v| @floatFromInt(v),
            .qword => |v| @floatFromInt(v),
            .float => |v| @floatCast(v),
            .double => |v| v,
        };
    }

    pub fn asUsize(self: Immediate) usize {
        return switch (self) {
            .byte => |v| @intCast(v),
            .word => |v| @intCast(v),
            .dword => |v| @intCast(v),
            .qword => |v| @intCast(v),
            .float => |v| @intFromFloat(v),
            .double => |v| @intFromFloat(v),
        };
    }

    pub fn size(self: Immediate) DataSize {
        return switch (self) {
            .byte => .byte,
            .word => .word,
            .dword => .dword,
            .qword => .qword,
            .float => .float,
            .double => .double,
        };
    }

    pub fn eql(self: Immediate, other: Immediate) bool {
        return switch (self) {
            .byte => |v| other == .byte and v == other.byte,
            .word => |v| other == .word and v == other.word,
            .dword => |v| other == .dword and v == other.dword,
            .qword => |v| other == .qword and v == other.qword,
            .float => |v| other == .float and v == other.float,
            .double => |v| other == .double and v == other.double,
        };
    }

    pub fn lessThan(self: Immediate, other: Immediate) bool {
        return switch (self) {
            .byte => |v| other == .byte and v < other.byte,
            .word => |v| other == .word and v < other.word,
            .dword => |v| other == .dword and v < other.dword,
            .qword => |v| other == .qword and v < other.qword,
            .float => |v| other == .float and v < other.float,
            .double => |v| other == .double and v < other.double,
        };
    }
};
