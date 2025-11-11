const std = @import("std");
const Allocator = std.mem.Allocator;
const ArrayList = std.array_list.Managed;

const StringInterner = @This();

pub const StringId = u32;

pub const INVALID_ID: StringId = std.math.maxInt(StringId);

allocator: Allocator,
strings: ArrayList([]const u8),
map: std.StringHashMap(StringId),

pub fn init(allocator: Allocator) StringInterner {
    return .{
        .allocator = allocator,
        .strings = .init(allocator),
        .map = .init(allocator),
    };
}

pub fn deinit(self: *StringInterner) void {
    for (self.strings.items) |s| {
        self.allocator.free(s);
    }
    self.strings.deinit();
    self.map.deinit();
}

pub fn intern(self: *StringInterner, s: []const u8) !StringId {
    if (self.map.get(s)) |id| {
        return id;
    }

    const id: StringId = @intCast(self.strings.items.len);
    const owned = try self.allocator.dupe(u8, s);
    errdefer self.allocator.free(owned);

    try self.strings.append(owned);
    try self.map.put(owned, id);

    return id;
}

pub fn get(self: *const StringInterner, id: StringId) ?[]const u8 {
    if (id >= self.strings.items.len) return null;
    return self.strings.items[id];
}

pub fn getId(self: *const StringInterner, s: []const u8) ?StringId {
    return self.map.get(s);
}

pub fn count(self: *const StringInterner) usize {
    return self.strings.items.len;
}
