# Instruction Set Reference

This document is the complete instruction reference for the Nyx virtual machine.

## Registers

Each of the 16 general-purpose register slots (0–15) can be accessed in six widths:

| Prefix | Type   | Size    | Example |
|--------|--------|---------|---------|
| `b`    | byte   | 8-bit   | `b0`    |
| `w`    | word   | 16-bit  | `w0`    |
| `d`    | dword  | 32-bit  | `d0`    |
| `q`    | qword  | 64-bit  | `q0`    |
| `ff`   | float  | 32-bit  | `ff0`   |
| `dd`   | double | 64-bit  | `dd0`   |

Three special-purpose registers also exist:

| Register | Purpose            |
|----------|--------------------|
| `ip`     | Instruction pointer |
| `sp`     | Stack pointer       |
| `bp`     | Base pointer        |

## Flags

The VM maintains two condition flags, set exclusively by the `cmp` instruction:

| Flag | Meaning                                    |
|------|--------------------------------------------|
| `eq` | Set when operands are equal                |
| `lt` | Set when the first operand is less than the second |

## Summary Table

| Mnemonic  | Operands              | Description                        | Group            |
|-----------|-----------------------|------------------------------------|------------------|
| `nop`     | —                     | No operation                       | Data Movement    |
| `mov`     | dest, src             | Move / load / store data           | Data Movement    |
| `push`    | src                   | Push value onto the stack          | Stack            |
| `pop`     | dest                  | Pop value from the stack           | Stack            |
| `add`     | dest, src1, src2      | Addition                           | Arithmetic       |
| `sub`     | dest, src1, src2      | Subtraction                        | Arithmetic       |
| `mul`     | dest, src1, src2      | Multiplication                     | Arithmetic       |
| `div`     | dest, src1, src2      | Division                           | Arithmetic       |
| `inc`     | reg                   | Increment by 1                     | Unary            |
| `dec`     | reg                   | Decrement by 1                     | Unary            |
| `neg`     | reg                   | Negate value                       | Unary            |
| `and`     | dest, src1, src2      | Bitwise AND                        | Bitwise          |
| `or`      | dest, src1, src2      | Bitwise OR                         | Bitwise          |
| `xor`     | dest, src1, src2      | Bitwise XOR                        | Bitwise          |
| `shl`     | dest, src1, src2      | Shift left                         | Bitwise          |
| `shr`     | dest, src1, src2      | Shift right                        | Bitwise          |
| `rol`     | dest, src1, src2      | Rotate left                        | Bitwise          |
| `ror`     | dest, src1, src2      | Rotate right                       | Bitwise          |
| `cmp`     | reg, reg/imm          | Compare and set flags              | Comparison       |
| `jmp`     | target                | Unconditional jump                 | Control Flow     |
| `jeq`     | target                | Jump if equal                      | Control Flow     |
| `jne`     | target                | Jump if not equal                  | Control Flow     |
| `jlt`     | target                | Jump if less than                  | Control Flow     |
| `jgt`     | target                | Jump if greater than               | Control Flow     |
| `jle`     | target                | Jump if less or equal              | Control Flow     |
| `jge`     | target                | Jump if greater or equal           | Control Flow     |
| `call`    | target                | Call subroutine                    | Subroutines      |
| `call`    | external_name         | Call external (FFI) function       | Subroutines      |
| `ret`     | —                     | Return from subroutine             | Subroutines      |
| `syscall` | —                     | Execute system call                | System           |
| `hlt`     | —                     | Halt the virtual machine           | System           |

---

## Data Movement

### `nop`

No operation. Does nothing; advances the instruction pointer.

```/dev/null/example.nyx#L1
nop
```

### `mov`

Move data between registers, memory, and immediates.

#### Register ← Register

Copy the value of one register into another.

```/dev/null/example.nyx#L1
mov q0, q1
```

#### Register ← Immediate

Load an immediate (constant) value into a register. The data size is inferred from the destination register prefix (`b` → byte, `w` → word, `d` → dword, `q` → qword, `ff` → float, `dd` → double).

```/dev/null/example.nyx#L1-3
mov b0, 0xFF
mov q1, 42
mov ff0, 3.14
```

#### Register ← Memory

Load a value from a memory address into a register.

```/dev/null/example.nyx#L1-4
mov q0, [q1]           ; base register
mov q0, [q1 + 8]       ; base register + offset
mov q0, [0x1000]       ; absolute address
mov q0, [0x1000 + 16]  ; absolute address + offset
```

