# Assembly Language Syntax

This document describes the source-level syntax of the Nyx assembly language —
the `.nyx` files consumed by the assembler.

---

## Program Structure

A Nyx program is organized into **sections**. Two section types are supported:

| Section          | Purpose                                      |
|------------------|----------------------------------------------|
| `.section text`  | Executable code (instructions)               |
| `.section data`  | Data declarations (strings, constants, buffers) |

Sections are introduced with the `.section` directive and remain active until
the next `.section` directive or end-of-file.

```/dev/null/example.nyx#L1-7
.section text
_start:
    mov q0, 1
    hlt

.section data
message: db "Hello", 0x00
```

---

## Entry Point

By default the label **`_start`** is used as the program entry point. This can
be overridden with the `.entry` directive:

```/dev/null/example.nyx#L1-2
.entry main       ; use the label "main" as entry point
.entry 0x0100     ; or specify an absolute address
```

The entry point address is written as the first 8 bytes (little-endian `u64`)
of the compiled bytecode file.

---

## Comments

Line comments begin with a semicolon. Everything after `;` on a line is
ignored by the assembler.

```/dev/null/example.nyx#L1-3
mov q0, 1       ; load stdout file descriptor
; this entire line is a comment
```

---

## Labels

A label is an identifier followed by a colon. Labels mark positions in the
bytecode and can be used as jump/call targets or data references.

```/dev/null/example.nyx#L1-5
loop_start:
    inc q0
    cmp q0, q1
    jlt loop_start   ; jump back to "loop_start"
```

When a label appears as an operand it resolves to its **absolute address** in
the bytecode at compile time.

```/dev/null/example.nyx#L1-2
mov q1, message   ; q1 ← address of "message" in the data section
```

---

## Number Literals

The assembler accepts integers in four bases and floating-point literals:

| Form          | Prefix | Examples                   |
|---------------|--------|----------------------------|
| Decimal       | —      | `42`, `-7`, `0`            |
| Hexadecimal   | `0x`   | `0xFF`, `0x1A2B`           |
| Binary        | `0b`   | `0b1010`, `0b11110000`     |
| Octal         | `0o`   | `0o755`, `0o644`           |
| Floating point| —      | `3.14`, `2.0`, `-1.5`      |

---

## String Literals

Strings are enclosed in double quotes and support standard C-like escape
sequences:

```/dev/null/example.nyx#L1
db "Hello, world!\n", 0x00
```

Recognized escape sequences include `\n` (newline), `\t` (tab), `\0` (null),
`\\` (backslash), `\"` (double quote), and others.

---

## Registers

Nyx exposes three classes of registers. Register names are **case-insensitive**.

### General-Purpose Registers

Sixteen physical slots, each accessible through four width views:

| Prefix | View   | Width   | Registers     |
|--------|--------|---------|---------------|
| `b`    | byte   | 8-bit   | `b0`–`b15`   |
| `w`    | word   | 16-bit  | `w0`–`w15`   |
| `d`    | dword  | 32-bit  | `d0`–`d15`   |
| `q`    | qword  | 64-bit  | `q0`–`q15`   |

### Floating-Point Registers

| Prefix | View   | Width              | Registers      |
|--------|--------|--------------------|----------------|
| `ff`   | float  | 32-bit IEEE 754    | `ff0`–`ff15`   |
| `dd`   | double | 64-bit IEEE 754    | `dd0`–`dd15`   |

### Special Registers

| Name | Purpose                |
|------|------------------------|
| `ip` | Instruction pointer    |
| `sp` | Stack pointer          |
| `bp` | Base pointer           |

Special registers are always qword (64-bit) sized.

---

## Data Size Keywords

When the operand size is ambiguous (e.g. memory operations without a register
to infer from), a size keyword can be used as a prefix:

| Keyword  | Size    |
|----------|---------|
| `byte`   | 1 byte  |
| `word`   | 2 bytes |
| `dword`  | 4 bytes |
| `qword`  | 8 bytes |
| `float`  | 4 bytes |
| `double`  | 8 bytes |

```/dev/null/example.nyx#L1-2
mov byte [q0], 0xFF
mov qword [q1 + 8], 0
```

---

## Memory Addressing

Memory operands are enclosed in square brackets. Several addressing forms are
supported:

