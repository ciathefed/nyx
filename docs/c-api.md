# C API & External Functions

Nyx can load native shared libraries (`.so`, `.dylib`, `.dll`) at runtime,
allowing external C/C++ code to interact with the VM. This is the mechanism
for writing native extensions — anything from custom I/O routines to full
graphical applications.

The public interface is defined in a single header: **`include/nyx.h`**.

---

## Overview

1. Write one or more C functions that match the external function signature.
2. Compile them into a shared library.
3. Declare the functions in your Nyx source with `.extern`.
4. Load the library at runtime with the `-l` CLI flag.

The VM resolves each `.extern` name against the symbols exported by the
loaded library. When a `call` to an external function is reached, the VM
invokes the native code, passing a pointer to itself so the function can
read and write registers and memory.

---

## CLI Usage

External libraries are passed with `-l` on the `exec` or `run` subcommands:

```/dev/null/cli.txt#L1-2
nyx exec program.nyb -l ./mylib.so
nyx run  program.nyx -l ./mylib.so
```

Multiple libraries can be loaded by repeating the flag:

```/dev/null/cli.txt#L1
nyx run program.nyx -l ./libfoo.so -l ./libbar.so
```

---

## Nyx Source Side

### Declaring External Functions

Use the `.extern` directive at the top of the source file to declare each
function that will be resolved from a shared library:

```/dev/null/example.nyx#L1-2
.extern my_print
.extern my_read
```

### Calling External Functions

Call an external function by name with the regular `call` instruction:

```/dev/null/example.nyx#L1
call my_print
```

Under the hood, the compiler emits the `call_ex` opcode, which encodes the
function name as a null-terminated string directly in the bytecode. At
runtime the VM looks up the symbol in the loaded libraries and jumps to it.

---

## External Function Signature

Every function exported from a shared library **must** have the following C
signature:

```/dev/null/sig.c#L1
int32_t function_name(Vm *vm);
```

| Item | Description |
|------|-------------|
| `Vm *vm` | Opaque pointer to the running VM instance. Pass it to every C API call. |
| Return value | An `int32_t`. Return `0` for success. |

The `Vm` type is forward-declared in `nyx.h` as an opaque struct — you never
access its fields directly.

---

## C API Reference

All functions below are declared in **`include/nyx.h`**.

### Register Access

| Function | Description |
|----------|-------------|
| `int64_t vm_get_reg_int(Vm *vm, uint8_t index)` | Read a register as a signed 64-bit integer. |
| `double vm_get_reg_float(Vm *vm, uint8_t index)` | Read a register as a 64-bit float (double). |
| `void vm_set_reg_int(Vm *vm, uint8_t index, int64_t value)` | Write a signed 64-bit integer to a register. |
| `void vm_set_reg_float(Vm *vm, uint8_t index, double value)` | Write a 64-bit float (double) to a register. |

The `index` parameter is one of the `Register` enum constants (see below).

### Memory Read

| Function | Description |
|----------|-------------|
| `uint8_t vm_mem_read_byte(Vm *vm, size_t addr)` | Read 1 byte from VM memory. |
| `uint16_t vm_mem_read_word(Vm *vm, size_t addr)` | Read 2 bytes (16-bit). |
| `uint32_t vm_mem_read_dword(Vm *vm, size_t addr)` | Read 4 bytes (32-bit). |
| `uint64_t vm_mem_read_qword(Vm *vm, size_t addr)` | Read 8 bytes (64-bit). |
| `float vm_mem_read_float(Vm *vm, size_t addr)` | Read a 32-bit IEEE 754 float. |
| `double vm_mem_read_double(Vm *vm, size_t addr)` | Read a 64-bit IEEE 754 double. |
| `const char *vm_mem_read_cstr(Vm *vm, size_t addr)` | Read a null-terminated C string starting at `addr`. |

All addresses refer to the VM's unified address space (program + data +
dynamic blocks).

---

## Register Enum

The `Register` enum in C mirrors the Zig enum used internally by the VM.
Each constant maps to the single-byte encoding used in bytecode.

```/dev/null/enum.c#L1-22
typedef enum Register {
    REG_B0,  REG_W0,  REG_D0,  REG_Q0,  REG_FF0,  REG_DD0,   /*  0– 5  */
    REG_B1,  REG_W1,  REG_D1,  REG_Q1,  REG_FF1,  REG_DD1,   /*  6–11  */
    REG_B2,  REG_W2,  REG_D2,  REG_Q2,  REG_FF2,  REG_DD2,   /* 12–17  */
    REG_B3,  REG_W3,  REG_D3,  REG_Q3,  REG_FF3,  REG_DD3,   /* 18–23  */
    REG_B4,  REG_W4,  REG_D4,  REG_Q4,  REG_FF4,  REG_DD4,   /* 24–29  */
    REG_B5,  REG_W5,  REG_D5,  REG_Q5,  REG_FF5,  REG_DD5,   /* 30–35  */
    REG_B6,  REG_W6,  REG_D6,  REG_Q6,  REG_FF6,  REG_DD6,   /* 36–41  */
    REG_B7,  REG_W7,  REG_D7,  REG_Q7,  REG_FF7,  REG_DD7,   /* 42–47  */
    REG_B8,  REG_W8,  REG_D8,  REG_Q8,  REG_FF8,  REG_DD8,   /* 48–53  */
    REG_B9,  REG_W9,  REG_D9,  REG_Q9,  REG_FF9,  REG_DD9,   /* 54–59  */
    REG_B10, REG_W10, REG_D10, REG_Q10, REG_FF10, REG_DD10,   /* 60–65  */
    REG_B11, REG_W11, REG_D11, REG_Q11, REG_FF11, REG_DD11,   /* 66–71  */
    REG_B12, REG_W12, REG_D12, REG_Q12, REG_FF12, REG_DD12,   /* 72–77  */
    REG_B13, REG_W13, REG_D13, REG_Q13, REG_FF13, REG_DD13,   /* 78–83  */
    REG_B14, REG_W14, REG_D14, REG_Q14, REG_FF14, REG_DD14,   /* 84–89  */
    REG_B15, REG_W15, REG_D15, REG_Q15, REG_FF15, REG_DD15,   /* 90–95  */
    REG_IP,  /* 96 */
    REG_SP,  /* 97 */
    REG_BP,  /* 98 */
} Register;
```