#### Memory ← Register

Store a register value to a memory address.

```/dev/null/example.nyx#L1-2
mov [q1], q0
mov [q1 + 8], q0
```

#### Memory ← Immediate

Store an immediate value to a memory address. A **data size prefix** is required because the assembler cannot infer the width from the immediate alone.

```/dev/null/example.nyx#L1-2
mov dword [q0], 42
mov byte [0x2000], 0x0A
```

Valid size prefixes: `byte`, `word`, `dword`, `qword`, `float`, `double`.

#### Memory ← Memory

Copy a value from one memory address to another. A **data size prefix** is required.

```/dev/null/example.nyx#L1
mov qword [q0], [q1]
```

#### Addressing Modes

All memory operands use one of two addressing variants:

| Variant | Base    | Syntax                       |
|---------|---------|------------------------------|
| 0x00    | Register | `[reg]` or `[reg + offset]` |
| 0x01    | Immediate | `[imm]` or `[imm + offset]` |

Both variants include a 64-bit offset (defaults to 0 when omitted).

---

## Stack Operations

The Nyx stack grows **downward** from high addresses toward low addresses.

### `push`

Push a value onto the stack. The stack pointer (`sp`) is decremented by the size of the pushed value, and the value is written at the new `sp`.

#### Push Immediate

```/dev/null/example.nyx#L1-2
push 42           ; size inferred when unambiguous
push qword 42     ; explicit size prefix
```

An optional data size prefix (`byte`, `word`, `dword`, `qword`, `float`, `double`) can be supplied when the size is ambiguous.

#### Push Register

```/dev/null/example.nyx#L1-2
push q0           ; size inferred from register prefix (qword)
push dword d3     ; explicit size override
```

#### Push Memory

```/dev/null/example.nyx#L1
push [q0]
```

### `pop`

Pop a value from the top of the stack. The value at `sp` is read, then `sp` is incremented.

#### Pop into Register

```/dev/null/example.nyx#L1
pop q0
```

#### Pop into Memory

```/dev/null/example.nyx#L1
pop [q0]
```

---

## Arithmetic

All arithmetic instructions follow the three-operand form:

```/dev/null/example.nyx#L1
<op> dest, src1, src2
```

**Operand rules:**

- `dest` — must be a register.
- `src1` — must be a register.
- `src2` — register **or** immediate value.
- The data size is determined by the destination register.
- Works with all data types: byte, word, dword, qword, float, and double.

### `add`

Addition. `dest = src1 + src2`.

```/dev/null/example.nyx#L1-2
add q0, q1, q2       ; q0 = q1 + q2
add ff0, ff1, 1.5    ; ff0 = ff1 + 1.5
```

### `sub`

Subtraction. `dest = src1 - src2`.

```/dev/null/example.nyx#L1
sub q0, q1, 10       ; q0 = q1 - 10
```

### `mul`

Multiplication. `dest = src1 * src2`.

```/dev/null/example.nyx#L1
mul d0, d1, d2       ; d0 = d1 * d2
```

### `div`

Division. `dest = src1 / src2`. For integer types this is **truncating** division. For float and double types it is IEEE 754 division.

```/dev/null/example.nyx#L1-2
div q0, q1, 4        ; integer truncating division
div dd0, dd1, dd2    ; double-precision division
```

---

## Bitwise Operations

Bitwise instructions share the same three-operand form as arithmetic:

```/dev/null/example.nyx#L1
<op> dest, src1, src2
```

**Operand rules** are identical to arithmetic, with one restriction: bitwise operations only work on **integer types** (`byte`, `word`, `dword`, `qword`). They are **not** valid for `float` or `double` registers.

### `and`

Bitwise AND. `dest = src1 & src2`.

```/dev/null/example.nyx#L1
and q0, q1, 0xFF
```

### `or`

Bitwise OR. `dest = src1 | src2`.

```/dev/null/example.nyx#L1
or d0, d1, d2
```

### `xor`

Bitwise XOR. `dest = src1 ^ src2`.

```/dev/null/example.nyx#L1
xor q0, q0, q0       ; zero a register
```

### `shl`

Shift left. `dest = src1 << src2`.

```/dev/null/example.nyx#L1
shl q0, q1, 4        ; shift left by 4 bits
```

### `shr`

Shift right. `dest = src1 >> src2`.

```/dev/null/example.nyx#L1
shr q0, q1, 8
```

