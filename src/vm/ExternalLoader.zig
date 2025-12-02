const std = @import("std");
const Allocator = std.mem.Allocator;
const DynLib = std.DynLib;
const Vm = @import("Vm.zig");

const ExternalLoader = @This();
const ExternalFn = *const fn (?*Vm) callconv(.c) i32;

external_libraries: std.StringHashMap(*DynLib),
gpa: Allocator,

pub fn init(gpa: Allocator) ExternalLoader {
    return ExternalLoader{
        .external_libraries = .init(gpa),
        .gpa = gpa,
    };
}

pub fn deinit(self: *ExternalLoader) void {
    var it = self.external_libraries.iterator();
    while (it.next()) |entry| {
        self.gpa.destroy(entry.value_ptr.*);
    }
    self.external_libraries.deinit();
}

pub fn load(self: *ExternalLoader, path: []const u8) !void {
    const lib = try DynLib.open(path);
    const obj = try self.gpa.create(DynLib);
    errdefer self.gpa.destroy(obj);
    obj.* = lib;
    try self.external_libraries.put(path, obj);
}

pub fn lookup(self: *ExternalLoader, name: []const u8) !ExternalFn {
    const temp = try self.gpa.dupeZ(u8, name);
    defer self.gpa.free(temp);

    var it = self.external_libraries.iterator();
    while (it.next()) |entry| {
        const res = entry.value_ptr.*.lookup(ExternalFn, temp);
        if (res) |func| return func;
    }
    return error.ExternalFunctionNotFound;
}
