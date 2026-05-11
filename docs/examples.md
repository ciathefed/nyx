# Examples

A collection of annotated example programs for Nyx, progressing from the basics
to more advanced patterns. Each example is self-contained and can be assembled
and run with:

```/dev/null/usage.txt#L1
nyx run <file>.nyx -i std/
```

The `-i std/` flag tells the compiler where to find the standard library
includes. If you have the `NYX_STDLIB_PATH` environment variable set, you can
omit it.

---

## 1. Hello World

The simplest possible Nyx program — write a string to standard output and halt.

```/dev/null/hello.nyx#L1-12
.section text
_start:
    ; Set up system call parameters for write()
    mov q0, 1           ; file descriptor (stdout)
    mov q1, message     ; pointer to message string
    mov q2, 14          ; number of bytes to write
    mov q15, 3          ; system call number (sys_write)
    syscall             ; make the system call
    hlt                 ; halt the program

.section data
message:
    db "Hello, world!\n", 0x00
```

### Concepts

- **Sections.** Every Nyx program is divided into at least two sections.
  `.section text` contains executable instructions; `.section data` contains
  static data such as strings and buffers. The assembler lays out the text
  section first, followed by the data section.

- **Labels.** `_start:` and `message:` are labels — named addresses in the
  program. `_start` is the default entry point where execution begins. When
  `message` is used as an operand (e.g. `mov q1, message`), the assembler
  substitutes its resolved address.

- **`mov` instruction.** `mov` is the universal data-movement instruction.
  Here it loads immediate values into registers:
  - `mov q0, 1` — loads the integer `1` into 64-bit register `q0`.
  - `mov q1, message` — loads the address of the `message` label into `q1`.
  - `mov q15, 3` — loads the syscall number into `q15`.

- **Syscall convention.** The `syscall` instruction reads the syscall number
  from `q15`. For `sys_write` (syscall `3`), the arguments are:
  - `q0` — file descriptor (`1` = stdout)
  - `q1` — pointer to the buffer to write
  - `q2` — number of bytes to write

- **`db` directive.** `db` (declare bytes) embeds raw byte data into the data
  section. String literals are expanded into their individual ASCII bytes.
  The trailing `0x00` is a null terminator.

- **`hlt` instruction.** Halts the virtual machine. Every program should end
  with `hlt` to cleanly terminate execution.

---

## 2. Hello World with Macros

The same program, rewritten using the preprocessor's macro and include system.

```/dev/null/macro.nyx#L1-15
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

### Concepts

- **`#include`** pulls in another source file before compilation. Here
  `"stdlib.nyx"` is the standard library, which defines constants like
  `SYS_WRITE` (`0x03`), `SYS_OPEN` (`0x00`), `STDOUT` (`0x01`), and others.
  The preprocessor searches the current directory, the source file's directory,
  any `-i` include paths, and the `NYX_STDLIB_PATH` environment variable.

- **`#macro` / `#endm`** defines a multi-line macro. Parameters are prefixed
  with `$` in the declaration and the body. When the macro is invoked, each
  `$param` is textually replaced with the corresponding argument.

- **Macro invocation.** `write STDOUT, message, 14` expands to the four `mov`
  instructions and the `syscall`, exactly as in the previous example. Macros
  make repetitive syscall patterns readable and reusable.

- **Standard library constants.** `SYS_WRITE` and `STDOUT` are `#define`
  constants from `stdlib.nyx`. Using named constants instead of raw numbers
  makes the code self-documenting and portable.

---

## 3. Arithmetic Operations

Demonstrates integer and floating-point arithmetic with the three-operand
instruction format.

