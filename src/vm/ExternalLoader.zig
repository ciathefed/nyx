const std = @import("std");
const Allocator = std.mem.Allocator;
const DynLib = std.DynLib;
const Vm = @import("Vm.zig");
const Mmu = @import("memory/Mmu.zig");
const Register = @import("register.zig").Register;
const DataSize = @import("../parser/immediate.zig").DataSize;
const Immediate = @import("../parser/immediate.zig").Immediate;
const c = @cImport({
    @cInclude("ffi.h");
});

const ExternalLoader = @This();

external_libraries: std.StringHashMap(*DynLib),
gpa: Allocator,

pub fn init(gpa: Allocator) ExternalLoader {
    return ExternalLoader{
        .external_libraries = .init(gpa),
        .gpa = gpa,
    };
}

pub fn deinit(self: *ExternalLoader) void {
    var it = self.external_libraries.iterator();
    while (it.next()) |entry| {
        self.gpa.destroy(entry.value_ptr.*);
    }
    self.external_libraries.deinit();
}

pub fn load(self: *ExternalLoader, path: []const u8) !void {
    const lib = try DynLib.open(path);
    const obj = try self.gpa.create(DynLib);
    errdefer self.gpa.destroy(obj);
    obj.* = lib;
    try self.external_libraries.put(path, obj);
}

pub fn lookup(self: *ExternalLoader, name: []const u8) !*anyopaque {
    const temp = try self.gpa.dupeZ(u8, name);
    defer self.gpa.free(temp);

    var it = self.external_libraries.iterator();
    while (it.next()) |entry| {
        if (entry.value_ptr.*.lookup(*anyopaque, temp)) |func| return func;
    }
    return error.ExternalFunctionNotFound;
}

pub const FfiType = enum(u8) {
    byte = 0x00,
    word = 0x01,
    dword = 0x02,
    qword = 0x03,
    float = 0x04,
    double = 0x05,
    void = 0x06,
    ptr = 0x07,
    // Values 0x80..0xFF encode struct-by-value types.
    // The struct size in bytes is (value - 0x80 + 1), giving sizes 1..128.
    _,

    pub fn isStruct(self: FfiType) bool {
        return @intFromEnum(self) >= 0x80;
    }

    pub fn structSize(self: FfiType) usize {
        return @as(usize, @intFromEnum(self)) - 0x80 + 1;
    }

    pub fn toLibffiType(self: FfiType, struct_type_buf: *StructTypeBuf) ?*c.ffi_type {
        return switch (@intFromEnum(self)) {
            0x00 => &c.ffi_type_uint8,
            0x01 => &c.ffi_type_uint16,
            0x02 => &c.ffi_type_uint32,
            0x03 => &c.ffi_type_uint64,
            0x04 => &c.ffi_type_float,
            0x05 => &c.ffi_type_double,
            0x06 => &c.ffi_type_void,
            0x07 => &c.ffi_type_pointer,
            0x80...0xFF => struct_type_buf.getOrCreate(self.structSize()),
            else => null,
        };
    }

    pub fn fromU8(val: u8) !FfiType {
        return switch (val) {
            0x00...0x07 => @enumFromInt(val),
            0x80...0xFF => @enumFromInt(val),
            else => error.InvalidFfiType,
        };
    }

    pub fn isFloat(self: FfiType) bool {
        return @intFromEnum(self) == 0x04 or @intFromEnum(self) == 0x05;
    }

    pub fn isIntOrPtr(self: FfiType) bool {
        const v = @intFromEnum(self);
        return v <= 0x03 or v == 0x07;
    }
};

