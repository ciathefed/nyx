const std = @import("std");
const mem = std.mem;
const Immediate = @import("../parser/immediate.zig").Immediate;

const gpr0: usize = 0x00;
const gpr1: usize = 0x01;
const gpr2: usize = 0x02;
const gpr3: usize = 0x03;
const gpr4: usize = 0x04;
const gpr5: usize = 0x05;
const gpr6: usize = 0x06;
const gpr7: usize = 0x07;
const gpr8: usize = 0x08;
const gpr9: usize = 0x09;
const gpr10: usize = 0x0A;
const gpr11: usize = 0x0B;
const gpr12: usize = 0x0C;
const gpr13: usize = 0x0D;
const gpr14: usize = 0x0E;
const gpr15: usize = 0x0F;

const fpr0: usize = 0x00;
const fpr1: usize = 0x01;
const fpr2: usize = 0x02;
const fpr3: usize = 0x03;
const fpr4: usize = 0x04;
const fpr5: usize = 0x05;
const fpr6: usize = 0x06;
const fpr7: usize = 0x07;
const fpr8: usize = 0x08;
const fpr9: usize = 0x09;
const fpr10: usize = 0x0A;
const fpr11: usize = 0x0B;
const fpr12: usize = 0x0C;
const fpr13: usize = 0x0D;
const fpr14: usize = 0x0E;
const fpr15: usize = 0x0F;

const dpr0: usize = 0x10;
const dpr1: usize = 0x11;
const dpr2: usize = 0x12;
const dpr3: usize = 0x13;
const dpr4: usize = 0x14;
const dpr5: usize = 0x15;
const dpr6: usize = 0x16;
const dpr7: usize = 0x17;
const dpr8: usize = 0x18;
const dpr9: usize = 0x19;
const dpr10: usize = 0x1A;
const dpr11: usize = 0x1B;
const dpr12: usize = 0x1C;
const dpr13: usize = 0x1D;
const dpr14: usize = 0x1E;
const dpr15: usize = 0x1F;

const ip_reg: usize = 0;
const sp_reg: usize = 1;
const bp_reg: usize = 2;