```/dev/null/arithmetic.nyx#L1-16
.section text
_start:
    mov q0, 20
    mov q1, 5
    add q2, q0, q1      ; q2 = 25
    sub q3, q0, q1      ; q3 = 15
    mul q4, q0, q1      ; q4 = 100
    div q5, q0, q1      ; q5 = 4

    ; Arithmetic with immediate values
    add q14, q0, 100    ; q14 = 120

    ; Floating point
    mov ff0, 3.5
    mov ff1, 2.0
    add ff2, ff0, ff1   ; ff2 = 5.5
    mul ff3, ff0, ff1   ; ff3 = 7.0

    hlt
```

### Concepts

- **Three-operand form.** All arithmetic instructions follow the pattern
  `op dest, src1, src2`. The destination is always a register. `src1` must be
  a register; `src2` can be a register *or* an immediate value. The result is
  written to `dest` without modifying the source operands.

- **Immediate operands.** `add q14, q0, 100` uses the literal value `100` as
  the second source operand, avoiding the need for a separate `mov`.

- **Floating-point registers.** Registers with the `ff` prefix (`ff0`–`ff15`)
  are 32-bit IEEE 754 float registers. They live in a separate physical
  register file from the integer `q` registers, so `ff0` and `q0` do not
  alias each other. The `dd` prefix accesses 64-bit double-precision registers.

- **Integer division.** `div` performs truncating integer division for integer
  registers. For `ff` and `dd` registers, it performs IEEE 754 floating-point
  division.

---

## 4. Bitwise Operations

Common bitwise patterns: masking, zeroing registers, and using shifts for
fast multiply/divide by powers of two.

```/dev/null/bitwise.nyx#L1-13
.section text
_start:
    mov q0, 0xFF
    mov q1, 0x0F
    and q2, q0, q1      ; q2 = 0x0F (mask lower 4 bits)
    or q3, q0, q1       ; q3 = 0xFF
    xor q4, q0, q0      ; q4 = 0 (zero a register)

    mov q5, 1
    shl q6, q5, 3       ; q6 = 8 (multiply by 8)
    shr q7, q6, 1       ; q7 = 4 (divide by 2)

    hlt
```

### Concepts

- **Bitwise AND for masking.** `and q2, q0, q1` applies `q1` as a bitmask to
  `q0`. Since `q1` is `0x0F` (binary `00001111`), only the lower four bits of
  `q0` survive — a common technique for extracting bit fields.

- **Bitwise OR for combining.** `or q3, q0, q1` combines the set bits from
  both operands. Since `q0` already has all eight low bits set (`0xFF`), the
  result is still `0xFF`.

- **XOR trick for zeroing.** `xor q4, q0, q0` XORs a register with itself,
  which always produces zero. This is a well-known idiom in assembly
  programming for clearing a register without needing a literal `0`.

- **Shift left as multiply.** `shl q6, q5, 3` shifts `1` left by 3 bit
  positions, producing `8` (i.e. 1 × 2³). Left-shifting by *n* is equivalent
  to multiplying by 2ⁿ.

- **Shift right as divide.** `shr q7, q6, 1` shifts `8` right by 1, producing
  `4` (i.e. 8 ÷ 2). Right-shifting by *n* is equivalent to integer division
  by 2ⁿ.

- **Integer-only.** Bitwise instructions (`and`, `or`, `xor`, `shl`, `shr`,
  `rol`, `ror`) only operate on integer register types (`b`, `w`, `d`, `q`).
  They are not valid for `ff` or `dd` registers.

---

## 5. Loops with Compare and Jump

A counted loop that increments a register from 0 to 10.

```/dev/null/loop.nyx#L1-10
.section text
_start:
    mov q0, 0           ; counter
    mov q1, 10          ; limit

loop:
    inc q0              ; counter++
    cmp q0, q1          ; compare counter to limit
    jlt loop            ; jump back if counter < limit
    ; q0 is now 10
    hlt
```

### Concepts

- **Labels as jump targets.** `loop:` defines an address that control-flow
  instructions can reference. `jlt loop` jumps back to this address when the
  condition is met, creating a loop.

