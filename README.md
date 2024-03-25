# Polarity

A programming language with dependent data and codata types.
Installation instructions and the language documentation is available at [polarity-lang.github.io](https://polarity-lang.github.io/).

## Requirements

Install Rust (e.g. via [rustup.rs](https://rustup.rs/)).

## Installation

To locally install the executable, run:

```sh
make install
```

By default, it gets installed to `~/.cargo/bin/pol`.

## Quickstart

From the root of this repository, run:

```sh
pol run examples/example.pol
```

Enable verbose output by supplying the `--trace` option like so:

```sh
pol --trace run examples/example.pol
```

To pretty-print a file, run:

```sh
pol fmt examples/example.pol
```

For more information about the CLI, run:

```sh
pol --help
```

## Project overview

```text
├── app                     CLI application
├── examples                Example code in the object language
├── lang                    Language implementation
│   ├── elaborator          Elaborating an untyped syntax tree into a typed syntax tree.
│   ├── lifting             Lift local (co)matches to top-level definitions
│   ├── lowering            Lowering concrete to (untyped) abstract syntax tree
│   ├── parser              Parse text to concrete syntax tree
│   ├── printer             Print abstract syntax tree to text
│   ├── query               Index data structures for annotated source code files and spans
│   ├── renaming            Rename abstract syntax tree s.t. it can be reparsed
│   ├── syntax              Syntax tree definitions
│   └── xfunc               De-/Refunctionalization implementation
├── test                    Integration tests
│   ├── suites              Test cases
│   └── test-runner         Test runner
├── util                    Utility libraries
│   ├── lsp                 LSP language server implementation
│   ├── miette_util         Convert source code spans
│   ├── tracer              Debugging library for generating trace output
│   └── tracer_macros       Procedural macros that generate trace output
└── web                     Web demo application
```

Please refer to the `README.md` files in the individual subprojects for further information.


## Licenses

Except for the code in the `web` directory, this project is distributed under the terms of both the MIT license and the Apache License 2.0.
See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.

The code contained in the `web` directory is based on [tower-lsp-web-demo](https://github.com/silvanshade/tower-lsp-web-demo/) by Darin Morrison.
It is licensed under Apache-2.0 WITH LLVM-exception.