pub const Register = enum {
    b0,
    w0,
    d0,
    q0,
    ff0,
    dd0,
    b1,
    w1,
    d1,
    q1,
    ff1,
    dd1,
    b2,
    w2,
    d2,
    q2,
    ff2,
    dd2,
    b3,
    w3,
    d3,
    q3,
    ff3,
    dd3,
    b4,
    w4,
    d4,
    q4,
    ff4,
    dd4,
    b5,
    w5,
    d5,
    q5,
    ff5,
    dd5,
    b6,
    w6,
    d6,
    q6,
    ff6,
    dd6,
    b7,
    w7,
    d7,
    q7,
    ff7,
    dd7,
    b8,
    w8,
    d8,
    q8,
    ff8,
    dd8,
    b9,
    w9,
    d9,
    q9,
    ff9,
    dd9,
    b10,
    w10,
    d10,
    q10,
    ff10,
    dd10,
    b11,
    w11,
    d11,
    q11,
    ff11,
    dd11,
    b12,
    w12,
    d12,
    q12,
    ff12,
    dd12,
    b13,
    w13,
    d13,
    q13,
    ff13,
    dd13,
    b14,
    w14,
    d14,
    q14,
    ff14,
    dd14,
    b15,
    w15,
    d15,
    q15,
    ff15,
    dd15,
    ip,
    sp,
    bp,

    pub fn fromString(value: []const u8) !Register {
        if (mem.eql(u8, value, "b0")) return .b0;
        if (mem.eql(u8, value, "w0")) return .w0;
        if (mem.eql(u8, value, "d0")) return .d0;
        if (mem.eql(u8, value, "q0")) return .q0;
        if (mem.eql(u8, value, "ff0")) return .ff0;
        if (mem.eql(u8, value, "dd0")) return .dd0;

        if (mem.eql(u8, value, "b1")) return .b1;
        if (mem.eql(u8, value, "w1")) return .w1;
        if (mem.eql(u8, value, "d1")) return .d1;
        if (mem.eql(u8, value, "q1")) return .q1;
        if (mem.eql(u8, value, "ff1")) return .ff1;
        if (mem.eql(u8, value, "dd1")) return .dd1;

        if (mem.eql(u8, value, "b2")) return .b2;
        if (mem.eql(u8, value, "w2")) return .w2;
        if (mem.eql(u8, value, "d2")) return .d2;
        if (mem.eql(u8, value, "q2")) return .q2;
        if (mem.eql(u8, value, "ff2")) return .ff2;
        if (mem.eql(u8, value, "dd2")) return .dd2;

        if (mem.eql(u8, value, "b3")) return .b3;
        if (mem.eql(u8, value, "w3")) return .w3;
        if (mem.eql(u8, value, "d3")) return .d3;
        if (mem.eql(u8, value, "q3")) return .q3;
        if (mem.eql(u8, value, "ff3")) return .ff3;
        if (mem.eql(u8, value, "dd3")) return .dd3;

        if (mem.eql(u8, value, "b4")) return .b4;
        if (mem.eql(u8, value, "w4")) return .w4;
        if (mem.eql(u8, value, "d4")) return .d4;
        if (mem.eql(u8, value, "q4")) return .q4;
        if (mem.eql(u8, value, "ff4")) return .ff4;
        if (mem.eql(u8, value, "dd4")) return .dd4;

        if (mem.eql(u8, value, "b5")) return .b5;
        if (mem.eql(u8, value, "w5")) return .w5;
        if (mem.eql(u8, value, "d5")) return .d5;
        if (mem.eql(u8, value, "q5")) return .q5;
        if (mem.eql(u8, value, "ff5")) return .ff5;
        if (mem.eql(u8, value, "dd5")) return .dd5;

        if (mem.eql(u8, value, "b6")) return .b6;
        if (mem.eql(u8, value, "w6")) return .w6;
        if (mem.eql(u8, value, "d6")) return .d6;
        if (mem.eql(u8, value, "q6")) return .q6;
        if (mem.eql(u8, value, "ff6")) return .ff6;
        if (mem.eql(u8, value, "dd6")) return .dd6;

        if (mem.eql(u8, value, "b7")) return .b7;
        if (mem.eql(u8, value, "w7")) return .w7;
        if (mem.eql(u8, value, "d7")) return .d7;
        if (mem.eql(u8, value, "q7")) return .q7;
        if (mem.eql(u8, value, "ff7")) return .ff7;
        if (mem.eql(u8, value, "dd7")) return .dd7;

        if (mem.eql(u8, value, "b8")) return .b8;
        if (mem.eql(u8, value, "w8")) return .w8;
        if (mem.eql(u8, value, "d8")) return .d8;
        if (mem.eql(u8, value, "q8")) return .q8;
        if (mem.eql(u8, value, "ff8")) return .ff8;
        if (mem.eql(u8, value, "dd8")) return .dd8;

        if (mem.eql(u8, value, "b9")) return .b9;
        if (mem.eql(u8, value, "w9")) return .w9;
        if (mem.eql(u8, value, "d9")) return .d9;
        if (mem.eql(u8, value, "q9")) return .q9;
        if (mem.eql(u8, value, "ff9")) return .ff9;
        if (mem.eql(u8, value, "dd9")) return .dd9;

        if (mem.eql(u8, value, "b10")) return .b10;
        if (mem.eql(u8, value, "w10")) return .w10;
        if (mem.eql(u8, value, "d10")) return .d10;
        if (mem.eql(u8, value, "q10")) return .q10;
        if (mem.eql(u8, value, "ff10")) return .ff10;
        if (mem.eql(u8, value, "dd10")) return .dd10;

        if (mem.eql(u8, value, "b11")) return .b11;
        if (mem.eql(u8, value, "w11")) return .w11;
        if (mem.eql(u8, value, "d11")) return .d11;
        if (mem.eql(u8, value, "q11")) return .q11;
        if (mem.eql(u8, value, "ff11")) return .ff11;
        if (mem.eql(u8, value, "dd11")) return .dd11;

        if (mem.eql(u8, value, "b12")) return .b12;
        if (mem.eql(u8, value, "w12")) return .w12;
        if (mem.eql(u8, value, "d12")) return .d12;
        if (mem.eql(u8, value, "q12")) return .q12;
        if (mem.eql(u8, value, "ff12")) return .ff12;
        if (mem.eql(u8, value, "dd12")) return .dd12;

        if (mem.eql(u8, value, "b13")) return .b13;
        if (mem.eql(u8, value, "w13")) return .w13;
        if (mem.eql(u8, value, "d13")) return .d13;
        if (mem.eql(u8, value, "q13")) return .q13;
        if (mem.eql(u8, value, "ff13")) return .ff13;
        if (mem.eql(u8, value, "dd13")) return .dd13;

        if (mem.eql(u8, value, "b14")) return .b14;
        if (mem.eql(u8, value, "w14")) return .w14;
        if (mem.eql(u8, value, "d14")) return .d14;
        if (mem.eql(u8, value, "q14")) return .q14;
        if (mem.eql(u8, value, "ff14")) return .ff14;
        if (mem.eql(u8, value, "dd14")) return .dd14;

        if (mem.eql(u8, value, "b15")) return .b15;
        if (mem.eql(u8, value, "w15")) return .w15;
        if (mem.eql(u8, value, "d15")) return .d15;
        if (mem.eql(u8, value, "q15")) return .q15;
        if (mem.eql(u8, value, "ff15")) return .ff15;
        if (mem.eql(u8, value, "dd15")) return .dd15;

        if (mem.eql(u8, value, "ip")) return .ip;
        if (mem.eql(u8, value, "sp")) return .sp;
        if (mem.eql(u8, value, "bp")) return .bp;

        return error.InvalidRegister;
    }

    pub fn fromU8(value: u8) !Register {
        if (value > @intFromEnum(Register.bp)) {
            return error.InvalidRegister;
        }
        return @enumFromInt(value);
    }

    pub fn physicalInfo(self: Register) PhysicalInfo {
        return switch (self) {
            .b0 => PhysicalInfo.init(.general_purpose, gpr0, .byte),
            .w0 => PhysicalInfo.init(.general_purpose, gpr0, .word),
            .d0 => PhysicalInfo.init(.general_purpose, gpr0, .dword),
            .q0 => PhysicalInfo.init(.general_purpose, gpr0, .qword),
            .ff0 => PhysicalInfo.init(.floating_point, fpr0, .float),
            .dd0 => PhysicalInfo.init(.floating_point, dpr0, .double),

            .b1 => PhysicalInfo.init(.general_purpose, gpr1, .byte),
            .w1 => PhysicalInfo.init(.general_purpose, gpr1, .word),
            .d1 => PhysicalInfo.init(.general_purpose, gpr1, .dword),
            .q1 => PhysicalInfo.init(.general_purpose, gpr1, .qword),
            .ff1 => PhysicalInfo.init(.floating_point, fpr1, .float),
            .dd1 => PhysicalInfo.init(.floating_point, dpr1, .double),

            .b2 => PhysicalInfo.init(.general_purpose, gpr2, .byte),
            .w2 => PhysicalInfo.init(.general_purpose, gpr2, .word),
            .d2 => PhysicalInfo.init(.general_purpose, gpr2, .dword),
            .q2 => PhysicalInfo.init(.general_purpose, gpr2, .qword),
            .ff2 => PhysicalInfo.init(.floating_point, fpr2, .float),
            .dd2 => PhysicalInfo.init(.floating_point, dpr2, .double),

            .b3 => PhysicalInfo.init(.general_purpose, gpr3, .byte),
            .w3 => PhysicalInfo.init(.general_purpose, gpr3, .word),
            .d3 => PhysicalInfo.init(.general_purpose, gpr3, .dword),
            .q3 => PhysicalInfo.init(.general_purpose, gpr3, .qword),
            .ff3 => PhysicalInfo.init(.floating_point, fpr3, .float),
            .dd3 => PhysicalInfo.init(.floating_point, dpr3, .double),

            .b4 => PhysicalInfo.init(.general_purpose, gpr4, .byte),
            .w4 => PhysicalInfo.init(.general_purpose, gpr4, .word),
            .d4 => PhysicalInfo.init(.general_purpose, gpr4, .dword),
            .q4 => PhysicalInfo.init(.general_purpose, gpr4, .qword),
            .ff4 => PhysicalInfo.init(.floating_point, fpr4, .float),
            .dd4 => PhysicalInfo.init(.floating_point, dpr4, .double),

            .b5 => PhysicalInfo.init(.general_purpose, gpr5, .byte),
            .w5 => PhysicalInfo.init(.general_purpose, gpr5, .word),
            .d5 => PhysicalInfo.init(.general_purpose, gpr5, .dword),
            .q5 => PhysicalInfo.init(.general_purpose, gpr5, .qword),
            .ff5 => PhysicalInfo.init(.floating_point, fpr5, .float),
            .dd5 => PhysicalInfo.init(.floating_point, dpr5, .double),

            .b6 => PhysicalInfo.init(.general_purpose, gpr6, .byte),
            .w6 => PhysicalInfo.init(.general_purpose, gpr6, .word),
            .d6 => PhysicalInfo.init(.general_purpose, gpr6, .dword),
            .q6 => PhysicalInfo.init(.general_purpose, gpr6, .qword),
            .ff6 => PhysicalInfo.init(.floating_point, fpr6, .float),
            .dd6 => PhysicalInfo.init(.floating_point, dpr6, .double),

            .b7 => PhysicalInfo.init(.general_purpose, gpr7, .byte),
            .w7 => PhysicalInfo.init(.general_purpose, gpr7, .word),
            .d7 => PhysicalInfo.init(.general_purpose, gpr7, .dword),
            .q7 => PhysicalInfo.init(.general_purpose, gpr7, .qword),
            .ff7 => PhysicalInfo.init(.floating_point, fpr7, .float),
            .dd7 => PhysicalInfo.init(.floating_point, dpr7, .double),

            .b8 => PhysicalInfo.init(.general_purpose, gpr8, .byte),
            .w8 => PhysicalInfo.init(.general_purpose, gpr8, .word),
            .d8 => PhysicalInfo.init(.general_purpose, gpr8, .dword),
            .q8 => PhysicalInfo.init(.general_purpose, gpr8, .qword),
            .ff8 => PhysicalInfo.init(.floating_point, fpr8, .float),
            .dd8 => PhysicalInfo.init(.floating_point, dpr8, .double),

            .b9 => PhysicalInfo.init(.general_purpose, gpr9, .byte),
            .w9 => PhysicalInfo.init(.general_purpose, gpr9, .word),
            .d9 => PhysicalInfo.init(.general_purpose, gpr9, .dword),
            .q9 => PhysicalInfo.init(.general_purpose, gpr9, .qword),
            .ff9 => PhysicalInfo.init(.floating_point, fpr9, .float),
            .dd9 => PhysicalInfo.init(.floating_point, dpr9, .double),

            .b10 => PhysicalInfo.init(.general_purpose, gpr10, .byte),
            .w10 => PhysicalInfo.init(.general_purpose, gpr10, .word),
            .d10 => PhysicalInfo.init(.general_purpose, gpr10, .dword),
            .q10 => PhysicalInfo.init(.general_purpose, gpr10, .qword),
            .ff10 => PhysicalInfo.init(.floating_point, fpr10, .float),
            .dd10 => PhysicalInfo.init(.floating_point, dpr10, .double),

            .b11 => PhysicalInfo.init(.general_purpose, gpr11, .byte),
            .w11 => PhysicalInfo.init(.general_purpose, gpr11, .word),
            .d11 => PhysicalInfo.init(.general_purpose, gpr11, .dword),
            .q11 => PhysicalInfo.init(.general_purpose, gpr11, .qword),
            .ff11 => PhysicalInfo.init(.floating_point, fpr11, .float),
            .dd11 => PhysicalInfo.init(.floating_point, dpr11, .double),

            .b12 => PhysicalInfo.init(.general_purpose, gpr12, .byte),
            .w12 => PhysicalInfo.init(.general_purpose, gpr12, .word),
            .d12 => PhysicalInfo.init(.general_purpose, gpr12, .dword),
            .q12 => PhysicalInfo.init(.general_purpose, gpr12, .qword),
            .ff12 => PhysicalInfo.init(.floating_point, fpr12, .float),
            .dd12 => PhysicalInfo.init(.floating_point, dpr12, .double),

            .b13 => PhysicalInfo.init(.general_purpose, gpr13, .byte),
            .w13 => PhysicalInfo.init(.general_purpose, gpr13, .word),
            .d13 => PhysicalInfo.init(.general_purpose, gpr13, .dword),
            .q13 => PhysicalInfo.init(.general_purpose, gpr13, .qword),
            .ff13 => PhysicalInfo.init(.floating_point, fpr13, .float),
            .dd13 => PhysicalInfo.init(.floating_point, dpr13, .double),

            .b14 => PhysicalInfo.init(.general_purpose, gpr14, .byte),
            .w14 => PhysicalInfo.init(.general_purpose, gpr14, .word),
            .d14 => PhysicalInfo.init(.general_purpose, gpr14, .dword),
            .q14 => PhysicalInfo.init(.general_purpose, gpr14, .qword),
            .ff14 => PhysicalInfo.init(.floating_point, fpr14, .float),
            .dd14 => PhysicalInfo.init(.floating_point, dpr14, .double),

            .b15 => PhysicalInfo.init(.general_purpose, gpr15, .byte),
            .w15 => PhysicalInfo.init(.general_purpose, gpr15, .word),
            .d15 => PhysicalInfo.init(.general_purpose, gpr15, .dword),
            .q15 => PhysicalInfo.init(.general_purpose, gpr15, .qword),
            .ff15 => PhysicalInfo.init(.floating_point, fpr15, .float),
            .dd15 => PhysicalInfo.init(.floating_point, dpr15, .double),

            .ip => PhysicalInfo.init(.special, ip_reg, .qword),
            .sp => PhysicalInfo.init(.special, sp_reg, .qword),
            .bp => PhysicalInfo.init(.special, bp_reg, .qword),
        };
    }
};

