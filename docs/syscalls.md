# Syscalls

Syscalls are the VM's interface to the host operating system. They provide
file I/O, memory management, networking, and process control.

A syscall is invoked with the `syscall` instruction. The syscall number is
read from **`q15`**. Arguments are passed in specific registers (documented
per syscall below), and return values are placed in **`q0`** (or `d0` for
some networking calls).

---

## Syscall Table

| Number | Name          | Description                        |
|--------|---------------|------------------------------------|
| `0x00` | `sys_open`    | Open a file                        |
| `0x01` | `sys_close`   | Close a file descriptor            |
| `0x02` | `sys_read`    | Read from a file descriptor        |
| `0x03` | `sys_write`   | Write to a file descriptor         |
| `0x04` | `sys_malloc`  | Allocate dynamic memory            |
| `0x05` | `sys_free`    | Free dynamic memory                |
| `0x06` | `sys_socket`  | Create a network socket            |
| `0x07` | `sys_connect` | Connect a socket                   |
| `0x08` | `sys_bind`    | Bind a socket to an address        |
| `0x09` | `sys_listen`  | Listen on a socket                 |
| `0x0A` | `sys_accept`  | Accept a connection on a socket    |
| `0xFF` | `sys_exit`    | Exit the program                   |

---

## Standard Library Constants

The standard library (`std/stdlib.nyx`) defines named constants for every
syscall number and for the standard file descriptors:

```/dev/null/constants.nyx#L1-14
SYS_OPEN    = 0x00
SYS_CLOSE   = 0x01
SYS_READ    = 0x02
SYS_WRITE   = 0x03
SYS_MALLOC  = 0x04
SYS_FREE    = 0x05
SYS_SOCKET  = 0x06
SYS_CONNECT = 0x07
SYS_BIND    = 0x08
SYS_LISTEN  = 0x09
SYS_ACCEPT  = 0x0A
SYS_EXIT    = 0xFF

STDIN  = 0x00
STDOUT = 0x01
STDERR = 0x02
```

---

## File I/O

### sys_open — `0x00`

Open a file and return a file descriptor.

| Register | Direction | Description                                          |
|----------|-----------|------------------------------------------------------|
| `q0`     | in        | Pointer to a null-terminated file path in VM memory  |
| `d1`     | in        | Flags (`O_RDONLY`, `O_WRONLY`, `O_RDWR`, `O_CREAT`, `O_TRUNC`, …) |
| `w2`     | in        | File mode / permissions (e.g. `0o644`)               |
| `q0`     | out       | File descriptor on success                           |

---

### sys_close — `0x01`

Close an open file descriptor.

| Register | Direction | Description              |
|----------|-----------|--------------------------|
| `d0`     | in        | File descriptor to close |

No return value.

---

### sys_read — `0x02`

Read bytes from a file descriptor into VM memory.

| Register | Direction | Description                         |
|----------|-----------|-------------------------------------|
| `d0`     | in        | File descriptor                     |
| `q1`     | in        | Destination address in VM memory    |
| `q2`     | in        | Number of bytes to read             |
| `q0`     | out       | Number of bytes actually read       |

---

### sys_write — `0x03`

Write bytes from VM memory to a file descriptor.

| Register | Direction | Description                         |
|----------|-----------|-------------------------------------|
| `d0`     | in        | File descriptor                     |
| `q1`     | in        | Source address in VM memory         |
| `q2`     | in        | Number of bytes to write            |
| `q0`     | out       | Number of bytes actually written    |

> **Note:** The file descriptor is read from `d0` (a 32-bit view). Because
> `q0` and `d0` share the same physical register slot, loading a small value
> like `1` (stdout) into `q0` also populates `d0` correctly. Many examples
> therefore set `q0` for the fd — this works because the lower 32 bits are
> the same.

---

## Memory Management

### sys_malloc — `0x04`

Allocate a new block of dynamic memory.