- **`cmp` and flags.** `cmp q0, q1` compares two values and sets the VM's
  condition flags:
  - `eq` is set when the operands are equal.
  - `lt` is set when the first operand is less than the second.

  `cmp` does not modify either operand — it only updates the flags.

- **Conditional jumps.** `jlt` (jump if less than) branches when the `lt` flag
  is set. The full set of conditional jumps is: `jeq`, `jne`, `jlt`, `jgt`,
  `jle`, `jge`. There is also `jmp` for unconditional jumps.

- **`inc` instruction.** `inc q0` increments the register by 1 in place. Its
  counterpart `dec` decrements by 1. Both work on integer and floating-point
  registers.

- **Loop structure.** This is the Nyx equivalent of a `for` or `while` loop:
  initialize a counter, perform work, compare, and conditionally jump back to
  the top. The loop exits by "falling through" when the condition is no longer
  true.

---

## 6. Function Calls

Calling a subroutine with `call` and returning with `ret`.

```/dev/null/functions.nyx#L1-9
.section text
_start:
    mov q0, 5
    mov q1, 3
    call add_numbers
    ; q0 now contains 8
    hlt

add_numbers:
    add q0, q0, q1
    ret
```

### Concepts

- **`call` instruction.** `call add_numbers` pushes the address of the *next*
  instruction onto the stack (the return address), then jumps to the
  `add_numbers` label. This is how the VM remembers where to resume after the
  function completes.

- **`ret` instruction.** `ret` pops the return address from the stack and jumps
  to it, transferring control back to the caller. Execution continues at the
  instruction immediately after the original `call`.

- **Register-based argument passing.** Nyx does not enforce a formal calling
  convention — arguments and return values are passed through registers by
  agreement between the caller and callee. In this example, `q0` and `q1` hold
  the inputs, and the result is left in `q0`.

- **Indirect calls.** `call` also accepts a register operand (e.g. `call q0`)
  for indirect / computed calls. External (FFI) functions require `.extern`
  declarations with type signatures (e.g. `.extern puts(ptr): i32`) and are
  called directly via libffi from shared libraries loaded with the `-l` flag.

---

## 7. Stack Operations

Using `push` and `pop` to save and restore register values.

```/dev/null/stack.nyx#L1-7
.section text
_start:
    mov q0, 42
    push q0             ; save q0 on stack
    mov q0, 0           ; use q0 for something else
    pop q0              ; restore q0 (back to 42)
    hlt
```

### Concepts

- **`push` instruction.** `push q0` decrements the stack pointer (`sp`) by the
  size of the value (8 bytes for a `q` register), then writes the value at the
  new `sp` address.

- **`pop` instruction.** `pop q0` reads the value at the current `sp`, stores
  it into `q0`, then increments `sp` by the appropriate size.

- **Stack grows downward.** The stack starts at the top of the memory block
  (initialized to the total memory size) and grows toward lower addresses.
  Every `push` moves `sp` down; every `pop` moves it back up.

- **Save/restore pattern.** The push-then-pop pattern is essential when you
  need to temporarily reuse a register without losing its value. This is
  especially common around function calls — the caller pushes any registers it
  needs preserved, calls the function, then pops them back afterward.

- **Size inference.** The data size pushed or popped is inferred from the
  register prefix. `push q0` pushes 8 bytes; `push b0` would push 1 byte.
  An explicit size prefix can also be used: `push dword 42`.

---

## 8. Writing to a File

A complete file I/O example: open a file, write to it, and close it.

```/dev/null/write_to_file.nyx#L1-22
.entry _start

#include "stdlib.nyx"

.section text
_start:
    mov q0, path
    mov q1, (O_WRONLY | O_CREAT | O_TRUNC)
    mov q2, 0o644
    mov q15, SYS_OPEN
    syscall
    push q0

    mov q1, message
    mov q2, 14
    mov q15, SYS_WRITE
    syscall

    pop q0
    mov q15, SYS_CLOSE
    syscall
    hlt

.section data
path:       .asciz "hello.txt"
message:    .asciz "Hello, world!\n"
```

