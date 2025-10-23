// TODO: add tests

const std = @import("std");
const process = std.process;
const fmt = std.fmt;
const fs = std.fs;
const Allocator = std.mem.Allocator;
const ArrayList = std.array_list.Managed;
const fehler = @import("fehler");
const yazap = @import("yazap");
const Lexer = @import("lexer/Lexer.zig");
const Parser = @import("parser/Parser.zig");
const Compiler = @import("compiler/Compiler.zig");
const Vm = @import("vm/Vm.zig");
const Preprocessor = @import("preprocessor/Preprocessor.zig");
const utils = @import("utils.zig");

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}).init;
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var app = yazap.App.init(allocator, "nyx", "A compiler and virtual machine for the Nyx assembly language");
    defer app.deinit();

    var nyx = app.rootCommand();
    nyx.setProperty(.subcommand_required);
    nyx.setProperty(.help_on_empty_args);

    try nyx.addSubcommand(try createBuildCommand(&app));
    try nyx.addSubcommand(try createExecCommand(&app));
    try nyx.addSubcommand(try createRunCommand(&app));

    const matches = try app.parseProcess();

    var reporter = fehler.ErrorReporter.init(allocator);
    defer reporter.deinit();

    if (matches.subcommandMatches("build")) |build_cmd_matches| {
        try executeBuildCommand(build_cmd_matches, &reporter, allocator);
    }

    if (matches.subcommandMatches("exec")) |exec_cmd_matches| {
        try executeExecCommand(exec_cmd_matches, &reporter, allocator);
    }

    if (matches.subcommandMatches("run")) |run_cmd_matches| {
        try executeRunCommand(run_cmd_matches, &reporter, allocator);
    }
}

fn createBuildCommand(app: *yazap.App) !yazap.Command {
    var build_cmd = app.createCommand("build", "Compile source code to bytecode");
    try build_cmd.addArgs(&.{
        yazap.Arg.positional("FILE", "Path to the source file to compile", null),
        yazap.Arg.singleValueOption("output", 'o', "Optional path to write the compiled bytecode output"),
    });
    build_cmd.setProperty(.positional_arg_required);
    build_cmd.setProperty(.help_on_empty_args);
    return build_cmd;
}

fn createExecCommand(app: *yazap.App) !yazap.Command {
    var exec_cmd = app.createCommand("exec", "Execute existing bytecode in the virtual machine");
    try exec_cmd.addArgs(&.{
        yazap.Arg.positional("FILE", "Path to the precompiled bytecode file to execute", null),
        yazap.Arg.multiValuesOption("library", 'l', "Link a dynamic librarie", 65536),
        yazap.Arg.singleValueOption("memory-size", 'm', "Size of virtual machine memory in bytes"),
    });
    exec_cmd.setProperty(.positional_arg_required);
    exec_cmd.setProperty(.help_on_empty_args);
    return exec_cmd;
}

fn createRunCommand(app: *yazap.App) !yazap.Command {
    var run_cmd = app.createCommand("run", "Compile and execute source code in the virtual machine");
    try run_cmd.addArgs(&.{
        yazap.Arg.positional("FILE", "Path to the source file to compile and execute", null),
        yazap.Arg.singleValueOption("output", 'o', "Optional path to write the compiled bytecode output"),
        yazap.Arg.multiValuesOption("library", 'l', "Link a dynamic librarie", 65536),
        yazap.Arg.singleValueOption("memory-size", 'm', "Size of virtual machine memory in bytes"),
    });
    run_cmd.setProperty(.positional_arg_required);
    run_cmd.setProperty(.help_on_empty_args);
    return run_cmd;
}

