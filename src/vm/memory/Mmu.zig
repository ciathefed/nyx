const std = @import("std");
const Allocator = std.mem.Allocator;
const ArrayList = std.array_list.Managed;
const Bus = @import("Bus.zig");
const Block = @import("Block.zig");
const DataSize = @import("../../parser/immediate.zig").DataSize;
const Immediate = @import("../../parser/immediate.zig").Immediate;

const Mmu = @This();

buses: ArrayList(Bus),
blocks: ArrayList(*Block),
allocated_slices: ArrayList([]u8),
allocator: Allocator,

pub fn init(allocator: Allocator) Mmu {
    return Mmu{
        .buses = .init(allocator),
        .blocks = .init(allocator),
        .allocated_slices = ArrayList([]u8).init(allocator),
        .allocator = allocator,
    };
}

pub fn deinit(self: *Mmu) void {
    for (self.blocks.items) |block| {
        block.deinit();
        self.allocator.destroy(block);
    }
    self.blocks.deinit();

    for (self.allocated_slices.items) |slice| {
        self.allocator.free(slice);
    }
    self.allocated_slices.deinit();
    self.buses.deinit();
}

pub fn addBlock(self: *Mmu, block_name: []const u8, len: usize) !usize {
    const start = self.size();

    const block = try self.allocator.create(Block);
    errdefer self.allocator.destroy(block);

    block.* = try Block.init(block_name, len, self.allocator);
    errdefer block.deinit();

    try self.blocks.append(block);
    try self.buses.append(block.bus());

    return start;
}

pub fn addBus(self: *Mmu, bus: Bus) !void {
    return self.buses.append(bus);
}

pub fn read(self: *Mmu, addr: usize, sz: DataSize) anyerror!Immediate {
    var start: usize = 0;
    for (self.buses.items) |*bus| {
        const end = start + bus.size();
        if (addr >= start and addr < end) {
            const offset = addr - start;
            return bus.read(offset, sz);
        }
        start = end;
    }
    return error.AddressOutOfBounds;
}

pub fn readSlice(self: *Mmu, addr: usize, len: usize) anyerror![]const u8 {
    var result = try self.allocator.alloc(u8, len);
    errdefer self.allocator.free(result);

    var bytes_read: usize = 0;
    var current_addr = addr;

    while (bytes_read < len) {
        var start: usize = 0;
        for (self.buses.items) |*bus| {
            const end = start + bus.size();
            if (current_addr >= start and current_addr < end) {
                const offset = current_addr - start;
                const remaining_in_bus = end - current_addr;
                const remaining_to_read = len - bytes_read;
                const to_read = @min(remaining_in_bus, remaining_to_read);

                const slice = try bus.readSlice(offset, offset + to_read);
                @memcpy(result[bytes_read .. bytes_read + to_read], slice);

                bytes_read += to_read;
                current_addr += to_read;
                break;
            }
            start = end;
        } else {
            self.allocator.free(result);
            return error.AddressOutOfBounds;
        }
    }

    try self.allocated_slices.append(result);
    return result;
}

pub fn write(self: *Mmu, addr: usize, value: Immediate, sz: DataSize) anyerror!void {
    var start: usize = 0;
    for (self.buses.items) |*bus| {
        const end = start + bus.size();
        if (addr >= start and addr < end) {
            const offset = addr - start;
            return bus.write(offset, value, sz);
        }
        start = end;
    }
    return error.AddressOutOfBounds;
}

pub fn writeSlice(self: *Mmu, addr: usize, data: []const u8) anyerror!void {
    var bytes_written: usize = 0;
    var current_addr = addr;

    while (bytes_written < data.len) {
        var start: usize = 0;
        for (self.buses.items) |*bus| {
            const end = start + bus.size();
            if (current_addr >= start and current_addr < end) {
                const offset = current_addr - start;
                const remaining_in_bus = end - current_addr;
                const remaining_to_write = data.len - bytes_written;
                const to_write = @min(remaining_in_bus, remaining_to_write);

                try bus.writeSlice(offset, data[bytes_written .. bytes_written + to_write]);

                bytes_written += to_write;
                current_addr += to_write;
                break;
            }
            start = end;
        } else {
            return error.AddressOutOfBounds;
        }
    }
}

pub fn size(self: *Mmu) usize {
    var sz: usize = 0;
    for (self.buses.items) |*bus| {
        sz += bus.size();
    }
    return sz;
}

pub fn debug(self: *Mmu) void {
    var start: usize = 0;
    for (self.buses.items) |*bus| {
        const end = start + bus.size();
        std.debug.print("{s} 0x{x:0>2}..0x{x:0>2}\n", .{
            bus.name(),
            start,
            end - 1,
        });
        start = end;
    }
}
