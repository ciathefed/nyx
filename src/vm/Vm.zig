const std = @import("std");
const mem = std.mem;
const Allocator = mem.Allocator;
const Registers = @import("register.zig").Registers;
const Register = @import("register.zig").Register;
const DataSize = @import("../parser/immediate.zig").DataSize;
const Immediate = @import("../parser/immediate.zig").Immediate;
const Mmu = @import("memory/Mmu.zig");
const Block = @import("memory/Block.zig");
const Flags = @import("Flags.zig");
const syscall = @import("syscall.zig");
const ExternalLoader = @import("ExternalLoader.zig");
const Opcode = @import("../compiler/opcode.zig").Opcode;
const addressing_variant_1 = @import("../compiler/Compiler.zig").addressing_variant_1;
const addressing_variant_2 = @import("../compiler/Compiler.zig").addressing_variant_2;

const Vm = @This();

regs: Registers,
mmu: Mmu,
flags: Flags,
syscalls: syscall.Syscalls,
external_loader: ExternalLoader,
halted: bool,

pub fn init(program: []const u8, mem_size: usize, allocator: Allocator) !Vm {
    if (program.len < 8) return error.ProgramTooSmall;
    if (program.len >= mem_size) return error.ProgramTooLarge;

    const entry_point: usize = @intCast(mem.readInt(u64, program[0..8], .little));
    if (entry_point >= program.len) return error.InvalidEntryPoint;

    const program_data = program[8..];

    var regs = Registers.init();
    regs.setSp(mem_size);
    regs.setBp(0);
    regs.setIp(entry_point);

    var mmu = Mmu.init(allocator);
    errdefer mmu.deinit();

    _ = try mmu.addBlock("Program", program_data.len);
    _ = try mmu.addBlock("Memory", mem_size - program_data.len);
    try mmu.writeSlice(0x00, program_data);

    return Vm{
        .regs = regs,
        .mmu = mmu,
        .flags = .init(),
        .syscalls = try syscall.collectSyscalls(allocator),
        .external_loader = .init(allocator),
        .halted = false,
    };
}

pub fn deinit(self: *Vm) void {
    self.mmu.deinit();
    self.syscalls.deinit();
    self.external_loader.deinit();
}

