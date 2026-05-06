# Preprocessor

The preprocessor runs before compilation and handles C-like directives over the raw source text. It can be disabled entirely with the `--disable-preprocessor` CLI flag.

## Directives

### `#define NAME [value]`

Define a preprocessor constant. If `value` is omitted, the symbol is defined but has no substitution value (useful for `#ifdef` guards).

The value can be an integer literal, float, string, or a constant expression.

```/dev/null/example.nyx#L1-4
#define MAX_SIZE 1024
#define PI 3.14159
#define GREETING "hello"
#define FEATURE_FLAG
```

### `#include "file.nyx"`

Include another source file. The preprocessor searches for the file in the following locations, in order:

1. The current working directory
2. The directory of the source file containing the `#include`
3. Any directories passed with the `-i` / `--include` CLI flag
4. The directory specified by the `NYX_STDLIB_PATH` environment variable

```/dev/null/example.nyx#L1-2
#include "stdlib.nyx"
#include "mylib.nyx"
```

### `#ifdef` / `#ifndef` ... `#else` ... `#endif`

Conditional compilation. Code between the directives is included or excluded based on whether a symbol is defined.

```/dev/null/example.nyx#L1-7
#ifdef __LINUX__
    mov q0, 1
#else
    mov q0, 2
#endif

#ifndef DEBUG
    ; release-mode code
#endif
```

### `#macro NAME ($param1, $param2, ...) ... #endm`

Define a multi-line macro with parameters. Parameters are prefixed with `$` in both the declaration and the body. When the macro is invoked, the body is expanded with actual arguments substituted in place of the parameters.

```/dev/null/example.nyx#L1-6
#macro write ($fd, $addr, $len)
    mov q0, $fd
    mov q1, $addr
    mov q2, $len
    mov q15, SYS_WRITE
    syscall
#endm
```

A macro is invoked by name, with arguments separated by commas:

```/dev/null/example.nyx#L1
write STDOUT, message, 14
```

Macros can contain any valid statements — instructions, directives, labels, etc.

### `#error "message"`

Emit a compile-time error with the given message. Useful for guarding against unsupported configurations.

```/dev/null/example.nyx#L1-3
#ifndef __LINUX__
#error "This program only supports Linux"
#endif
```

## Built-in Definitions

The preprocessor automatically defines platform-specific symbols based on the build target. These are available without any explicit `#define`.

**Architecture:**

| Symbol | Platform |
|---|---|
| `__X86_64__` | x86-64 |
| `__AARCH64__` | AArch64 / ARM64 |
| `__RISCV64__` | RISC-V 64-bit |
| `__ARM__` | ARM 32-bit |
| `__WASM32__` | WebAssembly 32-bit |

**Operating System:**

| Symbol | Platform |
|---|---|
| `__LINUX__` | Linux |
| `__MACOS__` | macOS |
| `__WINDOWS__` | Windows |
| `__FREEBSD__` | FreeBSD |

## Expression Evaluation

The preprocessor can evaluate constant expressions in `#define` values. Supported operators:

| Operator | Description |
|---|---|
| `+` `-` `*` `/` | Arithmetic |
| `\|` `&` `^` | Bitwise OR, AND, XOR |
| `-` (unary) | Negation |
| `(` `)` | Grouping |

```/dev/null/example.nyx#L1-3
#define PAGE_SIZE 4096
#define PAGE_MASK (PAGE_SIZE - 1)
#define FLAGS (0x01 | 0x04)
```

## Include Guards

The standard pattern for preventing duplicate inclusion uses `#ifndef` / `#define` / `#endif`:

```/dev/null/example.nyx#L1-5
#ifndef MYLIB_NYX
#define MYLIB_NYX

; ... library content ...

#endif
```

## Full Example

```/dev/null/example.nyx#L1-17
#include "stdlib.nyx"

#macro write ($fd, $addr, $len)
    mov q0, $fd
    mov q1, $addr
    mov q2, $len
    mov q15, SYS_WRITE
    syscall
#endm

.section text
_start:
    write STDOUT, message, 14
    hlt

.section data
message:
    db "Hello, world!\n", 0x00
```

## Disabling the Preprocessor

Pass `--disable-preprocessor` to skip preprocessing entirely:

```/dev/null/usage.txt#L1-2
nyx build main.nyx --disable-preprocessor
nyx run main.nyx --disable-preprocessor
```

When disabled, no directives are processed and the source is passed directly to the lexer.
