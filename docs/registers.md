# Registers

Nyx VM exposes three classes of registers: **general-purpose**, **floating-point**,
and **special**. Every register is encoded as a single byte in the bytecode —
its position in the canonical `Register` enum determines that byte value.

---

## General-Purpose Registers (GPRs)

There are **16 physical GPR slots** (`gpr0`–`gpr15`), each stored internally as
a `u64`. A single slot can be accessed through four different *views* (aliases)
that control how many bits are read or written.

| Prefix | View  | Width   | Registers  |
|--------|-------|---------|------------|
| `b`    | byte  | 8-bit   | `b0`–`b15`  |
| `w`    | word  | 16-bit  | `w0`–`w15`  |
| `d`    | dword | 32-bit  | `d0`–`d15`  |
| `q`    | qword | 64-bit  | `q0`–`q15`  |

All four views of the same index share the **same physical slot**:
`b3`, `w3`, `d3`, and `q3` all operate on `gpr3`.

### Read Semantics

Reading a smaller view returns the value masked to its width:

| View  | Mask                 |
|-------|----------------------|
| byte  | `value & 0xFF`       |
| word  | `value & 0xFFFF`     |
| dword | `value & 0xFFFF_FFFF`|
| qword | `value` (no mask)    |

### Write Semantics

| View  | Behaviour                                                        |
|-------|------------------------------------------------------------------|
| byte  | Preserves upper 56 bits, replaces the lowest 8 bits.             |
| word  | Preserves upper 48 bits, replaces the lowest 16 bits.            |
| dword | **Zeros the entire register**, then writes the 32-bit value (zero-extends to 64 bits). |
| qword | Overwrites the entire 64-bit register.                           |

> **Note:** The dword write rule matches x86-64 behaviour — writing a 32-bit
> register implicitly zero-extends into the full 64-bit slot.

---

## Floating-Point Registers (FPRs)

There are **32 physical FPR slots** (`fpr0`–`fpr15` at indices `0x00`–`0x0F` and
`dpr0`–`dpr15` at indices `0x10`–`0x1F`), each stored internally as a `u64`.
Two views are provided:

| Prefix | View   | Width              | Registers   | Physical Slots       |
|--------|--------|--------------------|-------------|----------------------|
| `ff`   | float  | 32-bit IEEE 754    | `ff0`–`ff15` | `fpr0`–`fpr15` (`0x00`–`0x0F`) |
| `dd`   | double | 64-bit IEEE 754    | `dd0`–`dd15` | `dpr0`–`dpr15` (`0x10`–`0x1F`) |

> **Important:** `ff` and `dd` registers use **different physical indices**.
> `ff0` maps to `fpr0` (index `0x00`) while `dd0` maps to `dpr0` (index `0x10`).
> They do **not** alias each other — writing to `ff0` never affects `dd0`.

---

## Special Registers

Three special registers are stored in a `[3]usize` array. All are accessed as
qword (64-bit) values.

| Name | Index | Purpose                                                         |
|------|-------|-----------------------------------------------------------------|
| `ip` | 0     | **Instruction Pointer** — current position in program memory.   |
| `sp` | 1     | **Stack Pointer** — initialized to total memory size (stack grows downward). |
| `bp` | 2     | **Base Pointer** — initialized to 0; marks the current stack frame base.     |

---

## Data Sizes

The VM associates a `DataSize` with every register access. The size is inferred
from the register name prefix:

| Prefix | DataSize | Byte Width |
|--------|----------|------------|
| `b`    | byte     | 1          |
| `w`    | word     | 2          |
| `d`    | dword    | 4          |
| `q`    | qword    | 8          |
| `ff`   | float    | 4          |
| `dd`   | double   | 8          |

Special registers (`ip`, `sp`, `bp`) are always qword-sized.

---

## Summary Table

| Prefix | View    | Size    | Bits              | Registers           |
|--------|---------|---------|--------------------|---------------------|
| `b`    | byte    | 1 byte  | 8-bit              | `b0`–`b15`          |
| `w`    | word    | 2 bytes | 16-bit             | `w0`–`w15`          |
| `d`    | dword   | 4 bytes | 32-bit             | `d0`–`d15`          |
| `q`    | qword   | 8 bytes | 64-bit             | `q0`–`q15`          |
| `ff`   | float   | 4 bytes | 32-bit IEEE 754    | `ff0`–`ff15`        |
| `dd`   | double  | 8 bytes | 64-bit IEEE 754    | `dd0`–`dd15`        |
| —      | special | 8 bytes | 64-bit             | `ip`, `sp`, `bp`    |

---

## Register Encoding

Every register is encoded as a **single byte** in the bytecode. The byte value
equals the register's position in the `Register` enum, which is ordered as
follows:

```/dev/null/enum.txt#L1-5
b0, w0, d0, q0, ff0, dd0,
b1, w1, d1, q1, ff1, dd1,
...
b15, w15, d15, q15, ff15, dd15,
ip, sp, bp
```

Each group of six entries corresponds to one GPR index (0–15) — the four
integer views followed by the two floating-point views. The three special
registers come last.

For quick reference, the encoding byte for register index *n* and view offset
*v* is `n * 6 + v`, where *v* is:

| View  | *v* |
|-------|-----|
| `b`   | 0   |
| `w`   | 1   |
| `d`   | 2   |
| `q`   | 3   |
| `ff`  | 4   |
| `dd`  | 5   |

The special registers follow at offsets `96` (`ip`), `97` (`sp`), and `98` (`bp`).

---

## Conventions

- **`q15`** — syscall number register. Set `q15` to the desired syscall ID
  before executing a `syscall` instruction.
- **`q0`–`q2`** — commonly used for syscall arguments (file descriptor, buffer
  address, length, etc.).
- **`b0`** — holds the exit status for `sysExit`.
