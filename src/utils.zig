const std = @import("std");
const Io = std.Io;
const Allocator = std.mem.Allocator;

pub fn readFromFile(io: std.Io, gpa: Allocator, file_path: []const u8) ![]u8 {
    var cwd = Io.Dir.cwd();
    const stat = try cwd.statFile(io, file_path, .{});
    const buffer = try gpa.alloc(u8, @intCast(stat.size));
    return try cwd.readFile(io, file_path, buffer);
}

pub fn writeToFile(io: std.Io, file_path: []const u8, data: []const u8) !void {
    var cwd = Io.Dir.cwd();
    try cwd.writeFile(io, .{ .sub_path = file_path, .data = data });
}

pub fn fileExists(io: std.Io, file_path: []const u8) bool {
    var cwd = Io.Dir.cwd();
    cwd.access(io, file_path, .{}) catch return false;
    return true;
}
