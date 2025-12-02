const std = @import("std");
const fs = std.fs;
const Allocator = std.mem.Allocator;

pub fn readFromFile(file_path: []const u8, gpa: Allocator) ![]u8 {
    const stat = try fs.cwd().statFile(file_path);
    const buffer = try gpa.alloc(u8, @intCast(stat.size));
    return fs.cwd().readFile(file_path, buffer);
}

pub fn writeToFile(file_path: []const u8, data: []const u8) !void {
    return fs.cwd().writeFile(.{ .sub_path = file_path, .data = data });
}

pub fn fileExists(file_path: []const u8) bool {
    fs.cwd().access(file_path, .{}) catch return false;
    return true;
}
