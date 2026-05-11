const std = @import("std");
const process = std.process;
const fmt = std.fmt;
const fs = std.fs;
const Allocator = std.mem.Allocator;
const ArrayList = std.array_list.Managed;
const fehler = @import("fehler");
const yazap = @import("yazap");
const StringInterner = @import("StringInterner.zig");
const Lexer = @import("lexer/Lexer.zig");
const Parser = @import("parser/Parser.zig");
const Compiler = @import("compiler/Compiler.zig");
const Vm = @import("vm/Vm.zig");
const Preprocessor = @import("preprocessor/Preprocessor.zig");
const utils = @import("utils.zig");

pub fn main(init: std.process.Init) !void {
    var app = yazap.App.init(init.gpa, "nyx", "A compiler and virtual machine for the Nyx assembly language");
    defer app.deinit();

    var nyx = app.rootCommand();
    nyx.setProperty(.subcommand_required);
    nyx.setProperty(.help_on_empty_args);

    try nyx.addSubcommand(try createBuildCommand(&app));
    try nyx.addSubcommand(try createExecCommand(&app));
    try nyx.addSubcommand(try createRunCommand(&app));

    const matches = try app.parseProcess(init.io, init.minimal.args);

    var reporter = fehler.ErrorReporter.init(init.gpa);
    defer reporter.deinit();

    if (matches.subcommandMatches("build")) |build_cmd_matches| {
        try executeBuildCommand(init.io, init.minimal.environ, init.gpa, build_cmd_matches, &reporter);
    }

    if (matches.subcommandMatches("exec")) |exec_cmd_matches| {
        try executeExecCommand(init.io, init.gpa, exec_cmd_matches, &reporter);
    }

    if (matches.subcommandMatches("run")) |run_cmd_matches| {
        try executeRunCommand(init.io, init.minimal.environ, init.gpa, run_cmd_matches, &reporter);
    }
}

fn createBuildCommand(app: *yazap.App) !yazap.Command {
    var build_cmd = app.createCommand("build", "Compile source code to bytecode");
    try build_cmd.addArgs(&.{
        yazap.Arg.positional("FILE", "Path to the source file to compile", null),
        yazap.Arg.singleValueOption("output", 'o', "Optional path to write the compiled bytecode output"),
        yazap.Arg.multiValuesOption("include", 'i', "Adds an include directory to the search path", 65536),
        yazap.Arg.booleanOption("disable-preprocessor", null, "Stop the preprocessor from running"),
    });
    build_cmd.setProperty(.positional_arg_required);
    build_cmd.setProperty(.help_on_empty_args);
    return build_cmd;
}

fn createExecCommand(app: *yazap.App) !yazap.Command {
    var exec_cmd = app.createCommand("exec", "Execute existing bytecode in the virtual machine");
    try exec_cmd.addArgs(&.{
        yazap.Arg.positional("FILE", "Path to the precompiled bytecode file to execute", null),
        yazap.Arg.multiValuesOption("library", 'l', "Link a dynamic libraries", 65536),
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
        yazap.Arg.multiValuesOption("library", 'l', "Link a dynamic libraries", 65536),
        yazap.Arg.multiValuesOption("include", 'i', "Adds an include directory to the search path", 65536),
        yazap.Arg.singleValueOption("memory-size", 'm', "Size of virtual machine memory in bytes"),
        yazap.Arg.booleanOption("disable-preprocessor", null, "Stop the preprocessor from running"),
    });
    run_cmd.setProperty(.positional_arg_required);
    run_cmd.setProperty(.help_on_empty_args);
    return run_cmd;
}

