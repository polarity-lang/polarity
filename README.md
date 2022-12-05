# Xfunc Implementation

Extending De-/Refunctionalization to the Lambda Cube

## Quickstart

Install Rust (e.g. via [rustup.rs](https://rustup.rs/)).

From the root of this repository, run:

```sh
cargo run -- run examples/example.xfn
```

Enable verbose output by supplying the `--trace` option like so:

```sh
cargo run -- --trace run examples/example.xfn
```

To pretty-print a file, run:

```sh
cargo run -- fmt examples/example.xfn
```

For more information about the CLI, run:

```sh
cargo run -- --help
```

## Installation

To locally install the executable, run:

```sh
cargo install --path app
```

By default, it gets installed to `~/.cargo/bin/xfunc`.

## Project overview

```text
├── app                     CLI application
├── examples                Example code in the object language
├── ext/vscode              VSCode extension
├── lang                    Language implementation
│   ├── core                Core (typechecker, evaluator)
│   ├── lifting             Lift local (co)matches to top-level definitions
│   ├── lowering            Lowering concrete to (untyped) abstract syntax tree
│   ├── parser              Parse text to concrete syntax tree
│   ├── printer             Print abstract syntax tree to text
│   ├── renaming            Rename abstract syntax tree s.t. it can be reparsed
│   ├── source              Index data structures for annotated source code files and spans
│   ├── syntax              Syntax tree definitions
│   └── xfunc               De-/Refunctionalization implementation
├── test                    Integration tests
│   ├── suites              Test cases
│   └── test-runner         Test runner
├── util                    Utility libraries
│   ├── console             Print function that works native and on WASM
│   ├── data                Collection of convenient data structures
│   ├── lsp                 LSP language server implementation
│   ├── miette_util         Convert source code spans
│   ├── tracer              Debugging library for generating trace output
│   └── tracer_macros       Procedural macros that generate trace output
└── web                     Web demo application
```

Please refer to the `README.md` files in the individual subprojects for further information.
