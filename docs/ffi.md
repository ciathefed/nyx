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
| `ptr` | 8 bytes | Pointer (VM address or host pointer) | `q` |
| `struct(N)` | N bytes | Struct passed by value (1-128 bytes) | `q` (address) |
| `void` | — | No value (return type only) | — |

### Examples

```nyx
.extern puts(ptr): i32
.extern InitWindow(i32, i32, ptr): void
.extern ClearBackground(struct(4)): void
.extern DrawCircle(i32, i32, f32, struct(4)): void
.extern GetFrameTime(): f32
.extern CloseWindow(): void
```

---

## Variadic Functions

Functions like `printf` and `TextFormat` accept a variable number of arguments.
Declare them with `...` after the fixed parameters:

```nyx
.extern printf(ptr, ...): i32
.extern TextFormat(ptr, ...): ptr
```

At the call site, specify the types of the variadic arguments in parentheses
after the function name:

```nyx
; printf("Hello %s!\n", "world")
mov q0, fmt_str
mov q1, arg_world
call printf(ptr)

; printf("%d + %d = %d\n", 10, 20, 30)
mov q0, fmt_three
mov d1, 10
mov d2, 20
mov d3, 30
call printf(i32, i32, i32)

; TextFormat("FPS: %d (target: %d)", fps, target)
mov q0, fmt_fps
mov d1, 60
mov d2, 60
call TextFormat(i32, i32)    ; returns ptr in q0
```

The fixed arguments follow the normal calling convention. The variadic
arguments continue from where the fixed arguments left off in the same
register pools.

Non-variadic calls use the normal `call name` syntax without parentheses.

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
3 integer args and 2 float args uses `q0`-`q2` and `ff0`-`ff1`.

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
| `struct(N)` | Written to VM memory at address in `q0` |

### Pointer Translation

When a `ptr` argument is passed, the VM checks if the value falls within VM
memory. If it does, the VM address is translated to a real host pointer. If
it falls outside VM memory (e.g. a pointer returned by a previous FFI call
like `TextFormat`), it is passed through as-is. This means you can freely
pass both label addresses and host pointers returned by native functions:

```nyx
mov q0, my_string    ; VM address of the string
call puts            ; puts receives a real const char*

mov q0, fmt
mov d1, 42
call TextFormat(i32) ; returns a host pointer in q0
; q0 now holds a host pointer, which can be passed directly:
mov d1, 10
mov d2, 10
mov d3, 20
mov q4, DARKGRAY
call DrawText        ; q0 (host pointer) is passed through correctly
```

### Struct Passing

When a `struct(N)` argument is passed, the register holds a VM address
pointing to the struct's bytes in memory. The VM copies those N bytes and
passes them **by value** to the native function via libffi. Layout structs
in the `.data` section (or on the stack) just like any other data:

```nyx
.extern ClearBackground(struct(4)): void

.section text
_start:
    mov q0, RAYWHITE       ; q0 = address of the Color bytes
    call ClearBackground    ; VM copies 4 bytes and passes by value
    hlt

.section data
RAYWHITE:   db 245, 245, 245, 255   ; Color { r, g, b, a }
```

For struct **return values**, the VM writes the returned bytes to the VM
memory address held in `q0` at the time of the call.

---

## Float Data

The `dd` directive accepts float literals, storing them as 4-byte IEEE 754
values. Similarly, `dq` accepts float literals as 8-byte IEEE 754 doubles:

```nyx
.section data
speed:      dd 120.0        ; f32
pi:         dq 3.14159265   ; f64
position:   dd 0.0          ; f32, initialized to zero
```

This is useful for storing mutable float state (e.g. positions, velocities)
that you load and store with `mov ff0, [addr]` / `mov [addr], ff0`.

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

Calling raylib functions directly with `struct(4)` for Color, `dd` for float
data, and `TextFormat` for dynamic text:

```nyx
.extern InitWindow(i32, i32, ptr): void
.extern CloseWindow(): void
.extern SetTargetFPS(i32): void
.extern WindowShouldClose(): i32
.extern BeginDrawing(): void
.extern EndDrawing(): void
.extern ClearBackground(struct(4)): void
.extern DrawCircle(i32, i32, f32, struct(4)): void
.extern DrawText(ptr, i32, i32, i32, struct(4)): void
.extern GetFrameTime(): f32
.extern GetFPS(): i32
.extern TextFormat(ptr, ...): ptr

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
    call GetFrameTime           ; ff0 = delta time
    mov q8, posX
    mov ff1, [q8]
    mul ff2, ff0, 120.0         ; speed = 120 px/sec
    add ff1, ff1, ff2
    mov [q8], ff1

    cmp ff1, 850.0
    jlt .no_wrap
    mov ff1, 0.0
    mov [q8], ff1
.no_wrap:

    call BeginDrawing

        mov q0, RAYWHITE
        call ClearBackground

        ; DrawCircle((int)posX, 225, 30.0, RED)
        mov q8, posX
        mov ff4, [q8]
        mov d0, ff4             ; float-to-int truncation
        mov d1, 225
        mov ff0, 30.0
        mov q2, RED
        call DrawCircle

        ; Dynamic FPS text using TextFormat (variadic)
        call GetFPS
        mov d1, d0
        mov q0, fmt_fps
        call TextFormat(i32)
        mov d1, 10
        mov d2, 10
        mov d3, 20
        mov q4, DARKGRAY
        call DrawText

    call EndDrawing

    call WindowShouldClose
    cmp d0, 1
    jne .loop

    call CloseWindow
    hlt

.section data
title:      .asciz "raylib example - nyx FFI"
fmt_fps:    .asciz "FPS: %d"

posX:       dd 0.0

RAYWHITE:   db 245, 245, 245, 255
RED:        db 230, 41, 55, 255
DARKGRAY:   db 80, 80, 80, 255
```

Run it:

```sh
nyx run raylib_demo.nyx -l /opt/homebrew/lib/libraylib.dylib   # macOS (Homebrew)
nyx run raylib_demo.nyx -l /usr/local/lib/libraylib.so          # Linux
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

- **Maximum 64 arguments** per extern call.
- **Struct size limit** is 128 bytes per `struct(N)` type.
