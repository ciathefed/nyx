const std = @import("std");
const Allocator = std.mem.Allocator;
const DataSize = @import("../../parser/immediate.zig").DataSize;
const Immediate = @import("../../parser/immediate.zig").Immediate;

const Bus = @This();

ptr: *anyopaque,
vtable: *const VTable,

pub const VTable = struct {
    name: *const fn (*anyopaque) []const u8,
    size: *const fn (*anyopaque) usize,
    read: *const fn (*anyopaque, addr: usize, sz: DataSize) anyerror!Immediate,
    readSlice: *const fn (*anyopaque, start: usize, end: usize) anyerror![]const u8,
    write: *const fn (*anyopaque, addr: usize, value: Immediate, sz: DataSize) anyerror!void,
    writeSlice: *const fn (*anyopaque, start: usize, data: []const u8) anyerror!void,
};

pub fn name(self: *Bus) []const u8 {
    return self.vtable.name(self.ptr);
}

pub fn size(self: *Bus) usize {
    return self.vtable.size(self.ptr);
}

pub fn read(self: *Bus, addr: usize, sz: DataSize) anyerror!Immediate {
    return self.vtable.read(self.ptr, addr, sz);
}

pub fn readSlice(self: *Bus, start: usize, end: usize) anyerror![]const u8 {
    return self.vtable.readSlice(self.ptr, start, end);
}

pub fn write(self: *Bus, addr: usize, value: Immediate, sz: DataSize) anyerror!void {
    return self.vtable.write(self.ptr, addr, value, sz);
}

pub fn writeSlice(self: *Bus, start: usize, data: []const u8) anyerror!void {
    return self.vtable.writeSlice(self.ptr, start, data);
}