### Concepts

- **`.entry` directive.** `.entry _start` explicitly declares the program's
  entry point. When omitted, the assembler defaults to the `_start` label.
  This directive is useful when the entry point has a different name or when
  you want to be explicit.

- **File I/O syscalls.** This program chains three syscalls:
  1. `SYS_OPEN` (`0x00`) — opens `"hello.txt"` for writing. `q0` points to
     the null-terminated path, `q1` holds the flags, `q2` holds the file
     permissions. On return, `q0` contains the new file descriptor.
  2. `SYS_WRITE` (`0x03`) — writes 14 bytes from `message` to the file. The
     fd is still in `q0` (restored from the stack after open).
  3. `SYS_CLOSE` (`0x01`) — closes the file descriptor.

- **Expression evaluation in immediates.** `(O_WRONLY | O_CREAT | O_TRUNC)` is
  a constant expression evaluated at compile time by the preprocessor. The
  parentheses group the bitwise OR of three `#define` constants from
  `stdlib.nyx`. On Linux x86-64, this evaluates to `1 | 64 | 512 = 577`.
  The preprocessor supports `+`, `-`, `*`, `/`, `|`, `&`, `^`, and `()` in
  expressions.

- **Octal literals.** `0o644` is an octal literal representing Unix file
  permissions (owner read/write, group read, others read). Nyx supports
  several numeric bases: decimal (`644`), hexadecimal (`0x1A4`), binary
  (`0b110100100`), and octal (`0o644`).

- **`.asciz` directive.** `.asciz "hello.txt"` embeds the string bytes followed
  by a null terminator (`0x00`). This is equivalent to `db "hello.txt", 0x00`.
  The related `.ascii` directive embeds the string *without* a null terminator.

- **Saving the file descriptor.** After `SYS_OPEN`, the returned fd is in `q0`.
  Since `SYS_WRITE` also uses `q0` for the fd, the program pushes `q0` onto
  the stack immediately after the open, then pops it back before the close.
  This is a common pattern when a syscall's return value is needed later but
  the register will be overwritten.

---

## 9. Memory Operations

Direct memory reads and writes using bracket-syntax addressing.

```/dev/null/memory.nyx#L1-13
.section text
_start:
    ; Store value to memory via address
    mov q0, buffer
    mov byte [q0], 0x41     ; Write 'A' to buffer[0]
    mov byte [q0 + 1], 0x42 ; Write 'B' to buffer[1]

    ; Read value from memory
    mov b1, [q0]             ; b1 = 0x41

    hlt

.section data
buffer:
    resb 64
```

### Concepts

- **Memory addressing with brackets.** Square brackets denote a memory access.
  `[q0]` means "the memory location whose address is in `q0`." The VM
  dereferences the register to read from or write to that address.

- **Register + offset addressing.** `[q0 + 1]` adds a constant offset to the
  base register before accessing memory. This is how you index into arrays and
  buffers — `q0` points to the start, and the offset selects a specific
  element.

- **Data size prefix for stores.** When storing an immediate value to memory,
  the assembler cannot infer the width from the immediate alone. A size prefix
  is required: `mov byte [q0], 0x41` writes exactly one byte. Valid prefixes
  are `byte`, `word`, `dword`, `qword`, `float`, and `double`.

- **Loads infer size from the register.** `mov b1, [q0]` reads one byte from
  memory because `b1` is a byte-width register. If the destination were `q1`
  instead, it would read 8 bytes. No size prefix is needed when the register
  determines the width.

- **`resb` directive.** `resb 64` reserves 64 zero-initialized bytes in the
  data section. The related directives `resw`, `resd`, and `resq` reserve
  words, dwords, and qwords respectively (each reserving N × element-size
  bytes).
