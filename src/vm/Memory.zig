const std = @import("std");
const mem = std.mem;
const Allocator = mem.Allocator;
const ArrayList = std.array_list.Managed;
const DataSize = @import("../parser/immediate.zig").DataSize;
const Immediate = @import("../parser/immediate.zig").Immediate;

const Memory = @This();

storage: ArrayList(u8),

pub fn init(size: usize, allocator: Allocator) !Memory {
    var storage = try ArrayList(u8).initCapacity(allocator, size);
    try storage.appendNTimes(0x00, size);
    return Memory{ .storage = storage };
}

pub fn deinit(self: *Memory) void {
    self.storage.deinit();
}

pub fn len(self: *Memory) usize {
    return self.storage.items.len;
}

pub fn read(self: *Memory, addr: usize, size: DataSize) !Immediate {
    switch (size) {
        .byte => {
            if (addr + 1 > self.len()) return error.AddressOutOfBounds;
            return .{ .byte = self.storage.items[addr] };
        },
        .word => {
            if (addr + 2 > self.len()) return error.AddressOutOfBounds;
            return .{
                .word = mem.readInt(u16, self.storage.items[addr .. addr + 2][0..2], .little),
            };
        },
        .dword => {
            if (addr + 4 > self.len()) return error.AddressOutOfBounds;
            return .{
                .dword = mem.readInt(u32, self.storage.items[addr .. addr + 4][0..4], .little),
            };
        },
        .qword => {
            if (addr + 8 > self.len()) return error.AddressOutOfBounds;
            return .{
                .qword = mem.readInt(u64, self.storage.items[addr .. addr + 8][0..8], .little),
            };
        },
        .float => {
            if (addr + 4 > self.len()) return error.AddressOutOfBounds;
            const bytes = self.storage.items[addr .. addr + 4][0..4];
            return .{
                .float = @bitCast(mem.readInt(u32, bytes, .little)),
            };
        },
        .double => {
            if (addr + 8 > self.len()) return error.AddressOutOfBounds;
            const bytes = self.storage.items[addr .. addr + 8][0..8];
            return .{
                .double = @bitCast(mem.readInt(u64, bytes, .little)),
            };
        },
    }
}

pub fn write(self: *Memory, addr: usize, value: Immediate, size: DataSize) !void {
    switch (size) {
        .byte => {
            if (addr + 1 > self.len()) return error.AddressOutOfBounds;
            self.storage.items[addr] = value.asU8();
        },
        .word => {
            if (addr + 2 > self.len()) return error.AddressOutOfBounds;
            @memcpy(self.storage.items[addr .. addr + 2], &mem.toBytes(value.asU16()));
        },
        .dword => {
            if (addr + 4 > self.len()) return error.AddressOutOfBounds;
            @memcpy(self.storage.items[addr .. addr + 4], &mem.toBytes(value.asU32()));
        },
        .qword => {
            if (addr + 8 > self.len()) return error.AddressOutOfBounds;
            @memcpy(self.storage.items[addr .. addr + 8], &mem.toBytes(value.asU64()));
        },
        .float => {
            if (addr + 4 > self.len()) return error.AddressOutOfBounds;
            @memcpy(self.storage.items[addr .. addr + 4], &mem.toBytes(value.asF32()));
        },
        .double => {
            if (addr + 8 > self.len()) return error.AddressOutOfBounds;
            @memcpy(self.storage.items[addr .. addr + 8], &mem.toBytes(value.asF64()));
        },
    }
}