pub fn step(self: *Vm) !void {
    if (self.halted) return;

    const byte = try self.readByte();
    if (byte > @as(u8, @intFromEnum(Opcode.hlt))) return error.InvalidOpcode;
    const opcode: Opcode = @enumFromInt(byte);

    switch (opcode) {
        .nop => {},
        .load_external => {
            const path = try self.readString();
            try self.external_loader.load(path);
        },
        .mov_reg_reg => {
            const dest = try self.readRegister();
            const src = try self.readRegister();
            self.regs.set(dest, self.regs.get(src));
        },
        .mov_reg_imm => {
            const dest = try self.readRegister();
            const src: Immediate = switch (DataSize.fromRegister(dest)) {
                .byte => .{ .byte = try self.readByte() },
                .word => .{ .word = try self.readWord() },
                .dword => .{ .dword = try self.readDword() },
                .qword => .{ .qword = try self.readQword() },
                .float => .{ .float = try self.readFloat() },
                .double => .{ .double = try self.readDouble() },
            };
            self.regs.set(dest, src);
        },
        .ldr => {
            const dest = try self.readRegister();
            const variant = try self.readByte();
            const base = switch (variant) {
                addressing_variant_1 => self.regs.get(try self.readRegister()).asU64(),
                addressing_variant_2 => try self.readQword(),
                else => return error.UnknownAddressingVariant,
            };
            const offset = try self.readQword();
            const addr: usize = @intCast(base + offset);
            const imm = try self.mmu.read(addr, DataSize.fromRegister(dest));
            self.regs.set(dest, imm);
        },
        .str => {
            const src = try self.readRegister();
            const value = self.regs.get(src);
            const variant = try self.readByte();
            const base = switch (variant) {
                addressing_variant_1 => self.regs.get(try self.readRegister()).asU64(),
                addressing_variant_2 => try self.readQword(),
                else => return error.UnknownAddressingVariant,
            };
            const offset = try self.readQword();
            const addr: usize = @intCast(base + offset);
            try self.mmu.write(addr, value, DataSize.fromRegister(src));
        },
        .sti => {
            const size = try self.readDataSize();
            const value: Immediate = switch (size) {
                .byte => .{ .byte = try self.readByte() },
                .word => .{ .word = try self.readWord() },
                .dword => .{ .dword = try self.readDword() },
                .qword => .{ .qword = try self.readQword() },
                .float => .{ .float = try self.readFloat() },
                .double => .{ .double = try self.readDouble() },
            };
            const variant = try self.readByte();
            const base = switch (variant) {
                addressing_variant_1 => self.regs.get(try self.readRegister()).asU64(),
                addressing_variant_2 => try self.readQword(),
                else => return error.UnknownAddressingVariant,
            };
            const offset = try self.readQword();
            const addr: usize = @intCast(base + offset);
            try self.mmu.write(addr, value, size);
        },
        .push_imm => {
            const size = try self.readDataSize();
            const imm: Immediate = switch (size) {
                .byte => .{ .byte = try self.readByte() },
                .word => .{ .word = try self.readWord() },
                .dword => .{ .dword = try self.readDword() },
                .qword => .{ .qword = try self.readQword() },
                .float => .{ .float = try self.readFloat() },
                .double => .{ .double = try self.readDouble() },
            };
            try self.push(imm);
        },
        .push_reg => {
            const size = try self.readDataSize();
            const src = try self.readRegister();
            const imm: Immediate = switch (size) {
                .byte => .{ .byte = self.regs.get(src).asU8() },
                .word => .{ .word = self.regs.get(src).asU16() },
                .dword => .{ .dword = self.regs.get(src).asU32() },
                .qword => .{ .qword = self.regs.get(src).asU64() },
                .float => .{ .float = self.regs.get(src).asF32() },
                .double => .{ .double = self.regs.get(src).asF64() },
            };
            try self.push(imm);
        },
        .push_addr => {
            const size = try self.readDataSize();
            const variant = try self.readByte();
            const base = switch (variant) {
                addressing_variant_1 => self.regs.get(try self.readRegister()).asU64(),
                addressing_variant_2 => try self.readQword(),
                else => return error.UnknownAddressingVariant,
            };
            const offset = try self.readQword();
            const addr: usize = @intCast(base + offset);
            const value = try self.mmu.read(addr, size);
            try self.push(value);
        },
        .pop_reg => {
            const size = try self.readDataSize();
            const dest = try self.readRegister();
            const value = try self.pop(size);
            self.regs.set(dest, value);
        },
        .pop_addr => {
            const size = try self.readDataSize();
            const variant = try self.readByte();
            const base = switch (variant) {
                addressing_variant_1 => self.regs.get(try self.readRegister()).asU64(),
                addressing_variant_2 => try self.readQword(),
                else => return error.UnknownAddressingVariant,
            };
            const offset = try self.readQword();
            const addr: usize = @intCast(base + offset);
            const value = try self.pop(size);
            try self.mmu.write(addr, value, size);
        },
        .add_reg_reg_reg => try self.executeBinaryOp(add, true),
        .add_reg_reg_imm => try self.executeBinaryOp(add, false),
        .sub_reg_reg_reg => try self.executeBinaryOp(sub, true),
        .sub_reg_reg_imm => try self.executeBinaryOp(sub, false),
        .mul_reg_reg_reg => try self.executeBinaryOp(mul, true),
        .mul_reg_reg_imm => try self.executeBinaryOp(mul, false),
        .div_reg_reg_reg => try self.executeBinaryOp(div, true),
        .div_reg_reg_imm => try self.executeBinaryOp(div, false),
        .and_reg_reg_reg => try self.executeBitwiseOp(bitAnd, true),
        .and_reg_reg_imm => try self.executeBitwiseOp(bitAnd, false),
        .or_reg_reg_reg => try self.executeBitwiseOp(bitOr, true),
        .or_reg_reg_imm => try self.executeBitwiseOp(bitOr, false),
        .xor_reg_reg_reg => try self.executeBitwiseOp(bitXor, true),
        .xor_reg_reg_imm => try self.executeBitwiseOp(bitXor, false),
        .shl_reg_reg_reg => try self.executeBitwiseOp(shl, true),
        .shl_reg_reg_imm => try self.executeBitwiseOp(shl, false),
        .shr_reg_reg_reg => try self.executeBitwiseOp(shr, true),
        .shr_reg_reg_imm => try self.executeBitwiseOp(shr, false),
        .cmp_reg_imm => {
            const reg = try self.readRegister();
            const lhs = self.regs.get(reg);
            const rhs: Immediate = switch (DataSize.fromRegister(reg)) {
                .byte => .{ .byte = try self.readByte() },
                .word => .{ .word = try self.readWord() },
                .dword => .{ .dword = try self.readDword() },
                .qword => .{ .qword = try self.readQword() },
                .float => .{ .float = try self.readFloat() },
                .double => .{ .double = try self.readDouble() },
            };
            self.flags.eq = lhs.eql(rhs);
            self.flags.lt = lhs.lessThan(rhs);
        },
        .cmp_reg_reg => {
            const lhs = self.regs.get(try self.readRegister());
            const rhs = self.regs.get(try self.readRegister());
            self.flags.eq = lhs.eql(rhs);
            self.flags.lt = lhs.lessThan(rhs);
        },
        .jmp_imm => {
            const addr: usize = try self.readQword();
            self.regs.setIp(addr);
        },
        .jmp_reg => {
            const addr = self.regs.get(try self.readRegister()).asUsize();
            self.regs.setIp(addr);
        },
        .jeq_imm => {
            const addr: usize = try self.readQword();
            if (self.flags.eq) self.regs.setIp(addr);
        },
        .jeq_reg => {
            const addr = self.regs.get(try self.readRegister()).asUsize();
            if (self.flags.eq) self.regs.setIp(addr);
        },
        .jne_imm => {
            const addr: usize = try self.readQword();
            if (!self.flags.eq) self.regs.setIp(addr);
        },
        .jne_reg => {
            const addr = self.regs.get(try self.readRegister()).asUsize();
            if (!self.flags.eq) self.regs.setIp(addr);
        },
        .jlt_imm => {
            const addr: usize = try self.readQword();
            if (self.flags.lt) self.regs.setIp(addr);
        },
        .jlt_reg => {
            const addr = self.regs.get(try self.readRegister()).asUsize();
            if (self.flags.lt) self.regs.setIp(addr);
        },
        .jgt_imm => {
            const addr: usize = try self.readQword();
            if (!self.flags.lt) self.regs.setIp(addr);
        },
        .jgt_reg => {
            const addr = self.regs.get(try self.readRegister()).asUsize();
            if (!self.flags.lt) self.regs.setIp(addr);
        },
        .jle_imm => {
            const addr: usize = try self.readQword();
            if (self.flags.lt or self.flags.eq) self.regs.setIp(addr);
        },
        .jle_reg => {
            const addr = self.regs.get(try self.readRegister()).asUsize();
            if (self.flags.lt or self.flags.eq) self.regs.setIp(addr);
        },
        .jge_imm => {
            const addr: usize = try self.readQword();
            if (!self.flags.lt or self.flags.eq) self.regs.setIp(addr);
        },
        .jge_reg => {
            const addr = self.regs.get(try self.readRegister()).asUsize();
            if (!self.flags.lt or self.flags.eq) self.regs.setIp(addr);
        },
        .call_imm => {
            const addr = try self.readQword();
            try self.push(.{ .qword = @intCast(self.regs.ip()) });
            self.regs.setIp(@intCast(addr));
        },
        .call_reg => {
            const reg = try self.readRegister();
            const addr = self.regs.get(reg).asUsize();
            try self.push(.{ .qword = @intCast(self.regs.ip()) });
            self.regs.setIp(addr);
        },
        .call_ex => {
            const name = try self.readString();
            const func = try self.external_loader.lookup(name);
            _ = func(self);
        },
        .inc => {
            const reg = try self.readRegister();
            const value = self.regs.get(reg);
            const new_value: Immediate = switch (value) {
                .byte => |imm| .{ .byte = imm + 1 },
                .word => |imm| .{ .word = imm + 1 },
                .dword => |imm| .{ .dword = imm + 1 },
                .qword => |imm| .{ .qword = imm + 1 },
                .float => |imm| .{ .float = imm + 1.0 },
                .double => |imm| .{ .double = imm + 1.0 },
            };
            self.regs.set(reg, new_value);
        },
        .dec => {
            const reg = try self.readRegister();
            const value = self.regs.get(reg);
            const new_value: Immediate = switch (value) {
                .byte => |imm| .{ .byte = imm - 1 },
                .word => |imm| .{ .word = imm - 1 },
                .dword => |imm| .{ .dword = imm - 1 },
                .qword => |imm| .{ .qword = imm - 1 },
                .float => |imm| .{ .float = imm - 1.0 },
                .double => |imm| .{ .double = imm - 1.0 },
            };
            self.regs.set(reg, new_value);
        },
        .neg => {
            const reg = try self.readRegister();
            const value = self.regs.get(reg);
            const new_value: Immediate = switch (value) {
                .byte => |imm| .{ .byte = @intCast(-@as(i8, @intCast(imm))) },
                .word => |imm| .{ .word = @intCast(-@as(i16, @intCast(imm))) },
                .dword => |imm| .{ .dword = @intCast(-@as(i32, @intCast(imm))) },
                .qword => |imm| .{ .qword = @intCast(-@as(i64, @intCast(imm))) },
                .float => |imm| .{ .float = -imm },
                .double => |imm| .{ .double = -imm },
            };
            self.regs.set(reg, new_value);
        },
        .ret => {
            const addr = (try self.pop(.qword)).asUsize();
            self.regs.setIp(addr);
        },
        .syscall => {
            const index = self.regs.get(.q15).asUsize();
            if (self.syscalls.get(index)) |sc| {
                try sc(self);
            } else {
                return error.UnknownSyscall;
            }
        },
        .hlt => self.halted = true,
        // else => return error.UnhandledOpcode,
    }
}

