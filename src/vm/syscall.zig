// TODO: move away from posix syscalls and use zig stdlib
// TODO: err register so program doesn't crash on a zig error
const std = @import("std");
const posix = std.posix.system;
const Allocator = std.mem.Allocator;
const Vm = @import("Vm.zig");

pub const SyscallFn = *const fn (self: *Vm) anyerror!void;
pub const Syscalls = std.AutoHashMap(usize, SyscallFn);

pub fn collectSyscalls(gpa: Allocator) !Syscalls {
    var syscalls = Syscalls.init(gpa);

    try syscalls.put(0x00, sysOpen);
    try syscalls.put(0x01, sysClose);
    try syscalls.put(0x02, sysRead);
    try syscalls.put(0x03, sysWrite);
    try syscalls.put(0x04, sysMalloc);
    try syscalls.put(0x05, sysFree);
    try syscalls.put(0x06, sysSocket);
    try syscalls.put(0x07, sysConnect);
    try syscalls.put(0x08, sysBind);
    try syscalls.put(0x09, sysListen);
    try syscalls.put(0x0A, sysAccept);
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

    const fd = posix.open(@ptrCast(path), @bitCast(flags), mode);

    self.regs.set(.q0, .{ .qword = @intCast(fd) });
}

fn sysClose(self: *Vm) anyerror!void {
    const fd: i32 = @intCast(self.regs.get(.d0).asU32());
    _ = posix.close(fd);
}

fn sysRead(self: *Vm) anyerror!void {
    const fd: i32 = @intCast(self.regs.get(.d0).asU32());
    const addr = self.regs.get(.q1).asUsize();
    const count = self.regs.get(.q2).asUsize();

    if (addr + count >= self.mmu.size()) return error.AddressOutOfBounds;

    var buf = try self.mmu.gpa.alloc(u8, count);
    defer self.mmu.gpa.free(buf);

    const n = posix.read(fd, @ptrCast(buf), buf.len);

    try self.mmu.writeSlice(addr, buf[0..n]);

    self.regs.set(.q0, .{ .qword = @intCast(n) });
}

fn sysWrite(self: *Vm) anyerror!void {
    const fd: i32 = @intCast(self.regs.get(.d0).asU32());
    const addr = self.regs.get(.q1).asUsize();
    const count = self.regs.get(.q2).asUsize();

    if (addr + count >= self.mmu.size()) return error.AddressOutOfBounds;

    const buf = try self.mmu.readSlice(addr, count);
    const n = posix.write(fd, @ptrCast(buf), buf.len);

    self.regs.set(.q0, .{ .qword = @intCast(n) });
}

fn sysMalloc(self: *Vm) anyerror!void {
    const size: usize = self.regs.get(.q0).asUsize();
    const addr = try self.mmu.addBlock("Block", size);
    self.regs.set(.q0, .{ .qword = @intCast(addr) });
}

fn sysFree(self: *Vm) !void {
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
            self.mmu.gpa.destroy(block);
            return;
        }

        start = end;
    }

    return error.InvalidFreeAddress;
}

fn sysSocket(self: *Vm) anyerror!void {
    const domain = self.regs.get(.d0).asU32();
    const socket_type = self.regs.get(.d1).asU32();
    const protocol = self.regs.get(.d2).asU32();

    const sockfd = posix.socket(domain, socket_type, protocol);

    self.regs.set(.d0, .{ .dword = @bitCast(@as(i32, @intCast(sockfd))) });
}

fn sysConnect(self: *Vm) anyerror!void {
    const sockfd: i32 = @intCast(self.regs.get(.d0).asU32());
    const sockaddr_ptr = self.regs.get(.q1).asUsize();
    const sockaddr_family = (try self.mmu.read(sockaddr_ptr, .word)).asU16();
    const sockaddr_port = (try self.mmu.read(sockaddr_ptr + 2, .word)).asU16();
    const sockaddr_addr = (try self.mmu.read(sockaddr_ptr + 4, .dword)).asU32();
    const sockaddr_zero = (try self.mmu.readSlice(sockaddr_ptr + 8, 8))[0..8];

    const sockaddr_in = posix.sockaddr.in{
        .family = sockaddr_family,
        .port = sockaddr_port,
        .addr = sockaddr_addr,
        .zero = (@constCast(sockaddr_zero)).*,
    };

    const res = posix.connect(sockfd, @ptrCast(&sockaddr_in), @sizeOf(@TypeOf(sockaddr_in)));

    self.regs.set(.q0, .{ .qword = @intCast(res) });
}

fn sysBind(self: *Vm) anyerror!void {
    const sockfd: i32 = @intCast(self.regs.get(.d0).asU32());
    const sockaddr_ptr = self.regs.get(.q1).asUsize();
    const sockaddr_family = (try self.mmu.read(sockaddr_ptr, .word)).asU16();
    const sockaddr_port = (try self.mmu.read(sockaddr_ptr + 2, .word)).asU16();
    const sockaddr_addr = (try self.mmu.read(sockaddr_ptr + 4, .dword)).asU32();
    const sockaddr_zero = (try self.mmu.readSlice(sockaddr_ptr + 8, 8))[0..8];

    const sockaddr_in = posix.sockaddr.in{
        .family = sockaddr_family,
        .port = sockaddr_port,
        .addr = sockaddr_addr,
        .zero = (@constCast(sockaddr_zero)).*,
    };

    const res = posix.bind(sockfd, @ptrCast(&sockaddr_in), @sizeOf(@TypeOf(sockaddr_in)));

    self.regs.set(.q0, .{ .qword = @intCast(res) });
}

fn sysListen(self: *Vm) anyerror!void {
    const sockfd: i32 = @intCast(self.regs.get(.d0).asU32());
    const backlog: c_uint = @intCast(self.regs.get(.d1).asU32());

    const res = posix.listen(sockfd, backlog);

    self.regs.set(.d0, .{ .dword = @bitCast(@as(i32, @intCast(res))) });
}

fn sysAccept(self: *Vm) anyerror!void {
    const sockfd: i32 = @intCast(self.regs.get(.d0).asU32());
    const sockaddr_ptr = self.regs.get(.q1).asUsize();

    var sockaddr_in: posix.sockaddr.in = undefined;
    var sockaddr_in_len: u32 = @sizeOf(posix.sockaddr.in);
    const res = posix.accept(sockfd, @ptrCast(&sockaddr_in), &sockaddr_in_len);

    try self.mmu.write(sockaddr_ptr, .{ .word = sockaddr_in.family }, .word);
    try self.mmu.write(sockaddr_ptr + 2, .{ .word = sockaddr_in.port }, .word);
    try self.mmu.write(sockaddr_ptr + 4, .{ .dword = sockaddr_in.addr }, .dword);
    try self.mmu.writeSlice(sockaddr_ptr + 8, &sockaddr_in.zero);

    self.regs.set(.q0, .{ .qword = @intCast(res) });
}

fn sysExit(self: *Vm) anyerror!void {
    const status = self.regs.get(.b0).asU8();
    posix.exit(status);
}