Each group of six corresponds to one GPR slot — four integer views (`b`, `w`,
`d`, `q`) followed by two floating-point views (`ff`, `dd`). The three
special registers come last. See [Registers](registers.md) for full details
on view semantics.

---

## Building a Shared Library

Compile your C source into a position-independent shared object and point the
include path at the Nyx `include/` directory:

```/dev/null/build.sh#L1-5
# Linux
gcc -shared -fPIC -o mylib.so mylib.c -I/path/to/nyx/include

# macOS
gcc -shared -fPIC -o mylib.dylib mylib.c -I/path/to/nyx/include
```

If your extension also links against the Nyx runtime library (e.g. for more
advanced use), add `-L` and `-l` flags as needed. The raylib example in
`_examples/raylib/` demonstrates this pattern with a Makefile.

---

## Example — Hello from C

### mylib.c

```/dev/null/mylib.c#L1-10
#include "nyx.h"
#include <stdio.h>

int32_t my_print(Vm *vm) {
    size_t addr = (size_t)vm_get_reg_int(vm, REG_Q0);
    const char *str = vm_mem_read_cstr(vm, addr);
    printf("From C: %s\n", str);
    return 0;
}
```

### program.nyx

```/dev/null/program.nyx#L1-11
.extern my_print

.section text
_start:
    mov q0, message
    call my_print
    hlt

.section data
message:
    .asciz "Hello from Nyx!"
```

### Build & Run

```/dev/null/shell.txt#L1-2
gcc -shared -fPIC -o mylib.so mylib.c -I/path/to/nyx/include
nyx run program.nyx -l ./mylib.so
```

Expected output:

```/dev/null/output.txt#L1
From C: Hello from Nyx!
```

---

## Real-World Example — Raylib Bridge

The repository includes a complete graphical example at `_examples/raylib/`
that wraps several raylib functions as external Nyx functions.

### Bridge Excerpt (`_examples/raylib/main.c`)

```/dev/null/raylib_bridge.c#L1-23
#include "raylib.h"
#include "nyx.h"

Color read_color(Vm *vm, size_t addr) {
    Color color = (Color){
        .r = vm_mem_read_byte(vm, addr),
        .g = vm_mem_read_byte(vm, addr+1),
        .b = vm_mem_read_byte(vm, addr+2),
        .a = vm_mem_read_byte(vm, addr+3),
    };
    return color;
}

int init_window(Vm *vm) {
    int width  = vm_get_reg_int(vm, REG_D0);
    int height = vm_get_reg_int(vm, REG_D1);
    const char *title = vm_mem_read_cstr(vm, vm_get_reg_int(vm, REG_Q2));
    InitWindow(width, height, title);
    return 0;
}

/* ... draw_text, clear_background, etc. ... */
```

### Nyx Side (`_examples/raylib/main.nyx`)

```/dev/null/raylib_nyx.nyx#L1-17
.extern init_window
.extern close_window
.extern set_target_fps
.extern window_should_close
.extern begin_drawing
.extern end_drawing
.extern clear_background
.extern draw_text

.section text
_start:
    mov d0, 800
    mov d1, 450
    mov q2, title
    call init_window
    ; ... game loop ...
```

The Makefile compiles the bridge into a shared library and links it against
both the Nyx runtime and raylib:

```/dev/null/Makefile#L1-3
CC = cc
CFLAGS = -I../../include -shared -fPIC
LDFLAGS = -L../../zig-out/lib -Wl,-rpath,../../zig-out/lib -lnyx -lraylib
```

---

## Conventions & Tips

- **Argument passing** — There is no formal calling convention for external
  functions. By convention, arguments are placed in low-numbered registers
  (`q0`–`q4`, `d0`–`d4`, etc.) before the `call`, and return values are
  written back to `q0` or `b0`. Choose register sizes that match the data
  width you need.

- **String arguments** — Store string data in the `.data` section with
  `.asciz` (null-terminated). Pass the label address in a `q` register, then
  use `vm_mem_read_cstr` on the C side.

- **Struct arguments** — For composite data (e.g. a colour with four byte
  fields), lay out the bytes in the `.data` section and pass the address.
  Use `vm_mem_read_byte` / `vm_mem_read_word` / etc. to read individual
  fields at known offsets, as the raylib example demonstrates.

- **Returning values to Nyx** — Use `vm_set_reg_int` or `vm_set_reg_float`
  to write results into registers that the Nyx program will read after the
  call returns.

- **Thread safety** — The `Vm` pointer is only valid for the duration of
  the call. Do not store it or use it from another thread.