| Register | Direction | Description                                    |
|----------|-----------|------------------------------------------------|
| `q0`     | in        | Size in bytes to allocate                      |
| `q0`     | out       | Start address of the allocated block           |

Internally, this adds a new `Block` to the MMU. The returned address is the
start of that block in the unified address space.

---

### sys_free — `0x05`

Free a previously allocated block of dynamic memory.

| Register | Direction | Description                                              |
|----------|-----------|----------------------------------------------------------|
| `q0`     | in        | Address of the block to free (must be the start address) |

No return value.

The address must exactly match the start address of a block that was returned
by `sys_malloc`. Passing any other address is an error.

---

## Networking

### sys_socket — `0x06`

Create a network socket.

| Register | Direction | Description                              |
|----------|-----------|------------------------------------------|
| `d0`     | in        | Domain (e.g. `AF_INET` = `2`)           |
| `d1`     | in        | Type (e.g. `SOCK_STREAM` = `1`)         |
| `d2`     | in        | Protocol (usually `0`)                   |
| `d0`     | out       | Socket file descriptor                   |

---

### sys_connect — `0x07`

Connect a socket to a remote address.

| Register | Direction | Description                                         |
|----------|-----------|-----------------------------------------------------|
| `d0`     | in        | Socket file descriptor                              |
| `q1`     | in        | Pointer to a `sockaddr_in` structure in VM memory   |
| `q0`     | out       | Result (`0` on success)                             |

#### `sockaddr_in` Layout

| Offset | Size    | Field      |
|--------|---------|------------|
| 0      | 2 bytes | `family`   |
| 2      | 2 bytes | `port`     |
| 4      | 4 bytes | `addr`     |
| 8      | 8 bytes | zero (pad) |

Total size: 16 bytes.

---

### sys_bind — `0x08`

Bind a socket to a local address.

| Register | Direction | Description                                         |
|----------|-----------|-----------------------------------------------------|
| `d0`     | in        | Socket file descriptor                              |
| `q1`     | in        | Pointer to a `sockaddr_in` structure in VM memory   |
| `q0`     | out       | Result (`0` on success)                             |

The `sockaddr_in` layout is the same as for `sys_connect`.

---

### sys_listen — `0x09`

Mark a socket as listening for incoming connections.

| Register | Direction | Description                          |
|----------|-----------|--------------------------------------|
| `d0`     | in        | Socket file descriptor               |
| `d1`     | in        | Backlog (max pending connections)    |
| `d0`     | out       | Result (`0` on success)             |

---

### sys_accept — `0x0A`

Accept an incoming connection on a listening socket.

| Register | Direction | Description                                                  |
|----------|-----------|--------------------------------------------------------------|
| `d0`     | in        | Socket file descriptor                                       |
| `q1`     | in        | Pointer to a `sockaddr_in` buffer in VM memory               |
| `q0`     | out       | New socket file descriptor for the accepted connection       |

The `sockaddr_in` buffer at `q1` is filled with the connecting client's
address information upon return.

---

## Process Control

### sys_exit — `0xFF`

Terminate the program immediately.

| Register | Direction | Description                  |
|----------|-----------|------------------------------|
| `b0`     | in        | Exit status code (8-bit)     |

This syscall does not return.

---

## Usage Example

A minimal "Hello, world!" program using `sys_write` and `sys_exit`:

```/dev/null/hello.nyx#L1-12
#include "stdlib.nyx"

.data
    msg: db "Hello, world!\n"

.text
_start:
    mov q15, SYS_WRITE      ; syscall number → q15
    mov d0, STDOUT           ; fd = 1
    lea q1, msg              ; source buffer
    mov q2, 14               ; byte count
    syscall                  ; invoke sys_write

    mov q15, SYS_EXIT        ; syscall number → q15
    mov b0, 0                ; exit status 0
    syscall                  ; invoke sys_exit
```
