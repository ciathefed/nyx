# Memory Model

The Nyx VM uses a Memory Management Unit (MMU) with a block-based memory model.
Memory is organized as a flat, linear address space composed of multiple
contiguous blocks.

---

## Bus Interface

Every memory region implements a vtable-based **Bus** interface:

| Method       | Description                                       |
|--------------|---------------------------------------------------|
| `name`       | Human-readable name of the block.                 |
| `size`       | Total size of the block in bytes.                 |
| `read`       | Read a single value at an offset.                 |
| `readSlice`  | Read a contiguous byte slice at an offset.        |
| `write`      | Write a single value at an offset.                |
| `writeSlice` | Write a contiguous byte slice at an offset.       |

The primary implementation is **Block** — a simple contiguous byte array.

---

## Address Translation

Addresses are unified across all blocks. When the MMU services a read or write,
it iterates through its block list to find the block that contains the target
address, then translates the global address to a local offset within that block.

```/dev/null/layout.txt#L1-5
Global address:   0x0000 ─────────────────────────────── mem_size-1
                  ├── Block 0 ──┤├── Block 1 ──┤├── ... ┤

Local offset:     Block 0 starts at global 0
                  Block 1 starts at global (Block 0 size), etc.
```

---

## Initial Memory Layout

When the VM starts, two blocks are created:

1. **Program Block** — Contains the loaded bytecode (text + data sections).
   Size equals the program data length (everything after the 8-byte entry point
   header in the `.nyb` file).

2. **Memory Block** — General-purpose memory for the stack and runtime data.
   Size equals `mem_size - program_data.len`.

The default total memory size is **65536 bytes**, configurable with the `-m`
flag.

```/dev/null/layout.txt#L1-4
Address space:

[0x0000 ... program_end-1]   → Program Block (bytecode)
[program_end ... mem_size-1]  → Memory Block (general memory + stack)
```

---

## Dynamic Allocation

The `sys_malloc` syscall adds new blocks to the MMU at runtime. These blocks are
appended after the existing blocks, extending the address space beyond the
initial `mem_size` boundary.

The `sys_free` syscall removes a dynamically allocated block by matching its
start address. Only blocks created by `sys_malloc` can be freed.

---

## Byte Order

All multi-byte values are stored in **little-endian** format.

---

## Data Sizes

| Name   | Size    | Description                        |
|--------|---------|------------------------------------|
| byte   | 1 byte  | 8-bit unsigned integer             |
| word   | 2 bytes | 16-bit unsigned integer            |
| dword  | 4 bytes | 32-bit unsigned integer            |
| qword  | 8 bytes | 64-bit unsigned integer            |
| float  | 4 bytes | 32-bit IEEE 754 floating point     |
| double | 8 bytes | 64-bit IEEE 754 floating point     |

---

## Stack

The stack lives at the top of the Memory Block and grows **downward**.

- **`sp`** (Stack Pointer) is initialized to the total memory size.
- **`push`** decrements `sp` by the data size, then writes the value at the
  new `sp`.
- **`pop`** reads the value at `sp`, then increments `sp` by the data size.

```/dev/null/stack.txt#L1-9
Memory Block (high addresses at top):

  mem_size  ┐ ← sp (initial)
            │  ↑ push decrements sp
            │  │
            │  stack grows downward
            │
            │  ... free space ...
  program_end ┘
```

### Overflow and Underflow

- **Stack overflow** — occurs if `sp` would go below 0.
- **Stack underflow** — occurs if `sp + size` would exceed the total memory
  size.

---

## Addressing Modes

Memory addresses in instructions use bracket syntax:

| Syntax           | Meaning                             |
|------------------|-------------------------------------|
| `[reg]`          | Register as base, offset = 0        |
| `[reg + offset]` | Register base + immediate offset    |
| `[imm]`          | Immediate address, offset = 0       |
| `[imm + offset]` | Immediate base + immediate offset   |

### Bytecode Encoding

In the bytecode, an address operand is encoded as:

1. **Variant byte** — `0x00` for register base, `0x01` for immediate base.
2. **Base value:**
   - If register base: 1 byte for the register index.
   - If immediate base: 8 bytes for the base address (`u64`, little-endian).
3. **Offset** — 8 bytes, signed (`i64`, little-endian), allowing negative
   offsets.

```/dev/null/encoding.txt#L1-7
Register-based:   [variant=0x00] [reg: 1 byte] [offset: i64 LE]
                  Total: 10 bytes

Immediate-based:  [variant=0x01] [base: u64 LE] [offset: i64 LE]
                  Total: 17 bytes
```

---

## Data Declaration Directives

These directives are used in `.section data` (or `.section text`) to embed
static data into the program.

### Value Declarations

| Directive          | Description                                    |
|--------------------|------------------------------------------------|
| `db val1, val2, ...` | Declare bytes (1 byte each). Strings are expanded to individual bytes. |
| `dw val1, val2, ...` | Declare words (2 bytes each, little-endian).   |
| `dd val1, val2, ...` | Declare dwords (4 bytes each, little-endian).  |
| `dq val1, val2, ...` | Declare qwords (8 bytes each, little-endian).  |

### Reservations

| Directive | Description                                     |
|-----------|-------------------------------------------------|
| `resb N`  | Reserve N bytes (zero-initialized).              |
| `resw N`  | Reserve N words (2×N bytes, zero-initialized).   |
| `resd N`  | Reserve N dwords (4×N bytes, zero-initialized).  |
| `resq N`  | Reserve N qwords (8×N bytes, zero-initialized).  |

### String Directives

| Directive         | Description                              |
|-------------------|------------------------------------------|
| `.ascii "string"` | Embed string bytes without a null terminator. |
| `.asciz "string"` | Embed string bytes with a null terminator.    |
