const std = @import("std");
const mem = std.mem;
const Allocator = mem.Allocator;
const Bus = @import("Bus.zig");
const DataSize = @import("../../parser/immediate.zig").DataSize;
const Immediate = @import("../../parser/immediate.zig").Immediate;

const Block = @This();

block_name: []const u8,
storage: []u8,
allocator: Allocator,

pub fn init(block_name: []const u8, len: usize, allocator: Allocator) !Block {
    return Block{
        .block_name = block_name,
        .storage = try allocator.alloc(u8, len),
        .allocator = allocator,
    };
}

pub fn deinit(self: *Block) void {
    self.allocator.free(self.storage);
}

fn name(ptr: *anyopaque) []const u8 {
    const self: *Block = @ptrCast(@alignCast(ptr));
    return self.block_name;
}

fn size(ptr: *anyopaque) usize {
    const self: *Block = @ptrCast(@alignCast(ptr));
    return self.storage.len;
}

fn read(ptr: *anyopaque, addr: usize, sz: DataSize) anyerror!Immediate {
    const self: *Block = @ptrCast(@alignCast(ptr));
    switch (sz) {
        .byte => {
            if (addr + 1 > size(ptr)) return error.AddressOutOfBounds;
            return .{ .byte = self.storage[addr] };
        },
        .word => {
            if (addr + 2 > size(ptr)) return error.AddressOutOfBounds;
            return .{
                .word = mem.readInt(u16, self.storage[addr .. addr + 2][0..2], .little),
            };
        },
        .dword => {
            if (addr + 4 > size(ptr)) return error.AddressOutOfBounds;
            return .{
                .dword = mem.readInt(u32, self.storage[addr .. addr + 4][0..4], .little),
            };
        },
        .qword => {
            if (addr + 8 > size(ptr)) return error.AddressOutOfBounds;
            return .{
                .qword = mem.readInt(u64, self.storage[addr .. addr + 8][0..8], .little),
            };
        },
        .float => {
            if (addr + 4 > size(ptr)) return error.AddressOutOfBounds;
            const bytes = self.storage[addr .. addr + 4][0..4];
            return .{
                .float = @bitCast(mem.readInt(u32, bytes, .little)),
            };
        },
        .double => {
            if (addr + 8 > size(ptr)) return error.AddressOutOfBounds;
            const bytes = self.storage[addr .. addr + 8][0..8];
            return .{
                .double = @bitCast(mem.readInt(u64, bytes, .little)),
            };
        },
    }
}

fn readSlice(ptr: *anyopaque, start: usize, end: usize) anyerror![]const u8 {
    const self: *Block = @ptrCast(@alignCast(ptr));
    if (start > end) return error.InvalidRange;
    if (end > self.storage.len) return error.AddressOutOfBounds;
    return self.storage[start..end];
}

fn write(ptr: *anyopaque, addr: usize, value: Immediate, sz: DataSize) anyerror!void {
    const self: *Block = @ptrCast(@alignCast(ptr));
    switch (sz) {
        .byte => {
            if (addr + 1 > size(ptr)) return error.AddressOutOfBounds;
            self.storage[addr] = value.asU8();
        },
        .word => {
            if (addr + 2 > size(ptr)) return error.AddressOutOfBounds;
            @memcpy(self.storage[addr .. addr + 2], &mem.toBytes(value.asU16()));
        },
        .dword => {
            if (addr + 4 > size(ptr)) return error.AddressOutOfBounds;
            @memcpy(self.storage[addr .. addr + 4], &mem.toBytes(value.asU32()));
        },
        .qword => {
            if (addr + 8 > size(ptr)) return error.AddressOutOfBounds;
            @memcpy(self.storage[addr .. addr + 8], &mem.toBytes(value.asU64()));
        },
        .float => {
            if (addr + 4 > size(ptr)) return error.AddressOutOfBounds;
            @memcpy(self.storage[addr .. addr + 4], &mem.toBytes(value.asF32()));
        },
        .double => {
            if (addr + 8 > size(ptr)) return error.AddressOutOfBounds;
            @memcpy(self.storage[addr .. addr + 8], &mem.toBytes(value.asF64()));
        },
    }
}

fn writeSlice(ptr: *anyopaque, start: usize, data: []const u8) anyerror!void {
    const self: *Block = @ptrCast(@alignCast(ptr));
    const end = start + data.len;
    if (end > self.storage.len) return error.AddressOutOfBounds;
    @memcpy(self.storage[start..end], data);
}

pub fn bus(self: *Block) Bus {
    return Bus{
        .ptr = self,
        .vtable = &.{
            .name = name,
            .size = size,
            .read = read,
            .readSlice = readSlice,
            .write = write,
            .writeSlice = writeSlice,
        },
    };
}