const StructTypeBuf = struct {
    element_arrays: [8][129]*c.ffi_type = undefined,
    null_sentinel: @TypeOf(@as(*c.ffi_type, undefined)) = undefined,
    types: [8]c.ffi_type = undefined,
    sizes: [8]usize = undefined,
    count: usize = 0,

    fn getOrCreate(self: *StructTypeBuf, size: usize) ?*c.ffi_type {
        for (0..self.count) |i| {
            if (self.sizes[i] == size) return &self.types[i];
        }
        if (self.count >= 8) return null;

        const idx = self.count;
        for (0..size) |j| {
            self.element_arrays[idx][j] = @constCast(&c.ffi_type_uint8);
        }

        const elem_ptr: [*]*c.ffi_type = &self.element_arrays[idx];
        const as_opt: [*]?*c.ffi_type = @ptrCast(elem_ptr);
        as_opt[size] = null;

        self.types[idx] = .{
            .size = 0,
            .alignment = 0,
            .type = c.FFI_TYPE_STRUCT,
            .elements = @ptrCast(elem_ptr),
        };
        self.sizes[idx] = size;
        self.count += 1;
        return &self.types[idx];
    }
};

const IntArgReg = struct { q: Register, d: Register, w: Register, b: Register };

const int_arg_regs = [6]IntArgReg{
    .{ .q = .q0, .d = .d0, .w = .w0, .b = .b0 },
    .{ .q = .q1, .d = .d1, .w = .w1, .b = .b1 },
    .{ .q = .q2, .d = .d2, .w = .w2, .b = .b2 },
    .{ .q = .q3, .d = .d3, .w = .w3, .b = .b3 },
    .{ .q = .q4, .d = .d4, .w = .w4, .b = .b4 },
    .{ .q = .q5, .d = .d5, .w = .w5, .b = .b5 },
};

const float_arg_regs_ff = [6]Register{ .ff0, .ff1, .ff2, .ff3, .ff4, .ff5 };
const float_arg_regs_dd = [6]Register{ .dd0, .dd1, .dd2, .dd3, .dd4, .dd5 };

/// Maximum number of arguments we support (register args + reasonable stack
/// overflow).
const MAX_ARGS = 64;

fn popVm(vm: *Vm, size: DataSize) !Immediate {
    const current_sp = vm.regs.sp();
    if (current_sp + size.sizeInBytes() > vm.mmu.size()) {
        return error.StackUnderflow;
    }
    const value = try vm.mmu.read(current_sp, size);
    vm.regs.setSp(current_sp + size.sizeInBytes());
    return value;
}

