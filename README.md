# Nyx

64-bit register-based virtual machine and compiler written in [Rust](https://www.rust-lang.org/). Nyx supports compiling a simple assembly-like language into bytecode and executing it on a custom VM.

> [!WARNING]
> Nyx is in very early development. It is incomplete, lightly tested, and many features are missing. Expect breaking changes and unstable behavior.

## Features

* Custom assembly-like language
* Compiler to bytecode
* Virtual machine with registers, stack, memory, and basic instructions
* Written in safe, modern [Rust](https://www.rust-lang.org/)

## Installation

Clone the repo and build:

```sh
git clone https://github.com/ciathefed/nyx.git
cd nyx
cargo build --release
```

This will create the binary at `target/release/nyx`.

## Usage

### Compile a source file to bytecode

```sh
nyx build examples/hello.nyx -o hello.nyb
```

### Compile and run source file directly

```sh
nyx run examples/hello.nyx
```

You can also specify memory size (default is 4096 bytes):

```sh
nyx run examples/hello.nyx --mem 8192
```

### Run precompiled bytecode

```sh
nyx execute hello.nyb
```

With a custom memory size:

```sh
nyx execute hello.nyb --mem 16384
```

## Example Program

```asm
mov q0, 1337
push q0
pop QWORD d0
hlt
```

## License

This project is licensed under the [MIT License](./LICENSE)
