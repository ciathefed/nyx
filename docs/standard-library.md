# Standard Library

The Nyx standard library is a collection of assembly-level include files that
ship in the `std/` directory. They provide common constants, string routines,
output helpers, and networking utilities so that user programs don't have to
reimplement them from scratch.

---

## Including the Standard Library

Standard library files are included with the `#include` directive:

```/dev/null/example.nyx#L1-3
#include "stdlib.nyx"
#include "string.nyx"
#include "print.nyx"
```

For the preprocessor to find these files, do one of the following:

- Set the **`NYX_STDLIB_PATH`** environment variable to point to the `std/`
  directory.
- Pass **`-i <path-to-std>`** on the command line to add it as an include
  search path.

Every standard library file uses an **include guard** (`#ifndef` / `#define` /
`#endif`) so it is safe to include the same file more than once.

---

## Files at a Glance

| File | Guard | Description |
|------|-------|-------------|
| `stdlib.nyx` | `NYX_STDLIB` | Syscall numbers, file descriptors, booleans, platform file flags |
| `string.nyx` | `NYX_STRING` | String and memory manipulation functions |
| `print.nyx` | `NYX_PRINT` | Console output functions (includes `string.nyx` and `stdlib.nyx`) |
| `socket.nyx` | `NYX_NET` | Networking constants and byte-order functions (Linux x86_64 only) |

---

## `stdlib.nyx`

Defines system call numbers and standard constants used throughout the rest of
the library. This file is **constants only** — it emits no code.

### Syscall Numbers

| Constant | Value | Description |
|----------|-------|-------------|
| `SYS_OPEN` | `0x00` | Open a file |
| `SYS_CLOSE` | `0x01` | Close a file descriptor |
| `SYS_READ` | `0x02` | Read from a file descriptor |
| `SYS_WRITE` | `0x03` | Write to a file descriptor |
| `SYS_MALLOC` | `0x04` | Allocate memory |
| `SYS_FREE` | `0x05` | Free allocated memory |
| `SYS_SOCKET` | `0x06` | Create a socket |
| `SYS_CONNECT` | `0x07` | Connect a socket |
| `SYS_BIND` | `0x08` | Bind a socket to an address |
| `SYS_LISTEN` | `0x09` | Listen for connections |
| `SYS_ACCEPT` | `0x0A` | Accept an incoming connection |
| `SYS_EXIT` | `0xFF` | Exit the program |

### Standard File Descriptors

| Constant | Value | Description |
|----------|-------|-------------|
| `STDIN` | `0x00` | Standard input |
| `STDOUT` | `0x01` | Standard output |
| `STDERR` | `0x02` | Standard error |

### Boolean Constants

| Constant | Value |
|----------|-------|
| `TRUE` | `1` |
| `FALSE` | `0` |

### Platform-Specific File Flags

File-open flags are conditionally defined based on the target platform. The
preprocessor automatically defines `__LINUX__`, `__MACOS__`, `__X86_64__`,
`__AARCH64__`, etc., so the correct set is selected at compile time.

#### Linux x86_64

| Constant | Value |
|----------|-------|
| `O_RDONLY` | 0 |
| `O_WRONLY` | 1 |
| `O_RDWR` | 2 |
| `O_CREAT` | 64 |
| `O_EXCL` | 128 |
| `O_TRUNC` | 512 |
| `O_APPEND` | 1024 |
| `O_NONBLOCK` | 2048 |

#### macOS aarch64

| Constant | Value |
|----------|-------|
| `O_ACCMODE` | `0x3` |
| `O_RDONLY` | 0 |
| `O_WRONLY` | 1 |
| `O_RDWR` | 2 |
| `O_APPEND` | 8 |
| `O_CREAT` | 512 |
| `O_TRUNC` | 1024 |
| `O_EXCL` | 2048 |
| `O_ASYNC` | `0x40` |
| `O_SYNC` | `0x80` |
| `O_NONBLOCK` | `0x4` |
| `O_NOFOLLOW` | `0x100` |
| `O_SHLOCK` | `0x10` |
| `O_EXLOCK` | `0x20` |
| `O_FSYNC` | (alias for `O_SYNC`) |
| `O_NDELAY` | (alias for `O_NONBLOCK`) |

> **Note:** Compiling on an unsupported platform/architecture combination
> triggers a `#error`.

---

## `string.nyx`

String and memory manipulation functions. All routines follow the Nyx calling
convention — arguments are passed in registers and functions are invoked with
`call`.

### `strcpy`

```/dev/null/sig.nyx#L1
strcpy(q0: *mut u8, q1: *const u8) -> void
```

