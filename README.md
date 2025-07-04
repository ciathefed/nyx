# Nyx

64-bit register-based virtual machine and compiler written in [Rust](https://www.rust-lang.org/). Nyx supports compiling a simple assembly-like language into bytecode and executing it on a custom VM.

![tests](https://img.shields.io/github/actions/workflow/status/ciathefed/nyx/rust.yml?label=tests%20%F0%9F%A7%AA&style=flat-square)

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
.section text
_start:
    mov q0, 1
    mov q1, message
    mov q2, 14
    mov q15, 3
    syscall
    hlt

.section data
message:
    db "Hello, world!\n", 0x00
```

## Contributing

Contributions are welcome. If you find a bug or want to add a feature, open an issue or pull request.

To contribute code:

1. Fork the repository
2. Create a new branch
3. Make your changes
4. Open a pull request with a clear description

Please follow the [Conventional Commits](https://www.conventionalcommits.org/) format for commit messages. Examples:

- `fix: handle empty source input in reporter`
- `feat: add support for multiple source files`
- `refactor: simplify diagnostic builder`

Keep changes focused and minimal. Include tests when appropriate.

## License

This project is licensed under the [MIT License](./LICENSE)
