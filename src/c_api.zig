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
