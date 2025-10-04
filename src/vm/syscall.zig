// TODO: move away from posix syscalls and use zig stdlib
// TODO: err register so program doesnt crash on a zig error
const std = @import("std");
const posix = std.posix;
const Allocator = std.mem.Allocator;
const Machine = @import("Machine.zig");

pub const SyscallFn = *const fn (self: *Machine) anyerror!void;
pub const Syscalls = std.AutoHashMap(usize, SyscallFn);

pub fn collectSyscalls(allocator: Allocator) !Syscalls {
    var syscalls = Syscalls.init(allocator);

    try syscalls.put(0x00, sysOpen);
    try syscalls.put(0x01, sysClose);
    try syscalls.put(0x02, sysRead);
    try syscalls.put(0x03, sysWrite);
    try syscalls.put(0xFF, sysExit);

    return syscalls;
}

fn sysOpen(self: *Machine) anyerror!void {
    const path_addr = self.regs.get(.q0).asUsize();
    const flags = self.regs.get(.d1).asU32();
    const mode = self.regs.get(.w2).asU16();

    if (path_addr >= self.memory.len()) return error.InstructionPointerOutOfBounds;

    const path = blk: {
        var i = path_addr;
        while (self.memory.storage.items[i] != 0) i += 1;
        break :blk self.memory.storage.items[path_addr..i];
    };

    const fd = try posix.open(path, @bitCast(flags), mode);

    self.regs.set(.q0, .{ .qword = @intCast(fd) });
}

fn sysClose(self: *Machine) anyerror!void {
    const fd: i32 = @intCast(self.regs.get(.d0).asU32());
    posix.close(fd);
}

fn sysRead(self: *Machine) anyerror!void {
    const fd: i32 = @intCast(self.regs.get(.d0).asU32());
    const addr = self.regs.get(.q1).asUsize();
    const count = self.regs.get(.q2).asUsize();

    if (addr + count >= self.memory.len()) return error.InstructionPointerOutOfBounds;

    const n = try posix.read(fd, self.memory.storage.items[addr .. addr + count]);

    self.regs.set(.q0, .{ .qword = @intCast(n) });
}

fn sysWrite(self: *Machine) anyerror!void {
    const fd: i32 = @intCast(self.regs.get(.d0).asU32());
    const addr = self.regs.get(.q1).asUsize();
    const count = self.regs.get(.q2).asUsize();

    if (addr + count >= self.memory.len()) return error.InstructionPointerOutOfBounds;

    const buf = self.memory.storage.items[addr .. addr + count];
    const n = try posix.write(fd, buf);

    self.regs.set(.q0, .{ .qword = @intCast(n) });
}

fn sysExit(self: *Machine) anyerror!void {
    const status = self.regs.get(.b0).asU8();
    posix.exit(status);
}
