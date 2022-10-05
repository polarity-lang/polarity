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

## Project overview

```text
├── app                     CLI application
├── data                    Collection of convenient data structures
├── examples                Example code in the object language
├── ext/vscode              VSCode extension
├── lang                    Language implementation
│   ├── core                Core (typechecker, evaluator)
│   ├── lowering            Lowering concrete to abstract syntax tree
│   ├── parser              Parse text to concrete syntax tree
│   ├── printer             Print abstract syntax tree to text
│   └── syntax              Syntax tree definitions
├── util                    Utility libraries
│   ├── data                Collection of convenient data structures
│   ├── lsp                 LSP language server implementation
│   ├── tracer              Debugging library for generating trace output
│   └── tracer_macros       Procedural macros that generate trace output
└── web                     Web demo application
```

Please refer to the `README.md` files in the individual subprojects for further information.