pub fn call(func_ptr: *anyopaque, ret_type: FfiType, arg_types: []const FfiType, fixed_arg_count: u8, vm: *Vm) !void {
    if (arg_types.len > MAX_ARGS) return error.TooManyArguments;

    var struct_type_buf: StructTypeBuf = .{};

    var ffi_arg_types: [MAX_ARGS]*c.ffi_type = undefined;
    for (arg_types, 0..) |at, i| {
        ffi_arg_types[i] = at.toLibffiType(&struct_type_buf) orelse return error.InvalidFfiType;
    }

    const ret_ffi_type = ret_type.toLibffiType(&struct_type_buf) orelse return error.InvalidFfiType;

    var cif: c.ffi_cif = undefined;
    const prep_status = if (fixed_arg_count < arg_types.len)
        c.ffi_prep_cif_var(
            &cif,
            c.FFI_DEFAULT_ABI,
            @intCast(fixed_arg_count),
            @intCast(arg_types.len),
            ret_ffi_type,
            if (arg_types.len > 0) @ptrCast(&ffi_arg_types) else null,
        )
    else
        c.ffi_prep_cif(
            &cif,
            c.FFI_DEFAULT_ABI,
            @intCast(arg_types.len),
            ret_ffi_type,
            if (arg_types.len > 0) @ptrCast(&ffi_arg_types) else null,
        );
    if (prep_status != c.FFI_OK) return error.FfiPrepFailed;

    var arg_values_u8: [MAX_ARGS]u8 = undefined;
    var arg_values_u16: [MAX_ARGS]u16 = undefined;
    var arg_values_u32: [MAX_ARGS]u32 = undefined;
    var arg_values_u64: [MAX_ARGS]u64 = undefined;
    var arg_values_f32: [MAX_ARGS]f32 = undefined;
    var arg_values_f64: [MAX_ARGS]f64 = undefined;
    var arg_values_ptr: [MAX_ARGS]?*anyopaque = undefined;
    var arg_struct_bufs: [MAX_ARGS][128]u8 = undefined;

    var arg_ptrs: [MAX_ARGS]?*anyopaque = undefined;

    var int_count: usize = 0;
    var float_count: usize = 0;
    var total_overflow: usize = 0;
    for (arg_types) |at| {
        if (at.isFloat()) {
            if (float_count < 6) {
                float_count += 1;
            } else {
                total_overflow += 1;
            }
        } else if (at.isIntOrPtr() or at.isStruct()) {
            if (int_count < 6) {
                int_count += 1;
            } else {
                total_overflow += 1;
            }
        }
    }

    var stack_values: [MAX_ARGS]u64 = undefined;
    for (0..total_overflow) |i| {
        stack_values[i] = (try popVm(vm, .qword)).asU64();
    }

    var int_reg_idx: usize = 0;
    var float_reg_idx: usize = 0;
    var stack_read_idx: usize = 0;

    for (arg_types, 0..) |at, i| {
        switch (at) {
            .byte => {
                if (int_reg_idx < 6) {
                    arg_values_u8[i] = vm.regs.get(int_arg_regs[int_reg_idx].b).asU8();
                    int_reg_idx += 1;
                } else {
                    arg_values_u8[i] = @truncate(stack_values[stack_read_idx]);
                    stack_read_idx += 1;
                }
                arg_ptrs[i] = @ptrCast(&arg_values_u8[i]);
            },
            .word => {
                if (int_reg_idx < 6) {
                    arg_values_u16[i] = vm.regs.get(int_arg_regs[int_reg_idx].w).asU16();
                    int_reg_idx += 1;
                } else {
                    arg_values_u16[i] = @truncate(stack_values[stack_read_idx]);
                    stack_read_idx += 1;
                }
                arg_ptrs[i] = @ptrCast(&arg_values_u16[i]);
            },
            .dword => {
                if (int_reg_idx < 6) {
                    arg_values_u32[i] = vm.regs.get(int_arg_regs[int_reg_idx].d).asU32();
                    int_reg_idx += 1;
                } else {
                    arg_values_u32[i] = @truncate(stack_values[stack_read_idx]);
                    stack_read_idx += 1;
                }
                arg_ptrs[i] = @ptrCast(&arg_values_u32[i]);
            },
            .qword => {
                if (int_reg_idx < 6) {
                    arg_values_u64[i] = vm.regs.get(int_arg_regs[int_reg_idx].q).asU64();
                    int_reg_idx += 1;
                } else {
                    arg_values_u64[i] = stack_values[stack_read_idx];
                    stack_read_idx += 1;
                }
                arg_ptrs[i] = @ptrCast(&arg_values_u64[i]);
            },
            .ptr => {
                const vm_addr: u64 = if (int_reg_idx < 6) blk: {
                    const v = vm.regs.get(int_arg_regs[int_reg_idx].q).asU64();
                    int_reg_idx += 1;
                    break :blk v;
                } else blk: {
                    const v = stack_values[stack_read_idx];
                    stack_read_idx += 1;
                    break :blk v;
                };
                if (vm_addr == 0) {
                    arg_values_ptr[i] = null;
                } else if (vm_addr < vm.mmu.size()) {
                    // Address falls within VM memory, resolve to host pointer
                    const host = vm.mmu.resolveHostPtr(@intCast(vm_addr));
                    arg_values_ptr[i] = if (host) |h| @ptrCast(h) else null;
                } else {
                    // Address is outside VM memory, treat as a raw host pointer
                    // (e.g. returned by a previous FFI call like TextFormat)
                    arg_values_ptr[i] = @ptrFromInt(@as(usize, @intCast(vm_addr)));
                }
                arg_ptrs[i] = @ptrCast(&arg_values_ptr[i]);
            },
            .float => {
                if (float_reg_idx < 6) {
                    arg_values_f32[i] = vm.regs.get(float_arg_regs_ff[float_reg_idx]).asF32();
                    float_reg_idx += 1;
                } else {
                    arg_values_f32[i] = @bitCast(@as(u32, @truncate(stack_values[stack_read_idx])));
                    stack_read_idx += 1;
                }
                arg_ptrs[i] = @ptrCast(&arg_values_f32[i]);
            },
            .double => {
                if (float_reg_idx < 6) {
                    arg_values_f64[i] = vm.regs.get(float_arg_regs_dd[float_reg_idx]).asF64();
                    float_reg_idx += 1;
                } else {
                    arg_values_f64[i] = @bitCast(stack_values[stack_read_idx]);
                    stack_read_idx += 1;
                }
                arg_ptrs[i] = @ptrCast(&arg_values_f64[i]);
            },
            else => {
                if (at.isStruct()) {
                    const struct_sz = at.structSize();
                    const vm_addr: u64 = if (int_reg_idx < 6) blk: {
                        const v = vm.regs.get(int_arg_regs[int_reg_idx].q).asU64();
                        int_reg_idx += 1;
                        break :blk v;
                    } else blk: {
                        const v = stack_values[stack_read_idx];
                        stack_read_idx += 1;
                        break :blk v;
                    };
                    const host_ptr = vm.mmu.resolveHostPtr(@intCast(vm_addr)) orelse return error.AddressOutOfBounds;
                    @memcpy(arg_struct_bufs[i][0..struct_sz], host_ptr[0..struct_sz]);
                    arg_ptrs[i] = @ptrCast(&arg_struct_bufs[i]);
                } else {
                    return error.VoidArgumentType;
                }
            },
        }
    }

    var ret_u64: u64 = 0;
    var ret_f32: f32 = 0;
    var ret_f64: f64 = 0;
    var ret_struct_buf: [128]u8 = undefined;
    const ret_storage: ?*anyopaque = blk: {
        const v = @intFromEnum(ret_type);
        if (v == @intFromEnum(FfiType.void)) break :blk null;
        if (v == @intFromEnum(FfiType.float)) break :blk @ptrCast(&ret_f32);
        if (v == @intFromEnum(FfiType.double)) break :blk @ptrCast(&ret_f64);
        if (ret_type.isStruct()) break :blk @ptrCast(&ret_struct_buf);
        break :blk @ptrCast(&ret_u64);
    };

    c.ffi_call(
        &cif,
        @ptrCast(@alignCast(func_ptr)),
        ret_storage,
        if (arg_types.len > 0) @ptrCast(&arg_ptrs) else null,
    );

    const ret_v = @intFromEnum(ret_type);
    if (ret_v == @intFromEnum(FfiType.void)) {
        // nothing
    } else if (ret_v == @intFromEnum(FfiType.byte)) {
        vm.regs.set(.b0, .{ .byte = @truncate(ret_u64) });
    } else if (ret_v == @intFromEnum(FfiType.word)) {
        vm.regs.set(.w0, .{ .word = @truncate(ret_u64) });
    } else if (ret_v == @intFromEnum(FfiType.dword)) {
        vm.regs.set(.d0, .{ .dword = @truncate(ret_u64) });
    } else if (ret_v == @intFromEnum(FfiType.qword) or ret_v == @intFromEnum(FfiType.ptr)) {
        vm.regs.set(.q0, .{ .qword = ret_u64 });
    } else if (ret_v == @intFromEnum(FfiType.float)) {
        vm.regs.set(.ff0, .{ .float = ret_f32 });
    } else if (ret_v == @intFromEnum(FfiType.double)) {
        vm.regs.set(.dd0, .{ .double = ret_f64 });
    } else if (ret_type.isStruct()) {
        const dest_addr = vm.regs.get(.q0).asU64();
        const struct_sz = ret_type.structSize();
        const host_ptr = vm.mmu.resolveHostPtr(@intCast(dest_addr)) orelse return error.AddressOutOfBounds;
        @memcpy(host_ptr[0..struct_sz], ret_struct_buf[0..struct_sz]);
    }
}
