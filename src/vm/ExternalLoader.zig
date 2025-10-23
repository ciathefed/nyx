const std = @import("std");
const Allocator = std.mem.Allocator;
const DynLib = std.DynLib;
const Vm = @import("Vm.zig");

const ExternalLoader = @This();
const ExternalFn = *const fn (?*Vm) callconv(.c) i32;

external_libraries: std.StringHashMap(*DynLib),
allocator: Allocator,

pub fn init(allocator: Allocator) ExternalLoader {
    return ExternalLoader{
        .external_libraries = .init(allocator),
        .allocator = allocator,
    };
}

pub fn deinit(self: *ExternalLoader) void {
    var it = self.external_libraries.iterator();
    while (it.next()) |entry| {
        self.allocator.destroy(entry.value_ptr.*);
    }
    self.external_libraries.deinit();
}

pub fn load(self: *ExternalLoader, path: []const u8) !void {
    const lib = try DynLib.open(path);
    const obj = try self.allocator.create(DynLib);
    errdefer self.allocator.destroy(obj);
    obj.* = lib;
    try self.external_libraries.put(path, obj);
}

pub fn lookup(self: *ExternalLoader, name: []const u8) !ExternalFn {
    const temp = try self.allocator.dupeZ(u8, name);
    defer self.allocator.free(temp);

    var it = self.external_libraries.iterator();
    while (it.next()) |entry| {
        const res = entry.value_ptr.*.lookup(ExternalFn, temp);
        if (res) |func| return func;
    }
    return error.ExternalFunctionNotFound;
}