fn compileSourceFile(
    io: std.Io,
    env: std.process.Environ,
    gpa: Allocator,
    input_file_path: []const u8,
    include_paths: []const []const u8,
    run_preprocessor: bool,
    reporter: *fehler.ErrorReporter,
) ![]const u8 {
    if (!utils.fileExists(io, input_file_path)) {
        logError(reporter, "{s}: cannot find file", .{input_file_path});
        process.exit(1);
    }

    const input = try utils.readFromFile(io, gpa, input_file_path);
    defer gpa.free(input);

    try reporter.addSource(input_file_path, input);

    var interner = StringInterner.init(gpa);
    defer interner.deinit();

    var lexer = Lexer.init(input_file_path, input, &interner, gpa);

    var parser = Parser.init(&lexer, reporter, gpa);
    defer parser.deinit();

    const stmts = try parser.parse();

    var all_include_paths = ArrayList([]const u8).init(gpa);
    try all_include_paths.append("");
    try all_include_paths.append(fs.path.basename(input_file_path));
    try all_include_paths.appendSlice(include_paths);
    const stdlib_path = env.getAlloc(gpa, "NYX_STDLIB_PATH") catch |err| switch (err) {
        error.EnvironmentVariableMissing => null,
        else => return err,
    };
    if (stdlib_path) |path| try all_include_paths.append(path);
    defer if (stdlib_path) |path| gpa.free(path);

    var preprocessor: ?Preprocessor = if (run_preprocessor)
        try Preprocessor.init(
            io,
            gpa,
            input_file_path,
            input,
            stmts,
            &interner,
            reporter,
            try all_include_paths.toOwnedSlice(),
        )
    else
        null;
    defer if (preprocessor) |*p| p.deinit();

    const new_stmts = if (preprocessor) |*p|
        try p.process()
    else
        stmts;

    var compiler = try Compiler.init(
        new_stmts,
        &interner,
        input_file_path,
        input,
        reporter,
        gpa,
    );
    defer compiler.deinit();

    return try compiler.compile();
}

fn runBytecode(
    bytecode: []const u8,
    external_libraries: [][]const u8,
    memory_size: usize,
    gpa: Allocator,
) !void {
    var vm = try Vm.init(bytecode, memory_size, external_libraries, gpa);
    defer vm.deinit();
    try vm.run();
}

fn executeBuildCommand(
    io: std.Io,
    env: std.process.Environ,
    gpa: Allocator,
    matches: yazap.ArgMatches,
    reporter: *fehler.ErrorReporter,
) !void {
    const input_file_path = matches.getSingleValue("FILE").?;
    const output_file_path = if (matches.getSingleValue("output")) |output| output else "out.nyb";
    const include_paths = matches.getMultiValues("include") orelse &.{};
    const run_preprocessor = !matches.containsArg("disable-preprocessor");

    const bytecode = try compileSourceFile(
        io,
        env,
        gpa,
        input_file_path,
        include_paths,
        run_preprocessor,
        reporter,
    );
    defer gpa.free(bytecode);

    try utils.writeToFile(io, output_file_path, bytecode);
}

fn executeExecCommand(
    io: std.Io,
    gpa: Allocator,
    matches: yazap.ArgMatches,
    reporter: *fehler.ErrorReporter,
) !void {
    const input_file_path = matches.getSingleValue("FILE").?;
    const external_libraries: [][]const u8 = matches.getMultiValues("library") orelse &.{};
    const memory_size = if (matches.getSingleValue("memory-size")) |size|
        fmt.parseInt(usize, size, 10) catch {
            logError(reporter, "{s}: not a valid number", .{size});
            process.exit(1);
        }
    else
        65536;

    const bytecode = try utils.readFromFile(io, gpa, input_file_path);
    defer gpa.free(bytecode);

    try runBytecode(bytecode, external_libraries, memory_size, gpa);
}

fn executeRunCommand(
    io: std.Io,
    env: std.process.Environ,
    gpa: Allocator,
    matches: yazap.ArgMatches,
    reporter: *fehler.ErrorReporter,
) !void {
    const input_file_path = matches.getSingleValue("FILE").?;
    const output_file_path = if (matches.getSingleValue("output")) |output| output else null;
    const external_libraries: [][]const u8 = matches.getMultiValues("library") orelse &.{};
    const include_paths = matches.getMultiValues("include") orelse &.{};
    const memory_size = if (matches.getSingleValue("memory-size")) |size|
        fmt.parseInt(usize, size, 10) catch {
            logError(reporter, "{s}: not a valid number", .{size});
            process.exit(1);
        }
    else
        65536;
    const run_preprocessor = !matches.containsArg("disable-preprocessor");

    const bytecode = try compileSourceFile(
        io,
        env,
        gpa,
        input_file_path,
        include_paths,
        run_preprocessor,
        reporter,
    );
    defer gpa.free(bytecode);

    if (output_file_path) |path| {
        try utils.writeToFile(io, path, bytecode);
    }

    try runBytecode(bytecode, external_libraries, memory_size, gpa);
}

fn logError(reporter: *fehler.ErrorReporter, comptime format: []const u8, args: anytype) void {
    const message = std.fmt.allocPrint(std.heap.page_allocator, format, args) catch unreachable;
    reporter.report(.{ .severity = .err, .message = message });
}