fn compileSourceFile(
    input_file_path: []const u8,
    reporter: *fehler.ErrorReporter,
    allocator: Allocator,
) ![]const u8 {
    if (!utils.fileExists(input_file_path)) {
        logError(reporter, "{s}: cannot find file", .{input_file_path});
        process.exit(1);
    }

    const input = try utils.readFromFile(input_file_path, allocator);
    defer allocator.free(input);

    try reporter.addSource(input_file_path, input);

    var lexer = Lexer.init(input_file_path, input, allocator);
    defer lexer.deinit();

    var parser = Parser.init(&lexer, reporter, allocator);
    defer parser.deinit();

    const stmts = try parser.parse();

    // TODO: add -I arg so we can add other paths
    var include_paths = ArrayList([]const u8).init(allocator);
    try include_paths.append("");
    try include_paths.append(fs.path.basename(input_file_path));
    const stdlib_path = std.process.getEnvVarOwned(allocator, "NYX_STDLIB_PATH") catch |err| switch (err) {
        error.EnvironmentVariableNotFound => null,
        else => return err,
    };
    if (stdlib_path) |path| try include_paths.append(path);
    defer if (stdlib_path) |path| allocator.free(path);

    // TODO: add flag to disable preprocessing
    var preprocessor = try Preprocessor.init(
        input_file_path,
        input,
        stmts,
        reporter,
        try include_paths.toOwnedSlice(),
        allocator,
    );
    defer preprocessor.deinit();

    const new_stmts = try preprocessor.process();

    var compiler = try Compiler.init(
        new_stmts,
        input_file_path,
        input,
        reporter,
        allocator,
    );
    defer compiler.deinit();

    return try compiler.compile();
}

fn runBytecode(
    bytecode: []const u8,
    external_libraires: [][]const u8,
    memory_size: usize,
    allocator: Allocator,
) !void {
    var vm = try Vm.init(bytecode, memory_size, external_libraires, allocator);
    defer vm.deinit();
    try vm.run();
}

fn executeBuildCommand(
    matches: yazap.ArgMatches,
    reporter: *fehler.ErrorReporter,
    allocator: Allocator,
) !void {
    const input_file_path = matches.getSingleValue("FILE").?;
    const output_file_path = if (matches.getSingleValue("output")) |output| output else "out.nyb";

    const bytecode = try compileSourceFile(input_file_path, reporter, allocator);
    defer allocator.free(bytecode);

    try utils.writeToFile(output_file_path, bytecode);
}

fn executeExecCommand(
    matches: yazap.ArgMatches,
    reporter: *fehler.ErrorReporter,
    allocator: Allocator,
) !void {
    const input_file_path = matches.getSingleValue("FILE").?;
    const external_libraires: [][]const u8 = matches.getMultiValues("library") orelse &.{};
    const memory_size = if (matches.getSingleValue("memory-size")) |size|
        fmt.parseInt(usize, size, 10) catch {
            logError(reporter, "{s}: not a valid number", .{size});
            process.exit(1);
        }
    else
        65536;

    const bytecode = try utils.readFromFile(input_file_path, allocator);
    defer allocator.free(bytecode);

    try runBytecode(bytecode, external_libraires, memory_size, allocator);
}

fn executeRunCommand(
    matches: yazap.ArgMatches,
    reporter: *fehler.ErrorReporter,
    allocator: Allocator,
) !void {
    const input_file_path = matches.getSingleValue("FILE").?;
    const output_file_path = if (matches.getSingleValue("output")) |output| output else null;
    const external_libraires: [][]const u8 = matches.getMultiValues("library") orelse &.{};
    const memory_size = if (matches.getSingleValue("memory-size")) |size|
        fmt.parseInt(usize, size, 10) catch {
            logError(reporter, "{s}: not a valid number", .{size});
            process.exit(1);
        }
    else
        65536;

    const bytecode = try compileSourceFile(input_file_path, reporter, allocator);
    defer allocator.free(bytecode);

    if (output_file_path) |path| {
        try utils.writeToFile(path, bytecode);
    }

    try runBytecode(bytecode, external_libraires, memory_size, allocator);
}

fn logError(reporter: *fehler.ErrorReporter, comptime format: []const u8, args: anytype) void {
    const message = std.fmt.allocPrint(std.heap.page_allocator, format, args) catch unreachable;
    reporter.report(.{ .severity = .err, .message = message });
}