pub fn run(self: *Vm) !void {
    self.regs.set(.q0, .{ .qword = 1300 });
    self.regs.set(.q1, .{ .qword = 37 });
    while (!self.halted) try self.step();
}

inline fn readByte(self: *Vm) !u8 {
    const ip = self.regs.ip();
    if (ip >= self.mmu.size()) return error.InstructionPointerOutOfBounds;
    const byte = (try self.mmu.read(ip, .byte)).asU8();
    self.regs.setIp(ip + 1);
    return byte;
}

inline fn readWord(self: *Vm) !u16 {
    const ip = self.regs.ip();
    if (ip + 2 >= self.mmu.size()) return error.InstructionPointerOutOfBounds;
    const word = (try self.mmu.read(ip, .word)).asU16();
    self.regs.setIp(ip + 2);
    return word;
}

inline fn readDword(self: *Vm) !u32 {
    const ip = self.regs.ip();
    if (ip + 4 >= self.mmu.size()) return error.InstructionPointerOutOfBounds;
    const dword = (try self.mmu.read(ip, .dword)).asU32();
    self.regs.setIp(ip + 4);
    return dword;
}

inline fn readQword(self: *Vm) !u64 {
    const ip = self.regs.ip();
    if (ip + 8 >= self.mmu.size()) return error.InstructionPointerOutOfBounds;
    const qword = (try self.mmu.read(ip, .qword)).asU64();
    self.regs.setIp(ip + 8);
    return qword;
}