Copies the null-terminated string at `q1` to the buffer at `q0`, including the
terminating null byte.

### `strcat`

```/dev/null/sig.nyx#L1
strcat(q0: *mut u8, q1: *const u8) -> void
```

Appends the null-terminated string `q1` to the end of the null-terminated
string `q0`. The caller must ensure `q0` has enough space for the combined
result.

### `strlen`

```/dev/null/sig.nyx#L1
strlen(q0: *const u8) -> q0: usize
```

Calculates the length of the null-terminated string at `q0` (not counting the
null terminator). The length is returned in `q0`.

### `strcmp`

```/dev/null/sig.nyx#L1
strcmp(q0: *const u8, q1: *const u8) -> q0: i32
```

Compares two null-terminated strings byte-by-byte. Returns `0` in `q0` if the
strings are equal, non-zero otherwise.

### `memcpy`

```/dev/null/sig.nyx#L1
memcpy(q0: *mut u8, q1: *const u8, q2: usize) -> void
```

Copies `q2` bytes from the address in `q1` to the address in `q0`. The memory
regions must not overlap.

### `memset`

```/dev/null/sig.nyx#L1
memset(q0: *mut u8, b1: u8, q2: usize) -> void
```

Sets `q2` bytes starting at `q0` to the byte value in `b1`.

### `memcmp`

```/dev/null/sig.nyx#L1
memcmp(q0: *const u8, q1: *const u8, q2: usize) -> q0: i32
```

Compares `q2` bytes starting at `q0` and `q1`. Returns `0` in `q0` if all
bytes are equal, non-zero otherwise.

---

## `print.nyx`

Console output functions. This file internally includes `string.nyx` and
`stdlib.nyx`, so you do not need to include them separately when using
`print.nyx`.

### `print_string`

```/dev/null/sig.nyx#L1
print_string(q0: *const u8) -> void
```

Prints a null-terminated string to standard output. Internally calls `strlen`
to determine the string length, then issues a `SYS_WRITE` syscall on `STDOUT`.
All used registers are preserved (pushed/popped around the call).

### `print_integer`

```/dev/null/sig.nyx#L1
print_integer(q0: i64) -> void
```

Converts a signed 64-bit integer to its decimal string representation and
prints it to standard output. Negative numbers are printed with a leading `-`
character. Uses a 24-byte stack buffer for the conversion. All used registers
are preserved.

---

## `socket.nyx`

Networking utility constants and byte-order conversion functions. Currently
**Linux x86_64 only** — compiling on macOS triggers a `#error`.

### Socket Type Constants

| Constant | Value |
|----------|-------|
| `SOCK_STREAM` | 1 |
| `SOCK_DGRAM` | 2 |
| `SOCK_RAW` | 3 |
| `SOCK_RDM` | 4 |
| `SOCK_SEQPACKET` | 5 |
| `SOCK_PACKET` | 10 |

### Address Family Constants

| Constant | Value |
|----------|-------|
| `AF_UNSPEC` | 0 |
| `AF_UNIX` | 1 |
| `AF_INET` | 2 |
| `AF_AX25` | 3 |
| `AF_IPX` | 4 |
| `AF_APPLETALK` | 5 |
| `AF_NETROM` | 6 |
| `AF_BRIDGE` | 7 |
| `AF_AAL5` | 8 |
| `AF_X25` | 9 |
| `AF_INET6` | 10 |
| `AF_MAX` | 12 |

### Protocol Family Aliases

Every `AF_*` constant has a corresponding `PF_*` alias with the same value
(e.g. `PF_INET` is defined as `AF_INET`).

### Other Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `SOMAXCONN` | 128 | Maximum listen backlog |

### `htons`

```/dev/null/sig.nyx#L1
htons(w0: u16) -> w0: u16
```

Converts a 16-bit value from host byte order (little-endian) to network byte
order (big-endian) by swapping the two bytes. The result is returned in `w0`.
All working registers are preserved.

### `htonl`

```/dev/null/sig.nyx#L1
htonl(d0: u32) -> d0: u32
```

Converts a 32-bit value from host byte order (little-endian) to network byte
order (big-endian) by reversing the four bytes. The result is returned in `d0`.
All working registers are preserved.

---

## Usage Example

A minimal program that prints a message using the standard library:

```/dev/null/hello.nyx#L1-14
#include "stdlib.nyx"
#include "string.nyx"
#include "print.nyx"

.section text
_start:
    mov q0, message
    call print_string
    hlt

.section data
message:
    .asciz "Hello from the standard library!\n"
```

Compile and run:

```/dev/null/shell.txt#L1-2
nyx run hello.nyx -i ./std
```
