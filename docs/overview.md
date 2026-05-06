# Nyx — Project Overview

Nyx is a 64-bit register-based virtual machine and compiler written in Zig (0.16.0). It compiles a custom assembly-like language into bytecode and executes it on a custom VM.

## Architecture Overview

The compilation and execution pipeline is:

```/dev/null/pipeline.txt#L1
Source Code (.nyx) → Preprocessor → Lexer → Parser → Compiler → Bytecode (.nyb) → Virtual Machine
```

### Preprocessor (`src/preprocessor/`)

The preprocessor runs first over the raw source text. It handles:

- `#define` — constant and macro definitions
- `#include` — file inclusion
- `#ifdef` / `#ifndef` / `#else` / `#endif` — conditional compilation
- `#macro` / `#endm` — multi-line macro definitions
- `#error` — user-triggered compilation errors

It also injects platform-specific definitions automatically (e.g. `__LINUX__`, `__X86_64__`), allowing source code to branch on the host platform.

### Lexer (`src/lexer/`)

The lexer tokenizes preprocessed source into a stream of tokens. Token types include identifiers, registers, integers (decimal, hex, binary, octal), floats, strings, instruction keywords, directives, data-size specifiers, and punctuation.

### Parser (`src/parser/`)

The parser consumes the token stream and builds an abstract syntax tree (AST). Statements in the AST represent labels, directives, and instructions with their operands. Operand expressions support unary and binary operators.

### Compiler (`src/compiler/`)

The compiler walks the AST and emits bytecode. It:

- Resolves label references to concrete addresses, applying fixups for forward references.
- Emits opcodes followed by encoded operands.
- Organizes output into two sections: `.text` (executable code) and `.data` (static data).
- Writes the final bytecode file: an 8-byte entry point address (little-endian `u64`), followed by the text section, then the data section.

### Virtual Machine (`src/vm/`)

The VM loads and executes compiled bytecode. Key components:

- **Registers** — 16 general-purpose registers, 16 floating-point registers, and 3 special-purpose registers (stack pointer, instruction pointer, flags).
- **MMU** — Block-based memory management. The address space is divided into a Program block (loaded bytecode), a Memory block (general-purpose RAM), and dynamically allocated blocks.
- **Stack** — Grows downward from the top of the memory block.
- **Flags** — Condition flags (`eq`, `lt`) set by comparison instructions.
- **Syscalls** — Built-in system call interface for I/O and OS interaction.
- **External library loading** — Supports loading shared libraries at runtime.

## Project Structure

| Directory | Description |
|---|---|
| `src/` | Main source code |
| `src/vm/` | Virtual machine — `Vm.zig`, `register.zig`, `syscall.zig`, `Flags.zig`, `ExternalLoader.zig` |
| `src/vm/memory/` | MMU, Block, Bus (vtable-based memory bus abstraction) |
| `src/compiler/` | Compiler — `Compiler.zig`, `Bytecode.zig`, `opcode.zig` |
| `src/lexer/` | Lexer — `Lexer.zig`, `Token.zig` |
| `src/parser/` | Parser — `Parser.zig`, `ast.zig`, `immediate.zig` |
| `src/preprocessor/` | Preprocessor — `Preprocessor.zig`, `defaults.zig` |
| `std/` | Standard library includes — `stdlib.nyx`, `string.nyx`, `print.nyx`, `socket.nyx` |
| `_examples/` | Example programs |
| `include/` | C API header (`nyx.h`) |

## CLI Usage

Nyx provides three subcommands:

### `build` — Compile source to bytecode

```/dev/null/usage.txt#L1
nyx build <FILE> [-o output] [-i include_dir] [--disable-preprocessor]
```

### `exec` — Execute a compiled bytecode file

```/dev/null/usage.txt#L1
nyx exec <FILE> [-l library] [-m memory_size]
```

### `run` — Compile and execute in one step

```/dev/null/usage.txt#L1
nyx run <FILE> [-o output] [-l library] [-i include_dir] [-m memory_size] [--disable-preprocessor]
```

### Defaults

- **Output file** — `out.nyb`
- **Memory size** — 65536 bytes
- **Standard library path** — Set the `NYX_STDLIB_PATH` environment variable to point to the standard library directory.

## Bytecode Format

Compiled bytecode is stored in `.nyb` files with the following binary layout:

| Offset | Size | Content |
|---|---|---|
| 0 | 8 bytes | Entry point address (`u64`, little-endian) |
| 8 | variable | Text section (executable code) |
| 8 + len(text) | variable | Data section (static data) |

The VM reads the entry point to determine where execution begins, loads the text and data sections into memory, and starts executing from the entry point address.
