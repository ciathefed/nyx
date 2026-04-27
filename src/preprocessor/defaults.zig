const std = @import("std");
const builtin = @import("builtin");
const Allocator = std.mem.Allocator;
const StringInterner = @import("../StringInterner.zig");
const StringId = StringInterner.StringId;
const ast = @import("../parser/ast.zig");

pub fn getDefaultDefinitions(gpa: Allocator, interner: *StringInterner) !std.AutoHashMap(StringId, *ast.Expression) {
    const arch = switch (builtin.cpu.arch) {
        .aarch64 => "__AARCH64__",
        .aarch64_be => "__AARCH64_BE__",
        .alpha => "__ALPHA__",
        .amdgcn => "__AMDGCN__",
        .arc => "__ARC__",
        .arceb => "__ARCEB__",
        .arm => "__ARM__",
        .armeb => "__ARMEB__",
        .avr => "__AVR__",
        .bpfeb => "__BPFEB__",
        .bpfel => "__BPFEL__",
        .csky => "__CSKY__",
        .hexagon => "__HEXAGON__",
        .hppa => "__HPPA__",
        .hppa64 => "__HPPA64__",
        .kalimba => "__KALIMBA__",
        .kvx => "__KVX__",
        .lanai => "__LANAI__",
        .loongarch32 => "__LOONGARCH32__",
        .loongarch64 => "__LOONGARCH64__",
        .m68k => "__M68K__",
        .microblaze => "__MICROBLAZE__",
        .microblazeel => "__MICROBLAZEEL__",
        .mips => "__MIPS__",
        .mipsel => "__MIPSEL__",
        .mips64 => "__MIPS64__",
        .mips64el => "__MIPS64EL__",
        .msp430 => "__MSP430__",
        .nvptx => "__NVPTX__",
        .nvptx64 => "__NVPTX64__",
        .or1k => "__OR1K__",
        .powerpc => "__POWERPC__",
        .powerpcle => "__POWERPCLE__",
        .powerpc64 => "__POWERPC64__",
        .powerpc64le => "__POWERPC64LE__",
        .propeller => "__PROPELLER__",
        .riscv32 => "__RISCV32__",
        .riscv32be => "__RISCV32BE__",
        .riscv64 => "__RISCV64__",
        .riscv64be => "__RISCV64BE__",
        .s390x => "__S390X__",
        .sh => "__SH__",
        .sheb => "__SHEB__",
        .sparc => "__SPARC__",
        .sparc64 => "__SPARC64__",
        .spirv32 => "__SPIRV32__",
        .spirv64 => "__SPIRV64__",
        .thumb => "__THUMB__",
        .thumbeb => "__THUMBEB__",
        .ve => "__VE__",
        .wasm32 => "__WASM32__",
        .wasm64 => "__WASM64__",
        .x86_16 => "__X86_16__",
        .x86 => "__X86__",
        .x86_64 => "__X86_64__",
        .xcore => "__XCORE__",
        .xtensa => "__XTENSA__",
        .xtensaeb => "__XTENSAEB__",
    };

    const os = switch (builtin.os.tag) {
        .freestanding => "__FREESTANDING__",
        .other => "__OTHER__",
        .contiki => "__CONTIKI__",
        .fuchsia => "__FUCHSIA__",
        .hermit => "__HERMIT__",
        .managarm => "__MANAGARM__",
        .haiku => "__HAIKU__",
        .hurd => "__HURD__",
        .illumos => "__ILLUMOS__",
        .linux => "__LINUX__",
        .plan9 => "__PLAN9__",
        .rtems => "__RTEMS__",
        .serenity => "__SERENITY__",
        .dragonfly => "__DRAGONFLY__",
        .freebsd => "__FREEBSD__",
        .netbsd => "__NETBSD__",
        .openbsd => "__OPENBSD__",
        .driverkit => "__DRIVERKIT__",
        .ios => "__IOS__",
        .maccatalyst => "__MACCATALYST__",
        .macos => "__MACOS__",
        .tvos => "__TVOS__",
        .visionos => "__VISIONOS__",
        .watchos => "__WATCHOS__",
        .windows => "__WINDOWS__",
        .uefi => "__UEFI__",
        .@"3ds" => "__3DS__",
        .ps3 => "__PS3__",
        .ps4 => "__PS4__",
        .ps5 => "__PS5__",
        .psp => "__PSP__",
        .vita => "__VITA__",
        .emscripten => "__EMSCRIPTEN__",
        .wasi => "__WASI__",
        .amdhsa => "__AMDHSA__",
        .amdpal => "__AMDPAL__",
        .cuda => "__CUDA__",
        .mesa3d => "__MESA3D__",
        .nvcl => "__NVCL__",
        .opencl => "__OPENCL__",
        .opengl => "__OPENGL__",
        .vulkan => "__VULKAN__",
    };

    var definitions = std.AutoHashMap(StringId, *ast.Expression).init(gpa);

    const arch_id = try interner.intern(arch);
    const arch_expr = try gpa.create(ast.Expression);
    const empty_string_id = try interner.intern("");
    arch_expr.* = .{ .string_literal = empty_string_id };

    const os_id = try interner.intern(os);
    const os_expr = try gpa.create(ast.Expression);
    os_expr.* = .{ .string_literal = empty_string_id };

    try definitions.put(arch_id, arch_expr);
    try definitions.put(os_id, os_expr);

    return definitions;
}