inline fn readFloat(self: *Vm) !f32 {
    const ip = self.regs.ip();
    if (ip + 4 >= self.mmu.size()) return error.InstructionPointerOutOfBounds;
    const bits = (try self.mmu.read(ip, .dword)).asU32();
    const float = @as(f32, @bitCast(bits));
    self.regs.setIp(ip + 4);
    return float;
}

inline fn readDouble(self: *Vm) !f64 {
    const ip = self.regs.ip();
    if (ip + 8 >= self.mmu.size()) return error.InstructionPointerOutOfBounds;
    const bits = (try self.mmu.read(ip, .qword)).asU64();
    const double = @as(f64, @bitCast(bits));
    self.regs.setIp(ip + 8);
    return double;
}

inline fn readRegister(self: *Vm) !Register {
    const byte = try self.readByte();
    return Register.fromU8(byte);
}

inline fn readDataSize(self: *Vm) !DataSize {
    const byte = try self.readByte();
    return DataSize.fromU8(byte);
}

inline fn readString(self: *Vm) ![]const u8 {
    const string = blk: {
        const addr = self.regs.get(.ip).asUsize();
        var i: usize = 0;
        while (try self.readByte() != 0x00) i += 1;
        break :blk try self.mmu.readSlice(addr, i);
    };
    return string;
}

