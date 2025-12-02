const std = @import("std");
const mem = std.mem;
const Allocator = mem.Allocator;
const ArrayList = std.array_list.Managed;

const Bytecode = @This();

pub const Section = enum { text, data };

text: ArrayList(u8),
data: ArrayList(u8),
current_section: Section,

pub fn init(capacity: ?usize, gpa: Allocator) !Bytecode {
    const cap = capacity orelse 1024;
    return Bytecode{
        .text = try .initCapacity(gpa, @divTrunc(cap, 2)),
        .data = try .initCapacity(gpa, @divTrunc(cap, 2)),
        .current_section = .text,
    };
}

pub fn deinit(self: *Bytecode) void {
    self.text.deinit();
    self.data.deinit();
}

pub fn len(self: *Bytecode, section: Section) usize {
    return switch (section) {
        .text => self.text.items.len,
        .data => self.data.items.len,
    };
}

pub inline fn push(self: *Bytecode, value: anytype) !void {
    const byte: u8 = switch (@typeInfo(@TypeOf(value))) {
        .@"enum" => @intCast(@intFromEnum(value)),
        .int => @intCast(value),
        .comptime_int => @intCast(value),
        else => @compileError("Expected enum, int, or comptime_int type, got " ++ @typeName(@TypeOf(value))),
    };

    switch (self.current_section) {
        .text => try self.text.append(byte),
        .data => try self.data.append(byte),
    }
}

pub inline fn extend(self: *Bytecode, iter: anytype) !void {
    switch (self.current_section) {
        .text => try self.text.appendSlice(iter),
        .data => try self.data.appendSlice(iter),
    }
}

pub inline fn grow(self: *Bytecode, amount: usize) !void {
    const zeros = try self.getAllocator().alloc(u8, amount);
    defer self.getAllocator().free(zeros);
    @memset(zeros, 0);

    switch (self.current_section) {
        .text => try self.text.appendSlice(zeros),
        .data => try self.data.appendSlice(zeros),
    }
}

inline fn getAllocator(self: *Bytecode) Allocator {
    return switch (self.current_section) {
        .text => self.text.allocator,
        .data => self.data.allocator,
    };
}

pub inline fn writeU8At(self: *Bytecode, section: Section, offset: usize, value: u8) void {
    switch (section) {
        .text => self.text.items[offset] = value,
        .data => self.data.items[offset] = value,
    }
}

pub inline fn writeU16At(self: *Bytecode, section: Section, offset: usize, value: u16) void {
    const bytes = mem.toBytes(value);
    switch (section) {
        .text => @memcpy(self.text.items[offset .. offset + 2], &bytes),
        .data => @memcpy(self.data.items[offset .. offset + 2], &bytes),
    }
}

pub inline fn writeU32At(self: *Bytecode, section: Section, offset: usize, value: u32) void {
    const bytes = mem.toBytes(value);
    switch (section) {
        .text => @memcpy(self.text.items[offset .. offset + 4], &bytes),
        .data => @memcpy(self.data.items[offset .. offset + 4], &bytes),
    }
}

pub inline fn writeU64At(self: *Bytecode, section: Section, offset: usize, value: u64) void {
    const bytes = mem.toBytes(value);
    switch (section) {
        .text => @memcpy(self.text.items[offset .. offset + 8], &bytes),
        .data => @memcpy(self.data.items[offset .. offset + 8], &bytes),
    }
}

pub fn finalize(self: *Bytecode, gpa: Allocator) ![]u8 {
    var bytes = ArrayList(u8).init(gpa);
    try bytes.appendSlice(self.text.items);
    try bytes.appendSlice(self.data.items);
    return bytes.toOwnedSlice();
}
