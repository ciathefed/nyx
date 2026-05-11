# External Functions (FFI)

Nyx can call native C functions from shared libraries directly, with no wrapper
code, no bridge library, and no glue. The VM uses [libffi](https://github.com/libffi/libffi)
to marshal arguments and return values at runtime.

## Quick Start

**1. Declare the function with its type signature:**

```nyx
.extern puts(ptr): i32
```

**2. Call it like any other function:**

```nyx
mov q0, message
call puts
```

**3. Run with the shared library:**

```sh
nyx run hello.nyx -l /usr/lib/libSystem.B.dylib    # macOS
nyx run hello.nyx -l /lib/x86_64-linux-gnu/libc.so.6  # Linux
```

That's it. No C code to write, no Makefile, no bridge library.

---

## `.extern` Declaration Syntax

```
.extern NAME(PARAM_TYPES...): RETURN_TYPE
```

| Part | Description |
|------|-------------|
| `NAME` | The symbol name as exported by the shared library. |
| `PARAM_TYPES` | Comma-separated list of parameter types. |
| `RETURN_TYPE` | The function's return type. |

### Supported Types

| Type | Size | Description | Register prefix |
|------|------|-------------|-----------------|
| `i8` | 1 byte | 8-bit integer | `b` |
| `i16` | 2 bytes | 16-bit integer | `w` |
| `i32` | 4 bytes | 32-bit integer | `d` |
| `i64` | 8 bytes | 64-bit integer | `q` |
| `f32` | 4 bytes | 32-bit float | `ff` |
| `f64` | 8 bytes | 64-bit float | `dd` |
| `ptr` | 8 bytes | Pointer (VM address → host pointer) | `q` |
| `void` | — | No value (return type only) | — |

### Examples

```nyx
.extern puts(ptr): i32
.extern InitWindow(i32, i32, ptr): void
.extern SetTargetFPS(i32): void
.extern sin(f64): f64
.extern CloseWindow(): void
```

---

## Calling Convention

When the VM executes a `call` to an extern function, it reads arguments from
registers following this convention:

### Integer and Pointer Arguments

| Argument # | Register |
|------------|----------|
| 1 | `q0` / `d0` / `w0` / `b0` (width matches declared type) |
| 2 | `q1` / `d1` / `w1` / `b1` |
| 3 | `q2` / `d2` / `w2` / `b2` |
| 4 | `q3` / `d3` / `w3` / `b3` |
| 5 | `q4` / `d4` / `w4` / `b4` |
| 6 | `q5` / `d5` / `w5` / `b5` |
| 7+ | Stack (pushed right-to-left) |

### Float Arguments

| Argument # | Register |
|------------|----------|
| 1 | `ff0` / `dd0` (width matches declared type) |
| 2 | `ff1` / `dd1` |
| 3 | `ff2` / `dd2` |
| 4 | `ff3` / `dd3` |
| 5 | `ff4` / `dd4` |
| 6 | `ff5` / `dd5` |
| 7+ | Stack (pushed right-to-left) |

Integer and float arguments use **separate register pools**. A function with
3 integer args and 2 float args uses `q0`–`q2` and `ff0`–`ff1`.

### Return Values

| Return type | Register |
|-------------|----------|
| `i8` | `b0` |
| `i16` | `w0` |
| `i32` | `d0` |
| `i64` / `ptr` | `q0` |
| `f32` | `ff0` |
| `f64` | `dd0` |
| `void` | — |

### Pointer Translation

When a `ptr` argument is passed, the VM automatically translates the virtual
memory address (stored in the register) to a real host pointer that the native
function can dereference. This means you can pass label addresses directly:

```nyx
mov q0, my_string    ; VM address of the string
call puts            ; puts receives a real const char*
```

---

## Loading Libraries

Shared libraries are loaded at runtime with the `-l` CLI flag:

```sh
nyx run program.nyx -l /path/to/library.dylib
nyx run program.nyx -l /path/to/library.so
```

Multiple libraries can be loaded:

```sh
nyx run program.nyx -l /usr/lib/libSystem.B.dylib -l /usr/local/lib/libraylib.dylib
```

The VM searches all loaded libraries for each extern symbol. The `-l` flag is
only needed at **execution time**. It is accepted by `run` and `exec`, but
not `build`.

---

## Complete Example

A minimal program that calls `puts` from libc:

```nyx
.extern puts(ptr): i32

#include "stdlib.nyx"

.section text
_start:
    mov q0, message
    call puts
    hlt

.section data
message:
    .asciz "Hello from Nyx via libffi!"
```

Run it:

```sh
# macOS
nyx run hello.nyx -l /usr/lib/libSystem.B.dylib

# Linux
nyx run hello.nyx -l /lib/x86_64-linux-gnu/libc.so.6
```

---

## Real-World Example: Raylib

Calling raylib functions directly, no C bridge needed:

```nyx
.extern InitWindow(i32, i32, ptr): void
.extern CloseWindow(): void
.extern SetTargetFPS(i32): void
.extern WindowShouldClose(): i32
.extern BeginDrawing(): void
.extern EndDrawing(): void
.extern ClearBackground(i32): void
.extern DrawText(ptr, i32, i32, i32, i32): void

#include "stdlib.nyx"

.section text
_start:
    mov d0, 800
    mov d1, 450
    mov q2, title
    call InitWindow

    mov d0, 60
    call SetTargetFPS

.loop:
    call BeginDrawing

        mov d0, 0xFFF5F5F5
        call ClearBackground

        mov q0, message
        mov d1, 190
        mov d2, 200
        mov d3, 20
        mov d4, 0xFFC8C8C8
        call DrawText

    call EndDrawing

    call WindowShouldClose
    cmp d0, 1
    jne .loop

    call CloseWindow
    hlt

.section data
title:      .asciz "raylib [core] example - basic window"
message:    .asciz "Congrats! You created your first window!"
```

Run it:

```sh
nyx run raylib_demo.nyx -l /usr/local/lib/libraylib.dylib
```

---

## Advanced: The C API (nyx.h)

The C header `include/nyx.h` and the shared library `libnyx` still exist for
**embedding** the Nyx VM inside a C/C++ application. Functions like
`vm_get_reg_int`, `vm_mem_read_cstr`, etc. let a host program inspect and
manipulate the VM state.

However, **you do not need the C API for external function calls**. The libffi
system handles everything automatically. The C API is only relevant if you are
building a program that creates and drives a `Vm` instance from C code.

---

## Limitations

- **Struct arguments/returns** are not yet supported. Use packed integers or
  helper functions as a workaround.
- **Variadic functions** (e.g. `printf`) are not yet supported. Use
  non-variadic alternatives like `puts` or `fputs`.
- **Maximum 64 arguments** per extern call.
