const std = @import("std");
const builtin = @import("builtin");
const Allocator = std.mem.Allocator;
const ast = @import("../parser/ast.zig");

pub fn getDefaultDefinitons(allocator: Allocator) !std.StringHashMap(*ast.Expression) {
    const arch = switch (builtin.cpu.arch) {
        .amdgcn => "__AMDGCN__",
        .arc => "__ARC__",
        .arm => "__ARM__",
        .armeb => "__ARMEB__",
        .thumb => "__THUMB__",
        .thumbeb => "__THUMBEB__",
        .aarch64 => "__AARCH64__",
        .aarch64_be => "__AARCH64_BE__",
        .avr => "__AVR__",
        .bpfel => "__BPFEL__",
        .bpfeb => "__BPFEB__",
        .csky => "__CSKY__",
        .hexagon => "__HEXAGON__",
        .kalimba => "__KALIMBA__",
        .lanai => "__LANAI__",
        .loongarch32 => "__LOONGARCH32__",
        .loongarch64 => "__LOONGARCH64__",
        .m68k => "__M68K__",
        .mips => "__MIPS__",
        .mipsel => "__MIPSEL__",
        .mips64 => "__MIPS64__",
        .mips64el => "__MIPS64EL__",
        .msp430 => "__MSP430__",
        .or1k => "__OR1K__",
        .nvptx => "__NVPTX__",
        .nvptx64 => "__NVPTX64__",
        .powerpc => "__POWERPC__",
        .powerpcle => "__POWERPCLE__",
        .powerpc64 => "__POWERPC64__",
        .powerpc64le => "__POWERPC64LE__",
        .propeller => "__PROPELLER__",
        .riscv32 => "__RISCV32__",
        .riscv64 => "__RISCV64__",
        .s390x => "__S390X__",
        .sparc => "__SPARC__",
        .sparc64 => "__SPARC64__",
        .spirv32 => "__SPIRV32__",
        .spirv64 => "__SPIRV64__",
        .ve => "__VE__",
        .wasm32 => "__WASM32__",
        .wasm64 => "__WASM64__",
        .x86 => "__X86__",
        .x86_64 => "__X86_64__",
        .xcore => "__XCORE__",
        .xtensa => "__XTENSA__",
    };

    const os = switch (builtin.os.tag) {
        .freestanding => "__FREESTANDING__",
        .other => "__OTHER__",
        .contiki => "__CONTIKI__",
        .fuchsia => "__FUCHSIA__",
        .hermit => "__HERMIT__",
        .aix => "__AIX__",
        .haiku => "__HAIKU__",
        .hurd => "__HURD__",
        .linux => "__LINUX__",
        .plan9 => "__PLAN9__",
        .rtems => "__RTEMS__",
        .serenity => "__SERENITY__",
        .zos => "__ZOS__",
        .dragonfly => "__DRAGONFLY__",
        .freebsd => "__FREEBSD__",
        .netbsd => "__NETBSD__",
        .openbsd => "__OPENBSD__",
        .driverkit => "__DRIVERKIT__",
        .ios => "__IOS__",
        .macos => "__MACOS__",
        .tvos => "__TVOS__",
        .visionos => "__VISIONOS__",
        .watchos => "__WATCHOS__",
        .illumos => "__ILLUMOS__",
        .solaris => "__SOLARIS__",
        .windows => "__WINDOWS__",
        .uefi => "__UEFI__",
        .ps3 => "__PS3__",
        .ps4 => "__PS4__",
        .ps5 => "__PS5__",
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

    var definitons = std.StringHashMap(*ast.Expression).init(allocator);

    const arch_expr = try allocator.create(ast.Expression);
    arch_expr.* = .{ .string_literal = "" };

    const os_expr = try allocator.create(ast.Expression);
    os_expr.* = .{ .string_literal = "" };

    try definitons.put(arch, arch_expr);
    try definitons.put(os, os_expr);

    return definitons;
}