| Form               | Description                          | Example              |
|--------------------|--------------------------------------|----------------------|
| `[reg]`            | Address from register value          | `[q0]`               |
| `[reg + offset]`   | Register plus immediate offset       | `[q0 + 8]`          |
| `[imm]`            | Immediate (absolute) address         | `[0x1000]`           |
| `[label]`          | Address of a label                   | `[message]`          |
| `[reg + label]`    | Register plus label address          | `[q0 + message]`     |

```/dev/null/example.nyx#L1-4
mov q0, [q1]           ; load from address in q1
mov q0, [q1 + 16]      ; load from q1 + 16
mov q0, [0x2000]       ; load from absolute address
mov q0, [buffer]       ; load from label address
```

---

## Instruction Format

Instructions follow the general forms below. The number and types of operands
depend on the specific instruction.

```/dev/null/example.nyx#L1-5
instruction
instruction operand
instruction operand1, operand2
instruction operand1, operand2, operand3
instruction size operand1, operand2
```

Operands are separated by commas. A size keyword may appear between the
mnemonic and the first operand when a size prefix is needed.

```/dev/null/example.nyx#L1-4
nop
hlt
mov q0, 42
add q0, q1, q2
```

---

## Expressions

Operands support compile-time constant expressions built from:

- **Binary operators:** `+`, `-`, `*`, `/`, `|`, `&`, `^`
- **Unary operator:** `-` (negation)
- **Parentheses** for grouping

Expressions are fully evaluated at compile time when used as immediates.

```/dev/null/example.nyx#L1-2
mov q0, (O_WRONLY | O_CREAT | O_TRUNC)
mov q1, 1024 * 4
```

---

## Data Declarations

Data declarations appear in the `.section data` section. Each declaration
begins with a label (optional) followed by a data directive.

### Define Directives

Define directives emit initialized data:

| Directive | Unit Size | Description       |
|-----------|-----------|-------------------|
| `db`      | 1 byte    | Define bytes      |
| `dw`      | 2 bytes   | Define words      |
| `dd`      | 4 bytes   | Define dwords     |
| `dq`      | 8 bytes   | Define qwords     |

Values are comma-separated. `db` accepts both integers and string literals.

```/dev/null/example.nyx#L1-4
message:  db "Hello, world!\n", 0x00
flags:    db 0x01, 0x02, 0x03
port:     dw 8080
magic:    dq 0xDEADBEEFCAFEBABE
```

### String Directives

| Directive | Description                              |
|-----------|------------------------------------------|
| `.ascii`  | Embed a raw string (no null terminator)  |
| `.asciz`  | Embed a null-terminated string           |

```/dev/null/example.nyx#L1-2
greeting: .asciz "Hello!"
raw:      .ascii "raw bytes"
```

### Reserve Directives

Reserve directives allocate zero-initialized space:

| Directive | Unit Size | Description          |
|-----------|-----------|----------------------|
| `resb`    | 1 byte    | Reserve N bytes      |
| `resw`    | 2 bytes   | Reserve N words      |
| `resd`    | 4 bytes   | Reserve N dwords     |
| `resq`    | 8 bytes   | Reserve N qwords     |

```/dev/null/example.nyx#L1-2
buffer: resb 256        ; 256 zero bytes
table:  resq 16         ; 16 zero qwords (128 bytes)
```

---

## Directives

| Directive          | Description                                          |
|--------------------|------------------------------------------------------|
| `.section text`    | Switch to the text (code) section                    |
| `.section data`    | Switch to the data section                           |
| `.entry name`      | Set the program entry point to a label or address    |
| `.extern name(types): ret` | Declare an external function with its FFI type signature |

```/dev/null/example.nyx#L1-10
.extern puts(ptr): i32

.section text
_start:
    mov q0, message
    call puts
    hlt

.section data
message:
    .asciz "Hello, world!"
```

---

## Complete Example

```/dev/null/hello.nyx#L1-14
.section text
_start:
    mov q0, 1           ; stdout
    mov q1, message     ; buffer address
    mov q2, 14          ; length
    mov q15, 3          ; sys_write
    syscall
    hlt

.section data
message:
    db "Hello, world!\n", 0x00
```

This program writes "Hello, world!" to standard output using the `sys_write`
syscall and then halts.