### `rol`

Rotate left. Bits shifted out of the high end wrap around to the low end.

```/dev/null/example.nyx#L1
rol d0, d1, 3
```

### `ror`

Rotate right. Bits shifted out of the low end wrap around to the high end.

```/dev/null/example.nyx#L1
ror d0, d1, 3
```

---

## Unary Operations

Unary instructions operate on a single register in place.

### `inc`

Increment the register value by 1. Works for both integer and floating-point types.

```/dev/null/example.nyx#L1-2
inc q0
inc ff0
```

### `dec`

Decrement the register value by 1. Works for both integer and floating-point types.

```/dev/null/example.nyx#L1
dec q0
```

### `neg`

Negate the register value. For integer types this is two's complement negation. For float/double types the sign bit is flipped.

```/dev/null/example.nyx#L1-2
neg q0        ; q0 = -q0 (two's complement)
neg dd0       ; dd0 = -dd0 (sign flip)
```

---

## Comparison

### `cmp`

Compare two values and set the `eq` and `lt` flags accordingly.

#### Register vs. Immediate

```/dev/null/example.nyx#L1
cmp q0, 100
```

#### Register vs. Register

```/dev/null/example.nyx#L1
cmp q0, q1
```

**Flag results:**

| Condition        | `eq`  | `lt`  |
|------------------|-------|-------|
| src1 == src2     | true  | false |
| src1 < src2      | false | true  |
| src1 > src2      | false | false |

---

## Control Flow

All jump instructions accept either an **immediate** (label) or a **register** as the target address.

### `jmp`

Unconditional jump.

```/dev/null/example.nyx#L1-2
jmp my_label
jmp q0           ; indirect jump
```

### `jeq`

Jump if equal — branches when `eq` is set.

```/dev/null/example.nyx#L1-2
cmp q0, 0
jeq is_zero
```

### `jne`

Jump if not equal — branches when `eq` is **not** set.

```/dev/null/example.nyx#L1-2
cmp q0, q1
jne not_equal
```

### `jlt`

Jump if less than — branches when `lt` is set.

```/dev/null/example.nyx#L1-2
cmp q0, 10
jlt below_ten
```

### `jgt`

Jump if greater than — branches when `lt` is **not** set **and** `eq` is **not** set.

```/dev/null/example.nyx#L1-2
cmp q0, 10
jgt above_ten
```

### `jle`

Jump if less or equal — branches when `lt` is set **or** `eq` is set.

```/dev/null/example.nyx#L1-2
cmp q0, 10
jle at_most_ten
```

### `jge`

Jump if greater or equal — branches when `lt` is **not** set **or** `eq` is set.

```/dev/null/example.nyx#L1-2
cmp q0, 10
jge at_least_ten
```

### Conditional Jump Summary

| Mnemonic | Condition                        | Meaning            |
|----------|----------------------------------|--------------------|
| `jeq`    | `eq == true`                     | Equal              |
| `jne`    | `eq == false`                    | Not equal          |
| `jlt`    | `lt == true`                     | Less than          |
| `jgt`    | `lt == false` **and** `eq == false` | Greater than    |
| `jle`    | `lt == true` **or** `eq == true`   | Less or equal    |
| `jge`    | `lt == false` **or** `eq == true`  | Greater or equal |

---

## Subroutines

### `call`

Push the current instruction pointer onto the stack as a return address, then jump to the target.

#### Call by Label / Immediate

```/dev/null/example.nyx#L1
call my_function
```

#### Call by Register (Indirect)

```/dev/null/example.nyx#L1
call q0
```

#### Call External Function (FFI)

Call a function from a dynamically loaded shared library. The function name is encoded as a null-terminated string in the bytecode.

```/dev/null/example.nyx#L1
call puts
```

External libraries are loaded via the `-l` CLI flag at execution time.

### `ret`

Pop the return address from the stack and jump to it, returning control to the caller.

```/dev/null/example.nyx#L1-3
my_function:
    ; ... function body ...
    ret
```

---

## System

### `syscall`

Execute a system call. The syscall number is read from register `q15`. Arguments are passed in registers according to the syscall calling convention (see the syscalls documentation for the full table).

```/dev/null/example.nyx#L1-2
mov q15, 1          ; syscall number for write
syscall
```

### `hlt`

Halt the virtual machine. Execution stops immediately.

```/dev/null/example.nyx#L1
hlt
```

Every program should end with `hlt` to cleanly terminate execution.
