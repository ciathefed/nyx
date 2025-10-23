const std = @import("std");
const Allocator = std.mem.Allocator;
const Vm = @import("vm/Vm.zig");
const Register = @import("vm/register.zig").Register;

export fn vm_get_reg_int(vm: ?*Vm, index: u8) i64 {
    var self = vm orelse return 0;
    const reg = Register.fromU8(index) catch return 0;
    return @intCast(self.regs.get(reg).asU64());
}

export fn vm_get_reg_float(vm: ?*Vm, index: u8) f64 {
    var self = vm orelse return 0.0;
    const reg = Register.fromU8(index) catch return 0.0;
    return self.regs.get(reg).asF64();
}

export fn vm_set_reg_int(vm: ?*Vm, index: u8, value: i64) void {
    var self = vm orelse return;
    const reg = Register.fromU8(index) catch return;
    self.regs.set(reg, .{ .qword = @intCast(value) });
}

export fn vm_set_reg_float(vm: ?*Vm, index: u8, value: f64) void {
    var self = vm orelse return;
    const reg = Register.fromU8(index) catch return;
    self.regs.set(reg, .{ .double = value });
}

export fn vm_mem_read_byte(vm: ?*Vm, addr: usize) u8 {
    var self = vm orelse return 0;
    return (self.mmu.read(addr, .byte) catch unreachable).asU8();
}

export fn vm_mem_read_word(vm: ?*Vm, addr: usize) u16 {
    var self = vm orelse return 0;
    return (self.mmu.read(addr, .word) catch unreachable).asU16();
}

export fn vm_mem_read_dword(vm: ?*Vm, addr: usize) u32 {
    var self = vm orelse return 0;
    return (self.mmu.read(addr, .dword) catch unreachable).asU32();
}

export fn vm_mem_read_qword(vm: ?*Vm, addr: usize) u64 {
    var self = vm orelse return 0;
    return (self.mmu.read(addr, .qword) catch unreachable).asU64();
}

export fn vm_mem_read_float(vm: ?*Vm, addr: usize) f32 {
    var self = vm orelse return 0;
    return (self.mmu.read(addr, .float) catch unreachable).asF32();
}

export fn vm_mem_read_double(vm: ?*Vm, addr: usize) f64 {
    var self = vm orelse return 0;
    return (self.mmu.read(addr, .double) catch unreachable).asF64();
}

export fn vm_mem_read_cstr(vm: ?*Vm, addr: usize) [*c]const u8 {
    var self = vm orelse return "";
    const str = blk: {
        var i = addr;
        while (true) {
            const byte = (self.mmu.read(i, .byte) catch unreachable).asU8();
            if (byte != 0) i += 1 else break;
        }
        break :blk self.mmu.readSlice(addr, i - addr) catch unreachable;
    };
    return @ptrCast(str);
}
