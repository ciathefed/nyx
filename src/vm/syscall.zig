// TODO: move away from posix syscalls and use zig stdlib
// TODO: err register so program doesnt crash on a zig error
const std = @import("std");
const posix = std.posix;
const Allocator = std.mem.Allocator;
const Vm = @import("Vm.zig");

pub const SyscallFn = *const fn (self: *Vm) anyerror!void;
pub const Syscalls = std.AutoHashMap(usize, SyscallFn);

pub fn collectSyscalls(allocator: Allocator) !Syscalls {
    var syscalls = Syscalls.init(allocator);

    try syscalls.put(0x00, sysOpen);
    try syscalls.put(0x01, sysClose);
    try syscalls.put(0x02, sysRead);
    try syscalls.put(0x03, sysWrite);
    try syscalls.put(0x04, sysMalloc);
    try syscalls.put(0x05, sysFree);
    try syscalls.put(0xFF, sysExit);

    return syscalls;
}

fn sysOpen(self: *Vm) anyerror!void {
    const path_addr = self.regs.get(.q0).asUsize();
    const flags = self.regs.get(.d1).asU32();
    const mode = self.regs.get(.w2).asU16();

    if (path_addr >= self.mmu.size()) return error.AddressOutOfBounds;

    const path = blk: {
        var i = path_addr;
        while ((try self.mmu.read(i, .byte)).asU8() != 0) i += 1;
        break :blk try self.mmu.readSlice(path_addr, i - path_addr);
    };

    const fd = try posix.open(path, @bitCast(flags), mode);

    self.regs.set(.q0, .{ .qword = @intCast(fd) });
}

fn sysClose(self: *Vm) anyerror!void {
    const fd: i32 = @intCast(self.regs.get(.d0).asU32());
    posix.close(fd);
}

fn sysRead(self: *Vm) anyerror!void {
    const fd: i32 = @intCast(self.regs.get(.d0).asU32());
    const addr = self.regs.get(.q1).asUsize();
    const count = self.regs.get(.q2).asUsize();

    if (addr + count >= self.mmu.size()) return error.AddressOutOfBounds;

    var buf = try self.mmu.allocator.alloc(u8, count);
    defer self.mmu.allocator.free(buf);

    const n = try posix.read(fd, buf);

    try self.mmu.writeSlice(addr, buf[0..n]);

    self.regs.set(.q0, .{ .qword = @intCast(n) });
}

fn sysWrite(self: *Vm) anyerror!void {
    const fd: i32 = @intCast(self.regs.get(.d0).asU32());
    const addr = self.regs.get(.q1).asUsize();
    const count = self.regs.get(.q2).asUsize();

    if (addr + count >= self.mmu.size()) return error.AddressOutOfBounds;

    const buf = try self.mmu.readSlice(addr, count);
    const n = try posix.write(fd, buf);

    self.regs.set(.q0, .{ .qword = @intCast(n) });
}

fn sysMalloc(self: *Vm) anyerror!void {
    const size: usize = self.regs.get(.q0).asUsize();
    const addr = try self.mmu.addBlock("Block", size);
    self.regs.set(.q0, .{ .qword = @intCast(addr) });
}

pub fn sysFree(self: *Vm) !void {
    const addr: usize = self.regs.get(.q0).asUsize();

    if (self.mmu.blocks.items.len <= 2) return error.NoDynamicBlocks;

    var start: usize = blk: {
        var s: usize = 0;
        for (self.mmu.blocks.items[0..2]) |b| {
            var bus = b.bus();
            s += bus.size();
        }
        break :blk s;
    };
    var i: usize = 2;
    while (i < self.mmu.blocks.items.len) : (i += 1) {
        const block = self.mmu.blocks.items[i];
        var bus = block.bus();
        const end = start + bus.size();

        if (addr == start) {
            _ = self.mmu.buses.orderedRemove(i);
            _ = self.mmu.blocks.orderedRemove(i);
            block.deinit();
            self.mmu.allocator.destroy(block);
            return;
        }

        start = end;
    }

    return error.InvalidFreeAddress;
}

fn sysExit(self: *Vm) anyerror!void {
    const status = self.regs.get(.b0).asU8();
    posix.exit(status);
}
