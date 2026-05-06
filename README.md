# Nyx

64-bit register-based virtual machine and compiler written in [Zig](https://ziglang.org/). Nyx supports compiling a custom assembly-like language into bytecode and executing it on a custom VM.

![Static Badge](https://img.shields.io/badge/Zig-0.16.0-ec915c?style=flat-square&logo=zig)
![Tests](https://img.shields.io/github/actions/workflow/status/ciathefed/nyx/zig.yml?label=Tests%20%F0%9F%A7%AA&style=flat-square)

> [!WARNING]
> Nyx is in very early development. It is incomplete, lightly tested, and many features are missing. Expect breaking changes and unstable behavior.

## Features

* Custom assembly-like language with a C-style preprocessor (`#define`, `#include`, `#macro`, conditionals)
* Compiler to bytecode (`.nyx` → `.nyb`)
* 64-bit register-based virtual machine with 16 general-purpose registers, 16 floating-point registers, and 3 special registers
* Block-based memory model with an MMU, stack, and dynamic allocation
* Syscalls for file I/O, memory management, and networking (sockets)
* C API for writing native extensions as shared libraries
* Standard library with string, print, and socket utilities
* Written in safe, modern [Zig](https://ziglang.org/)

## Installation

Clone the repo and build:

```sh
git clone https://github.com/ciathefed/nyx.git
cd nyx
zig build -Doptimize=ReleaseFast
```

This will create the binary at `zig-out/bin/nyx`.

## Usage

### Compile a source file to bytecode

```sh
nyx build _examples/hello.nyx -o hello.nyb
```

### Compile and execute source file directly

```sh
nyx run _examples/hello.nyx
```

You can also specify memory size (default is 65536 bytes):

```sh
nyx run _examples/hello.nyx -m 8192
```

### Run precompiled bytecode

```sh
nyx exec hello.nyb
```

With a custom memory size:

```sh
nyx exec hello.nyb -m 16384
```

### Additional options

```sh
# Add include search paths for #include directives
nyx build _examples/hello.nyx -i ./std -i ./my_includes

# Link a shared library for external function calls
nyx run _examples/raylib/main.nyx -l ./libbridge.so

# Disable the preprocessor
nyx build _examples/hello.nyx --disable-preprocessor
```

The `NYX_STDLIB_PATH` environment variable can be set to the standard library directory so `#include` resolves automatically.

## Example Program

```asm
.section text
_start:
    mov q0, 1           ; file descriptor (stdout)
    mov q1, message     ; pointer to message string
    mov q2, 14          ; number of bytes to write
    mov q15, 3          ; syscall number (sys_write)
    syscall
    hlt

.section data
message:
    db "Hello, world!\n", 0x00
```

More examples can be found in the [`_examples/`](./_examples) directory and in the [documentation](./docs/examples.md).

## Documentation

Detailed documentation lives in the [`docs/`](./docs) directory:

| Document | Description |
|----------|-------------|
| [Overview](./docs/overview.md) | Architecture, pipeline, project structure, CLI, and bytecode format |
| [Assembly Syntax](./docs/assembly-syntax.md) | Language syntax — sections, labels, literals, expressions, and directives |
| [Instruction Set](./docs/instructions.md) | Complete reference for all VM instructions |
| [Registers](./docs/registers.md) | GPRs, FPRs, special registers, encoding, and conventions |
| [Memory Model](./docs/memory.md) | MMU, blocks, stack, addressing modes, and data declarations |
| [Preprocessor](./docs/preprocessor.md) | `#define`, `#include`, conditionals, macros, and built-in definitions |
| [Syscalls](./docs/syscalls.md) | All syscalls with register-level input/output documentation |
| [Standard Library](./docs/standard-library.md) | `stdlib.nyx`, `string.nyx`, `print.nyx`, and `socket.nyx` |
| [C API](./docs/c-api.md) | Writing native extensions and the `nyx.h` C interface |
| [Examples](./docs/examples.md) | Annotated example programs |

## Contributing

Contributions are welcome. If you find a bug or want to add a feature, open an issue or pull request.

To contribute code:

1. Fork the repository
2. Create a new branch
3. Make your changes
4. Open a pull request with a clear description

Please follow the [Conventional Commits](https://www.conventionalcommits.org/) format for commit messages. Examples:

* `fix: handle empty source input in reporter`
* `feat: add support for multiple source files`
* `refactor: simplify diagnostic builder`

Keep changes focused and minimal. Include tests when appropriate.

## License

This project is licensed under the [MIT License](./LICENSE)