fn push(self: *Vm, imm: Immediate) !void {
    const size = imm.size();
    const size_bytes = size.sizeInBytes();
    const current_sp = self.regs.sp();

    if (current_sp < size_bytes) {
        return error.StackOverflow;
    }

    const new_sp = current_sp - size_bytes;
    self.regs.setSp(new_sp);
    return self.mmu.write(new_sp, imm, size);
}

fn pop(self: *Vm, size: DataSize) !Immediate {
    const current_sp = self.regs.sp();
    if (current_sp + size.sizeInBytes() > self.mmu.size()) {
        return error.StackUnderflow;
    }

    const value = try self.mmu.read(current_sp, size);
    self.regs.setSp(current_sp + size.sizeInBytes());
    return value;
}

fn executeBinaryOp(
    self: *Vm,
    comptime op: anytype,
    read_rhs_from_reg: bool,
) !void {
    const dest = try self.readRegister();
    const lhs = try self.readRegister();
    const lhs_val = self.regs.get(lhs);

    const rhs_val: Immediate = if (read_rhs_from_reg) blk: {
        const rhs = try self.readRegister();
        break :blk self.regs.get(rhs);
    } else blk: {
        break :blk switch (DataSize.fromRegister(dest)) {
            .byte => .{ .byte = try self.readByte() },
            .word => .{ .word = try self.readWord() },
            .dword => .{ .dword = try self.readDword() },
            .qword => .{ .qword = try self.readQword() },
            .float => .{ .float = try self.readFloat() },
            .double => .{ .double = try self.readDouble() },
        };
    };

    const result: Immediate = switch (DataSize.fromRegister(dest)) {
        .byte => .{ .byte = op(lhs_val.asU8(), rhs_val.asU8()) },
        .word => .{ .word = op(lhs_val.asU16(), rhs_val.asU16()) },
        .dword => .{ .dword = op(lhs_val.asU32(), rhs_val.asU32()) },
        .qword => .{ .qword = op(lhs_val.asU64(), rhs_val.asU64()) },
        .float => .{ .float = op(lhs_val.asF32(), rhs_val.asF32()) },
        .double => .{ .double = op(lhs_val.asF64(), rhs_val.asF64()) },
    };

    self.regs.set(dest, result);
}

inline fn add(a: anytype, b: anytype) @TypeOf(a, b) {
    return a + b;
}

inline fn sub(a: anytype, b: anytype) @TypeOf(a, b) {
    return a - b;
}

inline fn mul(a: anytype, b: anytype) @TypeOf(a, b) {
    return a * b;
}

inline fn div(a: anytype, b: anytype) @TypeOf(a, b) {
    return @divTrunc(a, b);
}

fn executeBitwiseOp(
    self: *Vm,
    comptime op: anytype,
    read_rhs_from_reg: bool,
) !void {
    const dest = try self.readRegister();
    const lhs = try self.readRegister();
    const lhs_val = self.regs.get(lhs);

    const rhs_val: Immediate = if (read_rhs_from_reg) blk: {
        const rhs = try self.readRegister();
        break :blk self.regs.get(rhs);
    } else blk: {
        break :blk switch (DataSize.fromRegister(dest)) {
            .byte => .{ .byte = try self.readByte() },
            .word => .{ .word = try self.readWord() },
            .dword => .{ .dword = try self.readDword() },
            .qword => .{ .qword = try self.readQword() },
            else => return error.InvalidDataSize,
        };
    };

    const result: Immediate = switch (DataSize.fromRegister(dest)) {
        .byte => .{ .byte = op(lhs_val.asU8(), rhs_val.asU8()) },
        .word => .{ .word = op(lhs_val.asU16(), rhs_val.asU16()) },
        .dword => .{ .dword = op(lhs_val.asU32(), rhs_val.asU32()) },
        .qword => .{ .qword = op(lhs_val.asU64(), rhs_val.asU64()) },
        else => return error.InvalidDataSize,
    };

    self.regs.set(dest, result);
}

inline fn bitAnd(a: anytype, b: anytype) @TypeOf(a, b) {
    return a & b;
}

inline fn bitOr(a: anytype, b: anytype) @TypeOf(a, b) {
    return a | b;
}

inline fn bitXor(a: anytype, b: anytype) @TypeOf(a, b) {
    return a ^ b;
}

inline fn shl(a: anytype, b: anytype) @TypeOf(a, b) {
    return a << @intCast(b);
}

inline fn shr(a: anytype, b: anytype) @TypeOf(a, b) {
    return a >> @intCast(b);
}