const PhysicalInfo = struct {
    type: PhysicalRegisterType,
    index: usize,
    view: RegisterView,

    pub fn init(t: PhysicalRegisterType, i: usize, v: RegisterView) PhysicalInfo {
        return PhysicalInfo{
            .type = t,
            .index = i,
            .view = v,
        };
    }
};

const PhysicalRegisterType = enum {
    general_purpose,
    floating_point,
    special,
};

const RegisterView = enum {
    byte,
    word,
    dword,
    qword,
    float,
    double,
};

pub const Registers = struct {
    gpr: [16]u64,
    fpr: [32]u64,
    special: [3]usize,

    pub fn init() Registers {
        return Registers{
            .gpr = mem.zeroes([16]u64),
            .fpr = mem.zeroes([32]u64),
            .special = mem.zeroes([3]usize),
        };
    }

    pub fn get(self: *Registers, reg: Register) Immediate {
        const info = reg.physicalInfo();

        switch (info.type) {
            .general_purpose => {
                const value = self.gpr[info.index];
                return switch (info.view) {
                    .byte => .{ .byte = @intCast(value) },
                    .word => .{ .word = @intCast(value) },
                    .dword => .{ .dword = @intCast(value) },
                    .qword => .{ .qword = value },
                    else => unreachable,
                };
            },
            .floating_point => {
                const value = self.fpr[info.index];
                return switch (info.view) {
                    .float => .{ .float = @floatFromInt(value) },
                    .double => .{ .double = @floatFromInt(value) },
                    else => unreachable,
                };
            },
            .special => {
                const value = self.special[info.index];
                return switch (info.view) {
                    .qword => .{ .qword = @intCast(value) },
                    else => unreachable,
                };
            },
        }
    }

    pub fn set(self: *Registers, reg: Register, imm: Immediate) void {
        const info = reg.physicalInfo();

        switch (info.type) {
            .general_purpose => switch (info.view) {
                .byte => {
                    const new_value = imm.asU8();
                    self.gpr[info.index] = (self.gpr[info.index] & 0xFFFFFFFFFFFFFF00) | (@as(u64, @intCast(new_value)));
                },
                .word => {
                    const new_value = imm.asU16();
                    self.gpr[info.index] = (self.gpr[info.index] & 0xFFFFFFFFFFFF0000) | (@as(u64, @intCast(new_value)));
                },
                .dword => {
                    const new_value = imm.asU32();
                    self.gpr[info.index] = @intCast(new_value);
                },
                .qword => {
                    const new_value = imm.asU64();
                    self.gpr[info.index] = new_value;
                },
                else => unreachable,
            },
            .floating_point => switch (info.view) {
                .float => {
                    const new_value = imm.asF32();
                    self.fpr[info.index] = @intFromFloat(new_value);
                },
                .double => {
                    const new_value = imm.asF64();
                    self.fpr[info.index] = @intFromFloat(new_value);
                },
                else => unreachable,
            },
            .special => switch (info.view) {
                .qword => {
                    const new_value = imm.asUsize();
                    self.special[info.index] = new_value;
                },
                else => unreachable,
            },
        }
    }

    pub fn ip(self: *Registers) usize {
        return self.special[ip_reg];
    }

    pub fn setIp(self: *Registers, val: usize) void {
        self.special[ip_reg] = val;
    }

    pub fn sp(self: *Registers) usize {
        return self.special[sp_reg];
    }

    pub fn setSp(self: *Registers, val: usize) void {
        self.special[sp_reg] = val;
    }

    pub fn bp(self: *Registers) usize {
        return self.special[bp_reg];
    }

    pub fn setBp(self: *Registers, val: usize) void {
        self.special[bp_reg] = val;
    }
};
